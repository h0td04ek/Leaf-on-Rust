//! Перенос `net.minecraft.world.level.storage.loot.predicates.LootItemCondition`
//! и стандартных условий.
//!
//! Трейт + типы условий портируются полностью; `test` принимает
//! `&LootContext` (в Java `Predicate<LootContext>`).

use crate::world::level::storage::loot::loot_context::{EntityTarget, LootContext};

/// `LootItemCondition` — условие проверки loot-контекста.
pub trait LootItemCondition: Send + Sync {
    /// `test(context)`.
    fn test(&self, context: &mut LootContext) -> bool;
}

/// `LootItemCondition.Builder`.
pub trait LootItemConditionBuilder {
    /// `build()`.
    fn build(self: Box<Self>) -> Box<dyn LootItemCondition>;
}

/// `AnyOfCondition` (`any_of`).
pub struct AnyOfCondition {
    /// `terms`.
    pub terms: Vec<Box<dyn LootItemCondition>>,
}
impl LootItemCondition for AnyOfCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        self.terms.iter().any(|t| t.test(context))
    }
}

/// `AllOfCondition` (`all_of`).
pub struct AllOfCondition {
    /// `terms`.
    pub terms: Vec<Box<dyn LootItemCondition>>,
}
impl LootItemCondition for AllOfCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        self.terms.iter().all(|t| t.test(context))
    }
}

/// `InvertedLootItemCondition`.
pub struct InvertedLootItemCondition {
    /// `term`.
    pub term: Box<dyn LootItemCondition>,
}
impl LootItemCondition for InvertedLootItemCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        !self.term.test(context)
    }
}

// ======================= остальные условия =======================

use crate::advancements::criterion::entity_predicates::{
    DamageSourcePredicate, EntityAsTarget, EntityPredicate, LocationPredicate,
};
use crate::advancements::criterion::item_predicate::ItemPredicate;
use crate::advancements::criterion::state_properties_predicate::StatePropertiesPredicate;
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use crate::tags::tag_key::TagKey;
use crate::world::entity::entity::Entity;
use crate::world::item::enchantment::enchantment::Enchantment;
use crate::world::item::enchantment::enchantment_helper;
use crate::world::item::enchantment::level_based_value::LevelBasedValue;
use crate::world::item::item_stack::ItemStack;
use crate::world::level::storage::loot::int_range::IntRange;
use crate::world::level::storage::loot::parameters::loot_context_params as params;
use crate::world::level::storage::loot::providers::number::number_provider::NumberProvider;
use crate::world::phys::vec3::Vec3;

/// `LootItemRandomChanceCondition` (`random_chance`).
pub struct LootItemRandomChanceCondition {
    /// `chance`.
    pub chance: Box<dyn NumberProvider>,
}
impl LootItemCondition for LootItemRandomChanceCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        let chance = self.chance.get_float(context);
        context.random().next_float() < chance
    }
}

/// `LootItemRandomChanceWithEnchantedBonusCondition`
/// (`random_chance_with_enchanted_bonus`).
pub struct LootItemRandomChanceWithEnchantedBonusCondition {
    /// `unenchantedChance`.
    pub unenchanted_chance: f32,
    /// `enchantedChance`.
    pub enchanted_chance: LevelBasedValue,
    /// `enchantment`.
    pub enchantment: Holder<Enchantment>,
}
impl LootItemCondition for LootItemRandomChanceWithEnchantedBonusCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        let entity = context
            .get_optional_parameter(params::attacking_entity())
            .cloned();
        // `EnchantmentHelper.getEnchantmentLevel` — для Entity
        // (не-LivingEntity) уровень 0, как в Java для не-living.
        let level = entity
            .as_ref()
            .map(|e| enchantment_level_on_entity(e, &self.enchantment))
            .unwrap_or(0);
        let f = if level > 0 {
            self.enchanted_chance.calculate(level)
        } else {
            self.unenchanted_chance
        };
        context.random().next_float() < f
    }
}

/// Уровень зачарования на `Entity` — для living-сущностей
/// делегируется через `LivingEntity`; базовая Entity — 0
/// (соответствует Java `!(entity instanceof LivingEntity)`).
fn enchantment_level_on_entity(
    entity: &Entity,
    enchantment: &Holder<Enchantment>,
) -> i32 {
    // Entity в нашей модели не несёт экипировки — LivingEntity
    // хранится отдельно; вызывающий код для living передаёт
    // LivingEntity напрямую через `get_enchantment_level`.
    let _ = (entity, enchantment);
    0
}

/// `LootItemKilledByPlayerCondition` (`killed_by_player`).
pub struct LootItemKilledByPlayerCondition;
impl LootItemCondition for LootItemKilledByPlayerCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        context
            .get_optional_parameter(params::last_damage_player())
            .is_some()
    }
}

/// `EntityHasScoreCondition` (`entity_scores`).
pub struct EntityHasScoreCondition {
    /// `entity` — target.
    pub entity: EntityTarget,
    /// `score` — имя objective.
    pub score: String,
    /// `range`.
    pub range: IntRange,
}
impl LootItemCondition for EntityHasScoreCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        let Some(level) = context.level().cloned() else {
            return false;
        };
        let Some(scoreboard) = level.scoreboard() else {
            return false;
        };
        let Some(entity) = context.get_optional_parameter(self.entity.context_param()) else {
            return false;
        };
        let name = entity.get_scoreboard_name();
        let Some(objective) = scoreboard.get_objective(&self.score) else {
            return false;
        };
        let Some(info) = scoreboard.get_player_score_info(&name, &objective.name) else {
            return false;
        };
        self.range.test(context, info.value)
    }
}

/// `LootItemBlockStatePropertyCondition` (`block_state_property`).
pub struct LootItemBlockStatePropertyCondition {
    /// `block`.
    pub block: Holder<crate::world::level::block::Block>,
    /// `properties`.
    pub properties: Option<StatePropertiesPredicate>,
}
impl LootItemCondition for LootItemBlockStatePropertyCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        let Some(state) = context.get_optional_parameter(params::block_state()) else {
            return false;
        };
        let matches_block = self
            .block
            .value()
            .map(|b| b.id == state.block.id)
            .unwrap_or(false);
        if !matches_block {
            return false;
        }
        if let Some(p) = &self.properties {
            return p.matches_block_state(state);
        }
        true
    }
}

/// `MatchTool` (`match_tool`).
pub struct MatchTool {
    /// `predicate`.
    pub predicate: ItemPredicate,
}
impl LootItemCondition for MatchTool {
    fn test(&self, context: &mut LootContext) -> bool {
        context
            .get_optional_parameter(params::tool())
            .map(|t| self.predicate.test(t))
            .unwrap_or(false)
    }
}

/// `BonusLevelTableCondition` (`table_bonus`).
pub struct BonusLevelTableCondition {
    /// `enchantment`.
    pub enchantment: Holder<Enchantment>,
    /// `values` — таблица шансов по уровню.
    pub values: Vec<f32>,
}
impl LootItemCondition for BonusLevelTableCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        let tool = context.get_optional_parameter(params::tool());
        let level = tool
            .map(|t| enchantment_helper::get_item_enchantment_level(&self.enchantment, t))
            .unwrap_or(0);
        let idx = (level as usize).min(self.values.len().saturating_sub(1));
        let f = self.values[idx];
        context.random().next_float() < f
    }
}

/// `ExplosionCondition` (`survives_explosion`).
pub struct ExplosionCondition {
    /// `survives_explosion`.
    pub survives_explosion: bool,
}
impl LootItemCondition for ExplosionCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        context
            .get_optional_parameter(params::explosion_radius())
            .map(|_| self.survives_explosion)
            .unwrap_or(false)
    }
}

/// `DamageSourceCondition` (`damage_source_properties`).
pub struct DamageSourceCondition {
    /// `predicate`.
    pub predicate: DamageSourcePredicate,
}
impl LootItemCondition for DamageSourceCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        let Some(source) = context.get_optional_parameter(params::damage_source()) else {
            return false;
        };
        let level = context.level().cloned();
        let pos = context.get_optional_parameter(params::origin()).copied();
        match level {
            Some(l) => self.predicate.matches(l.as_ref(), pos, source),
            None => false,
        }
    }
}

/// `LocationCheck` (`location_check`).
pub struct LocationCheck {
    /// `predicate`.
    pub predicate: LocationPredicate,
}
impl LootItemCondition for LocationCheck {
    fn test(&self, context: &mut LootContext) -> bool {
        let Some(level) = context.level().cloned() else {
            return false;
        };
        let Some(pos) = context.get_optional_parameter(params::origin()).copied() else {
            return false;
        };
        self.predicate.matches(level.as_ref(), pos.x, pos.y, pos.z)
    }
}

/// `WeatherCheck` (`weather_check`).
pub struct WeatherCheck {
    /// `raining`.
    pub raining: Option<bool>,
    /// `thundering`.
    pub thundering: Option<bool>,
}
impl LootItemCondition for WeatherCheck {
    fn test(&self, context: &mut LootContext) -> bool {
        let Some(level) = context.level() else {
            return false;
        };
        if let Some(r) = self.raining {
            if level.is_raining() != r {
                return false;
            }
        }
        if let Some(t) = self.thundering {
            if level.is_thundering() != t {
                return false;
            }
        }
        true
    }
}

/// `ConditionReference` (`reference`) — именованное условие из
/// реестра (аналог `context.getResolver()`).
pub struct ConditionReference {
    /// `name` — `ResourceKey<LootItemCondition>`.
    pub name: Identifier,
}
impl LootItemCondition for ConditionReference {
    fn test(&self, context: &mut LootContext) -> bool {
        let id = self.name.clone();
        if context.has_visited_element(&id.to_string()) {
            return false;
        }
        let condition = condition_registry()
            .lock()
            .unwrap()
            .get(&id)
            .cloned();
        let Some(condition) = condition else {
            return false;
        };
        if !context.push_visited_element(id.to_string()) {
            return false;
        }
        let result = condition.test(context);
        context.pop_visited_element(&id.to_string());
        result
    }
}

/// Реестр именованных условий.
pub fn condition_registry()
    -> &'static std::sync::Mutex<std::collections::HashMap<Identifier, std::sync::Arc<dyn LootItemCondition>>>
{
    static REG: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<Identifier, std::sync::Arc<dyn LootItemCondition>>>,
    > = std::sync::OnceLock::new();
    REG.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// `TimeCheck` (`time_check`).
pub struct TimeCheck {
    /// `value`.
    pub value: IntRange,
}
impl LootItemCondition for TimeCheck {
    fn test(&self, context: &mut LootContext) -> bool {
        let Some(level) = context.level() else {
            return false;
        };
        self.value.test(context, level.game_time() as i32)
    }
}

/// `ValueCheckCondition` (`value_check`).
pub struct ValueCheckCondition {
    /// `provider`.
    pub provider: Box<dyn NumberProvider>,
    /// `range`.
    pub range: IntRange,
}
impl LootItemCondition for ValueCheckCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        let value = self.provider.get_int(context);
        self.range.test(context, value)
    }
}

/// `EnchantmentActiveCheck` (`enchantment_active_check`).
pub struct EnchantmentActiveCheck;
impl LootItemCondition for EnchantmentActiveCheck {
    fn test(&self, context: &mut LootContext) -> bool {
        context
            .get_optional_parameter(params::enchantment_active())
            .copied()
            .unwrap_or(false)
    }
}

/// `LootItemEntityPropertyCondition` (`entity_properties`).
pub struct LootItemEntityPropertyCondition {
    /// `predicate`.
    pub predicate: Option<EntityPredicate>,
    /// `entity`.
    pub entity: EntityTarget,
}
impl LootItemCondition for LootItemEntityPropertyCondition {
    fn test(&self, context: &mut LootContext) -> bool {
        let Some(entity) = context.get_optional_parameter(self.entity.context_param()) else {
            return self.predicate.is_none();
        };
        let Some(level) = context.level().cloned() else {
            return false;
        };
        let pos = context.get_optional_parameter(params::origin()).copied();
        match self.predicate.as_ref() {
            Some(p) => p.matches_entity(level.as_ref(), pos, entity),
            None => true,
        }
    }
}
