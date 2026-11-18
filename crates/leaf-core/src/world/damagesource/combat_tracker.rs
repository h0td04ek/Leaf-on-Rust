//! Перенос `net.minecraft.world.damagesource.CombatTracker` — трекер боевых записей.

use crate::network::chat::component::Component;
use crate::network::chat::component_contents::TranslatableArg;
use crate::world::damagesource::combat_entry::CombatEntry;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::damagesource::death_message_type::DeathMessageType;
use crate::world::damagesource::fall_location::FallLocation;
use crate::world::entity::entity::Entity;

/// `CombatTracker` — трекер урона сущности (entries, inCombat, death message).
pub struct CombatTracker {
    /// `entries` — записи урона.
    pub entries: Vec<CombatEntry>,
    /// `mob` — сущность-владелец.
    pub mob: Entity,
    /// `lastDamageTime` — последний тик урона.
    pub last_damage_time: i32,
    /// `combatStartTime` — начало боя.
    pub combat_start_time: i32,
    /// `combatEndTime` — конец боя.
    pub combat_end_time: i32,
    /// `inCombat` — в бою.
    pub in_combat: bool,
    /// `takingDamage` — получает урон.
    pub taking_damage: bool,
}

impl CombatTracker {
    /// `RESET_DAMAGE_STATUS_TIME`.
    pub const RESET_DAMAGE_STATUS_TIME: i32 = 100;
    /// `RESET_COMBAT_STATUS_TIME`.
    pub const RESET_COMBAT_STATUS_TIME: i32 = 300;

    /// `new(mob)`.
    pub fn new(mob: Entity) -> Self {
        Self {
            entries: Vec::new(),
            mob,
            last_damage_time: 0,
            combat_start_time: 0,
            combat_end_time: 0,
            in_combat: false,
            taking_damage: false,
        }
    }

    /// `recordDamage(source, damage)` — записать урон (с fallLocation из сущности).
    pub fn record_damage(&mut self, source: DamageSource, damage: f32) {
        let fall_location = self.mob_fall_location();
        let fall_distance = self.mob.fall_distance;
        let entry = CombatEntry::new(source, damage, fall_location, fall_distance);
        self.record_damage_and_check(entry);
    }

    /// Текущая локация падения сущности — `None` (базовая Entity не отслеживает).
    fn mob_fall_location(&self) -> Option<FallLocation> {
        None
    }

    /// `recordDamageAndCheckCombatState(combatEntry)`.
    pub fn record_damage_and_check(&mut self, combat_entry: CombatEntry) {
        let source = combat_entry.source.clone();
        self.entries.push(combat_entry);
        self.last_damage_time = self.mob.tick_count;
        self.taking_damage = true;
        if !self.in_combat && self.mob.is_alive() && Self::should_enter_combat(&source) {
            self.in_combat = true;
            self.combat_start_time = self.mob.tick_count;
            self.combat_end_time = self.combat_start_time;
            self.mob_on_enter_combat();
        }
    }

    /// `mob.onEnterCombat()` — hook.
    fn mob_on_enter_combat(&mut self) {
        // LivingEntity переопределяет; базовая реализация — no-op.
    }

    /// `shouldEnterCombat(source)` — источник от LivingEntity.
    fn should_enter_combat(source: &DamageSource) -> bool {
        source.get_causing_entity().map(|e| e.is_living_entity()).unwrap_or(false)
    }

    /// `getDeathMessage()`.
    pub fn get_death_message(&self) -> Component {
        if self.entries.is_empty() {
            return Component::translatable(
                "death.attack.generic",
                vec![TranslatableArg::Component(Box::new(self.mob.get_display_name()))],
            );
        }
        let combat_entry = self.entries.last().unwrap();
        let damage_source = &combat_entry.source;
        let most_significant_fall = self.get_most_significant_fall();
        let death_message_type = damage_source
            .get_type()
            .value()
            .map(|dt| dt.death_message_type)
            .unwrap_or(DeathMessageType::Default);

        if death_message_type == DeathMessageType::FallVariants && most_significant_fall.is_some() {
            return self.get_fall_message(most_significant_fall.as_ref().unwrap(), damage_source.get_causing_entity());
        }
        if death_message_type == DeathMessageType::IntentionalGameDesign {
            let msg_id = damage_source
                .get_type()
                .value()
                .map(|dt| dt.msg_id.clone())
                .unwrap_or_else(|| "generic".to_string());
            let string = format!("death.attack.{}", msg_id);
            let component = Component::translatable(
                format!("{}.link", string),
                vec![],
            );
            return Component::translatable(
                format!("{}.message", string),
                vec![
                    TranslatableArg::Component(Box::new(self.mob.get_display_name())),
                    TranslatableArg::Component(Box::new(component)),
                ],
            );
        }
        damage_source.get_localized_death_message(&self.mob)
    }

    /// `getMostSignificantFall()`.
    pub fn get_most_significant_fall(&self) -> Option<CombatEntry> {
        let mut combat_entry: Option<CombatEntry> = None;
        let mut combat_entry1: Option<CombatEntry> = None;
        let mut f = 0.0f32;
        let mut f1 = 0.0f32;
        for i in 0..self.entries.len() {
            let entry = &self.entries[i];
            let prev = if i > 0 { Some(&self.entries[i - 1]) } else { None };
            let is_fall = entry.source.is_fall();
            let is_always = entry.source.is_always_most_significant_fall();
            let f2 = if is_always { f32::MAX } else { entry.fall_distance };
            if (is_fall || is_always) && f2 > 0.0 && (combat_entry.is_none() || f2 > f1) {
                combat_entry = if i > 0 { prev.cloned() } else { Some(entry.clone()) };
                f1 = f2;
            }
            if entry.fall_location.is_some() && (combat_entry1.is_none() || entry.damage > f) {
                combat_entry1 = Some(entry.clone());
                f = entry.damage;
            }
        }
        combat_entry1.or(combat_entry)
    }

    /// `getFallMessage(combatEntry, entity)`.
    fn get_fall_message(
        &self,
        combat_entry: &CombatEntry,
        entity: Option<&Entity>,
    ) -> Component {
        let damage_source = &combat_entry.source;
        if !damage_source.is_fall() && !damage_source.is_always_most_significant_fall() {
            let display_name = Self::get_display_name(entity);
            let entity1 = damage_source.get_causing_entity();
            let display_name1 = Self::get_display_name(entity1);
            if display_name1.is_some() && display_name1 != display_name {
                return self.get_message_for_assisted_fall(
                    entity1.unwrap(),
                    display_name1.unwrap(),
                    "death.fell.assist.item",
                    "death.fell.assist",
                );
            } else {
                return match display_name {
                    Some(name) => self.get_message_for_assisted_fall(
                        entity.unwrap(),
                        name,
                        "death.fell.finish.item",
                        "death.fell.finish",
                    ),
                    None => Component::translatable(
                        "death.fell.killer",
                        vec![TranslatableArg::Component(Box::new(self.mob.get_display_name()))],
                    ),
                };
            }
        } else {
            let fall_location = combat_entry.fall_location.unwrap_or(FallLocation::Generic);
            Component::translatable(
                fall_location.language_key(),
                vec![TranslatableArg::Component(Box::new(self.mob.get_display_name()))],
            )
        }
    }

    /// `getMessageForAssistedFall(entity, entityDisplayName, hasWeaponKey, noWeaponKey)`.
    fn get_message_for_assisted_fall(
        &self,
        entity: &Entity,
        entity_display_name: Component,
        has_weapon_key: &str,
        no_weapon_key: &str,
    ) -> Component {
        // Упрощённо: ItemStack.EMPTY-проверка — без инвентаря.
        let has_weapon = entity.get_main_hand_item().is_some();
        if has_weapon {
            let item_name = entity.get_main_hand_item().unwrap().get_hover_name();
            Component::translatable(
                has_weapon_key,
                vec![
                    TranslatableArg::Component(Box::new(self.mob.get_display_name())),
                    TranslatableArg::Component(Box::new(entity_display_name)),
                    TranslatableArg::Component(Box::new(item_name)),
                ],
            )
        } else {
            Component::translatable(
                no_weapon_key,
                vec![
                    TranslatableArg::Component(Box::new(self.mob.get_display_name())),
                    TranslatableArg::Component(Box::new(entity_display_name)),
                ],
            )
        }
    }

    /// `getDisplayName(entity)`.
    fn get_display_name(entity: Option<&Entity>) -> Option<Component> {
        entity.map(|e| e.get_display_name())
    }

    /// `getCombatStatus()`.
    pub fn get_combat_status(&self) -> i32 {
        if self.in_combat {
            let combat_ticks = self.mob.tick_count - self.combat_start_time;
            if combat_ticks < Self::RESET_COMBAT_STATUS_TIME {
                return combat_ticks;
            }
        }
        0
    }

    /// `getTicksSinceDamage()` — тики с последнего урона.
    pub fn get_ticks_since_damage(&self) -> i32 {
        self.mob.tick_count - self.last_damage_time
    }

    /// `resetDamageStatus()`.
    pub fn reset_damage_status(&mut self) {
        self.last_damage_time = self.mob.tick_count - Self::RESET_DAMAGE_STATUS_TIME - 1;
    }

    /// `recheckStatus()`.
    pub fn recheck_status(&mut self) {
        let i = self.mob.tick_count - Self::RESET_COMBAT_STATUS_TIME;
        if self.taking_damage && self.last_damage_time <= i {
            self.taking_damage = false;
        }
        if (self.in_combat && this_combat_expired(i)) || (!this_in_combat_expired(i) && self.in_combat) {
            // Обновление inCombat в tick
        }
    }

    /// `tick()` — вызывается из LivingEntity.tick.
    pub fn tick(&mut self) {
        self.recheck_status();
        let i = self.mob.tick_count - Self::RESET_COMBAT_STATUS_TIME;
        if (this_in_combat_expired(i) && self.in_combat) || (!this_in_combat_expired(i) && !self.in_combat) {
            // Смена состояния
        }
        if this_in_combat_expired(i) && self.in_combat {
            self.in_combat = false;
            self.combat_end_time = self.mob.tick_count;
            self.mob_on_exit_combat();
        }
        if !this_in_combat_expired(i) && !self.in_combat && self.mob.is_alive() {
            // Проверка входа в бой происходит в recordDamage
        }
    }

    /// `mob.onLeaveCombat()` — hook.
    fn mob_on_exit_combat(&mut self) {
        // LivingEntity переопределяет.
    }
}

fn this_in_combat_expired(i: i32) -> bool {
    i > 0
}

fn this_combat_expired(i: i32) -> bool {
    i < 0
}

impl Clone for CombatTracker {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            mob: self.mob.clone(),
            last_damage_time: self.last_damage_time,
            combat_start_time: self.combat_start_time,
            combat_end_time: self.combat_end_time,
            in_combat: self.in_combat,
            taking_damage: self.taking_damage,
        }
    }
}
