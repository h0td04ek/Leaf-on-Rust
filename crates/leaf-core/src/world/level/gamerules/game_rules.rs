//! `net.minecraft.world.level.gamerules.GameRules` — реестр ~60
//! ванильных правил + хранилище `GameRuleMap`.
//!
//! Отличия от Java:
//! - Brigadier/DFU-части заменены строковой сериализацией и
//!   прямым NBT-кодеком (`to_nbt`/`from_nbt` ↔ level.dat
//!   `GameRules`).
//! - Paper `DISABLE_LIMITS` (`paper.disableGameRuleLimits`) —
//!   читается из env `PAPER_DISABLE_GAMERULE_LIMITS`.
//! - Paper per-world `ServerLevel` callback в `set` — через
//!   `set_with_callback`; plugin API отложен.

use std::sync::Once;

use crate::nbt::tag::Tag;
use crate::world::flag::feature_flag_set::FeatureFlagSet;

use super::game_rule::{
    game_rule_bool, game_rule_int, game_rule_registry, GameRule, GameRuleArgument,
    GameRuleRegistry,
};
use super::game_rule_category as cat;
use super::game_rule_map::GameRuleMap;
use super::game_rule_type::GameRuleType;

/// `GameRuleTypeVisitor` — Java interface с default-методами.
pub trait GameRuleTypeVisitor {
    /// `visit(GameRule<?>)` — нетипизированный хук.
    fn visit_ref(&mut self, _rule: &super::game_rule_map::GameRuleRef) {}
    /// `visitBoolean(rule)`.
    fn visit_boolean(&mut self, _rule: &GameRule<bool>) {}
    /// `visitInteger(rule)`.
    fn visit_integer(&mut self, _rule: &GameRule<i32>) {}
}

/// `GameRules` — загруженный набор правил мира.
#[derive(Clone)]
pub struct GameRules {
    /// `rules`.
    pub rules: GameRuleMap,
    enabled_features: FeatureFlagSet,
}

impl GameRules {
    /// `new GameRules(enabledFeatures)` — все правила, чьи
    /// `requiredFeatures` ⊆ enabledFeatures, с дефолтами.
    pub fn new(enabled_features: FeatureFlagSet) -> Self {
        bootstrap();
        let registry = game_rule_registry().lock().unwrap();
        let mut map = GameRuleMap::new();
        for inner in registry.entries() {
            if inner.required_features.is_subset_of(&enabled_features) {
                map.set_default(inner.clone());
            }
        }
        Self { rules: map, enabled_features }
    }

    /// `new GameRules(enabledFeatures, rules)` — merge поверх
    /// дефолтов (Java `setFromIf(rules, this.rules::has)`).
    pub fn with_overrides(enabled_features: FeatureFlagSet, overrides: GameRuleMap) -> Self {
        let mut g = Self::new(enabled_features);
        let existing: std::collections::HashSet<usize> = g
            .rules
            .key_set()
            .iter()
            .map(|r| r.inner.game_rule_index)
            .collect();
        g.rules
            .set_from_if(&overrides, move |r| existing.contains(&r.inner.game_rule_index));
        g
    }

    /// `codec(enabledFeatures)` → NBT: `to_nbt`/`from_nbt`.
    pub fn to_nbt(&self) -> Tag {
        self.rules.to_nbt()
    }

    /// `from_nbt` — загрузка из level.dat.
    pub fn from_nbt(tag: &Tag, enabled_features: FeatureFlagSet) -> Self {
        bootstrap();
        // guard ограничен скоупом — `Self::new` сам берёт лок
        let map = {
            let registry = game_rule_registry().lock().unwrap();
            GameRuleMap::from_nbt(tag, &registry)
        };
        let mut g = Self::new(enabled_features.clone());
        g.rules.set_from_if(&map, |_| true);
        g
    }

    /// `availableRules()`.
    pub fn available_rules(&self) -> Vec<super::game_rule_map::GameRuleRef> {
        self.rules.key_set()
    }

    /// `get(rule)` → `Option<T>`.
    pub fn get<T: Clone + Send + Sync + 'static>(&self, rule: &GameRule<T>) -> Option<T> {
        self.rules.get(rule)
    }

    /// `get(rule)` с дефолтом при отсутствии.
    pub fn get_or_default<T: Clone + Send + Sync + 'static>(&self, rule: &GameRule<T>) -> T {
        self.rules.get(rule).unwrap_or_else(|| rule.default_value())
    }

    /// `get(rule, gameRuleOverride)` — Java-версия для bool-правил.
    pub fn get_bool_with_override(&self, rule: &GameRule<bool>, game_rule_override: Option<bool>) -> bool {
        if let Some(v) = game_rule_override {
            return v;
        }
        self.get_or_default(rule)
    }

    /// `set(rule, value)` — Paper-вариант без level-callback.
    pub fn set<T: Clone + Send + Sync + 'static>(&mut self, rule: &GameRule<T>, value: T) {
        self.rules.set(rule, value);
    }

    /// `set(rule, value)` + Paper `onGameRuleChanged` callback.
    pub fn set_with_callback<T: Clone + Send + Sync + 'static>(
        &mut self,
        rule: &GameRule<T>,
        value: T,
        on_change: impl Fn(&super::game_rule_map::GameRuleRef),
    ) {
        self.rules.set(rule, value);
        on_change(&super::game_rule_map::GameRuleRef::from_inner(rule.inner_arc()));
    }

    /// `setAll(other)` — скопировать все значения.
    pub fn set_all(&mut self, other: &GameRules) {
        self.rules.set_from_if(&other.rules, |_| true);
    }

    /// `setAll(rules)` — GameRuleMap-вариант.
    pub fn set_all_map(&mut self, rules: &GameRuleMap) {
        self.rules.set_from_if(rules, |_| true);
    }

    /// `copy(enabledFeatures)`.
    pub fn copy(&self, enabled_features: FeatureFlagSet) -> Self {
        let mut g = Self::new(enabled_features);
        g.rules = self.rules.copy_of();
        g
    }

    /// `visitGameRuleTypes(visitor)`.
    pub fn visit_game_rule_types(&self, visitor: &mut dyn GameRuleTypeVisitor) {
        for rule_ref in self.rules.key_set() {
            visitor.visit_ref(&rule_ref);
            call_visitor_typed(&rule_ref, visitor);
        }
    }

    /// `getAsString(rule)` — serialize(get(rule)).
    pub fn get_as_string<T: Clone + Send + Sync + 'static>(&self, rule: &GameRule<T>) -> String {
        let v = self.get_or_default(rule);
        rule.serialize(&v)
    }

    /// `getAsString` для bool.
    pub fn get_as_string_bool(&self, rule: &GameRule<bool>) -> String {
        self.get_or_default(rule).to_string()
    }

    /// `getAsString` для int.
    pub fn get_as_string_int(&self, rule: &GameRule<i32>) -> String {
        self.get_or_default(rule).to_string()
    }
}

impl std::fmt::Debug for GameRules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameRules")
            .field("rules", &self.rules.size())
            .finish()
    }
}

/// Внутренний вызов typed-visitor'а по типу правила.
fn call_visitor_typed(rule: &super::game_rule_map::GameRuleRef, visitor: &mut dyn GameRuleTypeVisitor) {
    let index = rule.inner.game_rule_index;
    let inner = rule.inner.clone();
    match inner.game_rule_type {
        GameRuleType::Bool => {
            let r = GameRule::<bool>::from_inner(inner);
            visitor.visit_boolean(&r);
            let _ = index;
        }
        GameRuleType::Int => {
            let r = GameRule::<i32>::from_inner(inner);
            visitor.visit_integer(&r);
        }
    }
}

/// `GameRules.bootstrap(registry)` — регистрирует все ванильные
/// правила (один раз).
pub fn bootstrap() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let mut reg = game_rule_registry().lock().unwrap();
        register_all(&mut reg);
    });
}

/// Полная регистрация ванильных правил (порядок Java).
fn register_all(reg: &mut GameRuleRegistry) {
    use cat::*;
    let features_empty = FeatureFlagSet::empty();
    let features_minecart = FeatureFlagSet::minecart_improvements();

    // UPDATES
    register_bool(reg, "advance_time", &updates(), true, &features_empty);
    register_bool(reg, "advance_weather", &updates(), true, &features_empty);
    register_bool(reg, "lava_source_conversion", &updates(), false, &features_empty);
    register_bool(reg, "spread_vines", &updates(), true, &features_empty);
    register_bool(reg, "water_source_conversion", &updates(), true, &features_empty);
    register_int(reg, "fire_spread_radius_around_player", &updates(), 128, -1, i32::MAX, &features_empty);
    register_int_bounded(reg, "max_snow_accumulation_height", &updates(), 1, 0, 8, &features_empty);
    register_int(reg, "random_tick_speed", &updates(), 3, 0, i32::MAX, &features_empty);

    // MISC
    register_bool(reg, "allow_entering_nether_using_portals", &misc(), true, &features_empty);
    register_bool(reg, "command_blocks_work", &misc(), true, &features_empty);
    register_bool(reg, "global_sound_events", &misc(), true, &features_empty);
    register_bool(reg, "reduced_debug_info", &misc(), false, &features_empty);
    register_bool(reg, "spawner_blocks_work", &misc(), true, &features_empty);
    register_bool(reg, "tnt_explodes", &misc(), true, &features_empty);
    register_int(reg, "max_block_modifications", &misc(), 32768, 1, i32::MAX, &features_empty);
    register_int(reg, "max_command_forks", &misc(), 65536, 0, i32::MAX, &features_empty);
    register_int(reg, "max_command_sequence_length", &misc(), 65536, 0, i32::MAX, &features_empty);
    register_int_bounded(reg, "max_minecart_speed", &misc(), 8, 1, 1000, &features_minecart);

    // DROPS
    register_bool(reg, "block_drops", &drops(), true, &features_empty);
    register_bool(reg, "block_explosion_drop_decay", &drops(), true, &features_empty);
    register_bool(reg, "entity_drops", &drops(), true, &features_empty);
    register_bool(reg, "mob_explosion_drop_decay", &drops(), true, &features_empty);
    register_bool(reg, "projectiles_can_break_blocks", &drops(), true, &features_empty);
    register_bool(reg, "tnt_explosion_drop_decay", &drops(), false, &features_empty);

    // PLAYER
    register_bool(reg, "drowning_damage", &player(), true, &features_empty);
    register_bool(reg, "elytra_movement_check", &player(), true, &features_empty);
    register_bool(reg, "ender_pearls_vanish_on_death", &player(), true, &features_empty);
    register_bool(reg, "fall_damage", &player(), true, &features_empty);
    register_bool(reg, "fire_damage", &player(), true, &features_empty);
    register_bool(reg, "freeze_damage", &player(), true, &features_empty);
    register_bool(reg, "immediate_respawn", &player(), false, &features_empty);
    register_bool(reg, "keep_inventory", &player(), false, &features_empty);
    register_bool(reg, "limited_crafting", &player(), false, &features_empty);
    register_bool(reg, "locator_bar", &player(), true, &features_empty);
    register_bool(reg, "natural_health_regeneration", &player(), true, &features_empty);
    register_bool(reg, "player_movement_check", &player(), true, &features_empty);
    register_bool(reg, "pvp", &player(), true, &features_empty);
    register_bool(reg, "spectators_generate_chunks", &player(), true, &features_empty);
    register_int(reg, "players_nether_portal_creative_delay", &player(), 0, 0, i32::MAX, &features_empty);
    register_int(reg, "players_nether_portal_default_delay", &player(), 80, 0, i32::MAX, &features_empty);
    register_int_bounded(reg, "players_sleeping_percentage", &player(), 100, 0, 100, &features_empty);
    register_int(reg, "respawn_radius", &player(), 10, 0, i32::MAX, &features_empty);

    // MOBS
    register_bool(reg, "forgive_dead_players", &mobs(), true, &features_empty);
    register_bool(reg, "mob_drops", &mobs(), true, &features_empty);
    register_bool(reg, "mob_griefing", &mobs(), true, &features_empty);
    register_bool(reg, "raids", &mobs(), true, &features_empty);
    register_bool(reg, "universal_anger", &mobs(), false, &features_empty);
    register_int(reg, "max_entity_cramming", &mobs(), 24, 0, i32::MAX, &features_empty);

    // SPAWNING
    register_bool(reg, "spawn_mobs", &spawning(), true, &features_empty);
    register_bool(reg, "spawn_monsters", &spawning(), true, &features_empty);
    register_bool(reg, "spawn_patrols", &spawning(), true, &features_empty);
    register_bool(reg, "spawn_phantoms", &spawning(), true, &features_empty);
    register_bool(reg, "spawn_wandering_traders", &spawning(), true, &features_empty);
    register_bool(reg, "spawn_wardens", &spawning(), true, &features_empty);

    // CHAT
    register_bool(reg, "command_block_output", &chat(), true, &features_empty);
    register_bool(reg, "log_admin_commands", &chat(), true, &features_empty);
    register_bool(reg, "send_command_feedback", &chat(), true, &features_empty);
    register_bool(reg, "show_advancement_messages", &chat(), true, &features_empty);
    register_bool(reg, "show_death_messages", &chat(), true, &features_empty);
}

/// `registerBoolean`.
fn register_bool(
    reg: &mut GameRuleRegistry,
    name: &str,
    category: &super::game_rule_category::GameRuleCategory,
    default: bool,
    features: &FeatureFlagSet,
) {
    let cat = category.clone();
    let feats = features.clone();
    let parse: std::sync::Arc<dyn Fn(&str) -> Option<Box<dyn std::any::Any + Send + Sync>> + Send + Sync> =
        std::sync::Arc::new(|s| s.parse::<bool>().ok().map(|v| Box::new(v) as Box<dyn std::any::Any + Send + Sync>));
    reg.register(
        name,
        cat,
        GameRuleType::Bool,
        GameRuleArgument::Bool,
        visitor_erased_bool,
        default,
        |b| *b as i32,
        parse,
        feats,
    );
}

/// `registerInteger(name, cat, default, min)` — max = i32::MAX.
fn register_int(
    reg: &mut GameRuleRegistry,
    name: &str,
    category: &super::game_rule_category::GameRuleCategory,
    default: i32,
    min: i32,
    max: i32,
    features: &FeatureFlagSet,
) {
    register_int_inner(reg, name, category, default, min, max, features);
}

/// `registerInteger(name, cat, default, min, max)`.
fn register_int_bounded(
    reg: &mut GameRuleRegistry,
    name: &str,
    category: &super::game_rule_category::GameRuleCategory,
    default: i32,
    min: i32,
    max: i32,
    features: &FeatureFlagSet,
) {
    register_int_inner(reg, name, category, default, min, max, features);
}

fn register_int_inner(
    reg: &mut GameRuleRegistry,
    name: &str,
    category: &super::game_rule_category::GameRuleCategory,
    default: i32,
    min: i32,
    max: i32,
    features: &FeatureFlagSet,
) {
    let cat = category.clone();
    let feats = features.clone();
    let parse: std::sync::Arc<dyn Fn(&str) -> Option<Box<dyn std::any::Any + Send + Sync>> + Send + Sync> = std::sync::Arc::new(move |s| {
        s.parse::<i32>()
            .ok()
            .filter(|v| *v >= min && *v <= max)
            .map(|v| Box::new(v) as Box<dyn std::any::Any + Send + Sync>)
    });
    reg.register(
        name,
        cat,
        GameRuleType::Int,
        GameRuleArgument::Int { min, max },
        visitor_erased_int,
        default,
        |i| *i,
        parse,
        feats,
    );
}

/// Type-erased visitor callers.
fn visitor_erased_bool(
    visitor: &mut dyn GameRuleTypeVisitor,
    index: usize,
) {
    let inner = game_rule_registry()
        .lock()
        .unwrap()
        .by_index(index)
        .expect("gamerule index")
        .clone();
    let r = GameRule::<bool>::from_inner(inner);
    visitor.visit_boolean(&r);
}

fn visitor_erased_int(
    visitor: &mut dyn GameRuleTypeVisitor,
    index: usize,
) {
    let inner = game_rule_registry()
        .lock()
        .unwrap()
        .by_index(index)
        .expect("gamerule index")
        .clone();
    let r = GameRule::<i32>::from_inner(inner);
    visitor.visit_integer(&r);
}

// ---------- Типизированные константы-аксессоры ----------

macro_rules! bool_rule {
    ($name:ident, $id:literal) => {
        /// Java `GameRules::$name`.
        pub fn $name() -> GameRule<bool> {
            bootstrap();
            game_rule_bool($id)
        }
    };
}
macro_rules! int_rule {
    ($name:ident, $id:literal) => {
        /// Java `GameRules::$name`.
        pub fn $name() -> GameRule<i32> {
            bootstrap();
            game_rule_int($id)
        }
    };
}

bool_rule!(advance_time, "advance_time");
bool_rule!(advance_weather, "advance_weather");
bool_rule!(allow_entering_nether_using_portals, "allow_entering_nether_using_portals");
bool_rule!(block_drops, "block_drops");
bool_rule!(block_explosion_drop_decay, "block_explosion_drop_decay");
bool_rule!(command_blocks_work, "command_blocks_work");
bool_rule!(command_block_output, "command_block_output");
bool_rule!(drowning_damage, "drowning_damage");
bool_rule!(elytra_movement_check, "elytra_movement_check");
bool_rule!(ender_pearls_vanish_on_death, "ender_pearls_vanish_on_death");
bool_rule!(entity_drops, "entity_drops");
bool_rule!(fall_damage, "fall_damage");
bool_rule!(fire_damage, "fire_damage");
bool_rule!(forgive_dead_players, "forgive_dead_players");
bool_rule!(freeze_damage, "freeze_damage");
bool_rule!(global_sound_events, "global_sound_events");
bool_rule!(immediate_respawn, "immediate_respawn");
bool_rule!(keep_inventory, "keep_inventory");
bool_rule!(lava_source_conversion, "lava_source_conversion");
bool_rule!(limited_crafting, "limited_crafting");
bool_rule!(locator_bar, "locator_bar");
bool_rule!(log_admin_commands, "log_admin_commands");
bool_rule!(mob_drops, "mob_drops");
bool_rule!(mob_explosion_drop_decay, "mob_explosion_drop_decay");
bool_rule!(mob_griefing, "mob_griefing");
bool_rule!(natural_health_regeneration, "natural_health_regeneration");
bool_rule!(player_movement_check, "player_movement_check");
bool_rule!(projectiles_can_break_blocks, "projectiles_can_break_blocks");
bool_rule!(pvp, "pvp");
bool_rule!(raids, "raids");
bool_rule!(reduced_debug_info, "reduced_debug_info");
bool_rule!(send_command_feedback, "send_command_feedback");
bool_rule!(show_advancement_messages, "show_advancement_messages");
bool_rule!(show_death_messages, "show_death_messages");
bool_rule!(spawner_blocks_work, "spawner_blocks_work");
bool_rule!(spawn_mobs, "spawn_mobs");
bool_rule!(spawn_monsters, "spawn_monsters");
bool_rule!(spawn_patrols, "spawn_patrols");
bool_rule!(spawn_phantoms, "spawn_phantoms");
bool_rule!(spawn_wandering_traders, "spawn_wandering_traders");
bool_rule!(spawn_wardens, "spawn_wardens");
bool_rule!(spectators_generate_chunks, "spectators_generate_chunks");
bool_rule!(spread_vines, "spread_vines");
bool_rule!(tnt_explodes, "tnt_explodes");
bool_rule!(tnt_explosion_drop_decay, "tnt_explosion_drop_decay");
bool_rule!(universal_anger, "universal_anger");
bool_rule!(water_source_conversion, "water_source_conversion");

int_rule!(fire_spread_radius_around_player, "fire_spread_radius_around_player");
int_rule!(max_block_modifications, "max_block_modifications");
int_rule!(max_command_forks, "max_command_forks");
int_rule!(max_command_sequence_length, "max_command_sequence_length");
int_rule!(max_entity_cramming, "max_entity_cramming");
int_rule!(max_minecart_speed, "max_minecart_speed");
int_rule!(max_snow_accumulation_height, "max_snow_accumulation_height");
int_rule!(players_nether_portal_creative_delay, "players_nether_portal_creative_delay");
int_rule!(players_nether_portal_default_delay, "players_nether_portal_default_delay");
int_rule!(players_sleeping_percentage, "players_sleeping_percentage");
int_rule!(random_tick_speed, "random_tick_speed");
int_rule!(respawn_radius, "respawn_radius");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamerules_defaults_and_set() {
        bootstrap();
        let mut g = GameRules::new(FeatureFlagSet::default_flags());
        let mob_griefing = mob_griefing();
        assert!(g.get_or_default(&mob_griefing));
        g.set(&mob_griefing, false);
        assert!(!g.get_or_default(&mob_griefing));
        // int-правило с границами
        let sleeping = players_sleeping_percentage();
        assert_eq!(g.get_or_default(&sleeping), 100);
        g.set(&sleeping, 50);
        assert_eq!(g.get_or_default(&sleeping), 50);
        assert_eq!(g.get_as_string_int(&sleeping), "50");
    }

    #[test]
    fn gamerules_nbt_roundtrip() {
        bootstrap();
        let mut g = GameRules::new(FeatureFlagSet::default_flags());
        g.set(&keep_inventory(), true);
        let tag = g.to_nbt();
        let g2 = GameRules::from_nbt(&tag, FeatureFlagSet::default_flags());
        assert!(g2.get_or_default(&keep_inventory()));
        assert_eq!(
            g2.get_or_default(&random_tick_speed()),
            g.get_or_default(&random_tick_speed())
        );
    }

    #[test]
    fn gamerule_deserialize_bounds() {
        bootstrap();
        let r = max_entity_cramming();
        assert!(r.deserialize("24").is_ok());
        assert!(r.deserialize("abc").is_err());
    }
}
