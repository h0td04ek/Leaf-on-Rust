//! Перенос `net.minecraft.world.entity.ai.goal.MeleeAttackGoal` и
//! `ZombieAttackGoal` — ближний бой: преследование цели,
//! пересчёт пути, кулдаун атаки, замах и `doHurtTarget`.

use crate::core::block_pos::BlockPos;
use crate::entity::humanoid_arm::HumanoidArm;
use crate::world::entity::ai::goal::goal::{Goal, GoalFlag, GoalFlagSet};
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::level::pathfinder::path::Path;

/// `EntitySelector.NO_CREATIVE_OR_SPECTATOR` — false только для
/// игрока в spectator/creative.
pub(crate) fn no_creative_or_spectator(target: Option<&LivingEntity>) -> bool {
    match target {
        Some(t) => {
            !(t.entity.is_player()
                && (t.entity.is_spectator() || t.entity.is_creative()))
        }
        None => true,
    }
}

/// `MeleeAttackGoal` — атака в ближнем бою.
pub struct MeleeAttackGoal {
    speed_modifier: f64,
    following_target_even_if_not_seen: bool,
    path: Option<Path>,
    pathed_target_x: f64,
    pathed_target_y: f64,
    pathed_target_z: f64,
    ticks_until_next_path_recalculation: i32,
    ticks_until_next_attack: i32,
    last_can_use_check: i64,
}

impl MeleeAttackGoal {
    /// `new(mob, speedModifier, followingTargetEvenIfNotSeen)`.
    pub fn new(_mob: &Mob, speed_modifier: f64, following_target_even_if_not_seen: bool) -> Self {
        Self {
            speed_modifier,
            following_target_even_if_not_seen,
            path: None,
            pathed_target_x: 0.0,
            pathed_target_y: 0.0,
            pathed_target_z: 0.0,
            ticks_until_next_path_recalculation: 0,
            ticks_until_next_attack: 0,
            last_can_use_check: 0,
        }
    }

    /// `checkAndPerformAttack(target)`.
    pub fn check_and_perform_attack(&mut self, mob: &mut Mob, target: &LivingEntity) {
        if self.can_perform_attack(mob, target) {
            self.reset_attack_cooldown();
            let arm = mob.living.get_main_arm();
            mob.living.swing(arm);
            let mut t = target.clone();
            mob.do_hurt_target(&mut t);
        }
    }

    /// `resetAttackCooldown()`.
    pub fn reset_attack_cooldown(&mut self) {
        self.ticks_until_next_attack = self.adjusted_tick_delay(20);
    }

    /// `isTimeToAttack()`.
    pub fn is_time_to_attack(&self) -> bool {
        self.ticks_until_next_attack <= 0
    }

    /// `canPerformAttack(entity)`.
    pub fn can_perform_attack(&self, mob: &mut Mob, entity: &LivingEntity) -> bool {
        self.is_time_to_attack()
            && mob.is_within_melee_attack_range(entity)
            && mob.has_line_of_sight(&entity.entity)
    }

    /// `getTicksUntilNextAttack()`.
    pub fn get_ticks_until_next_attack(&self) -> i32 {
        self.ticks_until_next_attack
    }

    /// `getAttackInterval()`.
    pub fn get_attack_interval(&self) -> i32 {
        self.adjusted_tick_delay(20)
    }

    /// `mob.getSensing().hasLineOfSight(target)` — с учётом кэша
    /// Sensing; `mob.has_line_of_sight` уже делает mem::take.
    fn has_los(&mut self, mob: &mut Mob, target: &LivingEntity) -> bool {
        mob.has_line_of_sight(&target.entity)
    }
}

impl Goal for MeleeAttackGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let game_time = mob.level().map(|l| l.game_time()).unwrap_or(0);
        if game_time - self.last_can_use_check < 20 {
            return false;
        }
        self.last_can_use_check = game_time;

        let Some(target) = mob.get_target() else {
            return false;
        };
        if !target.is_alive() {
            return false;
        }

        let path = mob.get_navigation().and_then(|nav| {
            nav.borrow_mut().create_path_entity(mob, target, 0)
        });
        self.path = path;
        self.path.is_some() || mob.is_within_melee_attack_range(target)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(target) = mob.get_target() else {
            return false;
        };
        if !target.is_alive() {
            return false;
        }
        if !self.following_target_even_if_not_seen {
            return mob.navigation_is_done() == false;
        }
        mob.is_within_home(target.block_position()) && no_creative_or_spectator(Some(target))
    }

    fn start(&mut self, mob: &mut Mob) {
        let nav = mob.get_navigation();
        if let Some(nav) = nav {
            nav.borrow_mut().move_to_path(mob, self.path.take(), self.speed_modifier);
        }
        mob.set_aggressive(true);
        self.ticks_until_next_path_recalculation = 0;
        self.ticks_until_next_attack = 0;
    }

    fn stop(&mut self, mob: &mut Mob) {
        if !no_creative_or_spectator(mob.get_target()) {
            mob.set_target(None);
        }
        mob.set_aggressive(false);
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().stop(mob);
        }
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(target) = mob.get_target().cloned() else {
            return;
        };
        mob.look_control
            .set_look_at_entity_deltas(&target.entity, 30.0, 30.0);

        self.ticks_until_next_path_recalculation =
            (self.ticks_until_next_path_recalculation - 1).max(0);

        if (self.following_target_even_if_not_seen || self.has_los(mob, &target))
            && self.ticks_until_next_path_recalculation <= 0
            && (self.pathed_target_x == 0.0
                && self.pathed_target_y == 0.0
                && self.pathed_target_z == 0.0
                || target
                    .entity
                    .distance_to_sqr_pos(
                        self.pathed_target_x,
                        self.pathed_target_y,
                        self.pathed_target_z,
                    )
                    >= 1.0
                || mob.random.borrow_mut().next_float() < 0.05)
        {
            self.pathed_target_x = target.entity.get_x();
            self.pathed_target_y = target.entity.get_y();
            self.pathed_target_z = target.entity.get_z();
            self.ticks_until_next_path_recalculation =
                4 + mob.random.borrow_mut().next_int_bound(7);

            let d = target.entity.distance_to_sqr(&mob.living.entity);
            if d > 1024.0 {
                self.ticks_until_next_path_recalculation += 10;
            } else if d > 256.0 {
                self.ticks_until_next_path_recalculation += 5;
            }

            let moved = mob.get_navigation().map(|nav| {
                nav.borrow_mut().move_to_entity(mob, &target, self.speed_modifier)
            });
            if moved != Some(true) {
                self.ticks_until_next_path_recalculation += 15;
            }

            self.ticks_until_next_path_recalculation =
                self.adjusted_tick_delay(self.ticks_until_next_path_recalculation);
        }

        self.ticks_until_next_attack = (self.ticks_until_next_attack - 1).max(0);
        self.check_and_perform_attack(mob, &target);
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move, GoalFlag::Look].into_iter().collect()
    }
}

/// `ZombieAttackGoal` — зомби: после 5 тиков и при готовности атаки
/// выставляет `aggressive` (анимация поднятия рук). В Java
/// наследует `MeleeAttackGoal` — здесь композиция.
pub struct ZombieAttackGoal {
    melee: MeleeAttackGoal,
    raise_arm_ticks: i32,
}

impl ZombieAttackGoal {
    /// `new(zombie, speedModifier, followingTargetEvenIfNotSeen)`.
    pub fn new(mob: &Mob, speed_modifier: f64, following_target_even_if_not_seen: bool) -> Self {
        Self {
            melee: MeleeAttackGoal::new(mob, speed_modifier, following_target_even_if_not_seen),
            raise_arm_ticks: 0,
        }
    }

    /// `getTicksUntilNextAttack()` — делегат для проверки
    /// `raiseArmTicks >= 5 && ticksUntilNextAttack < attackInterval / 2`.
    pub fn get_ticks_until_next_attack(&self) -> i32 {
        self.melee.get_ticks_until_next_attack()
    }

    /// `getAttackInterval()` — делегат.
    pub fn get_attack_interval(&self) -> i32 {
        self.melee.get_attack_interval()
    }
}

impl Goal for ZombieAttackGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        self.melee.can_use(mob)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.melee.can_continue_to_use(mob)
    }

    fn start(&mut self, mob: &mut Mob) {
        self.melee.start(mob);
        self.raise_arm_ticks = 0;
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.melee.stop(mob);
        mob.set_aggressive(false);
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        self.melee.tick(mob);
        self.raise_arm_ticks += 1;
        if self.raise_arm_ticks >= 5
            && self.get_ticks_until_next_attack() < self.get_attack_interval() / 2
        {
            mob.set_aggressive(true);
        } else {
            mob.set_aggressive(false);
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move, GoalFlag::Look].into_iter().collect()
    }
}

/// `RangedAttackMob.performRangedAttack(target, distanceFactor)` —
/// в Java это интерфейс на мобе; здесь — callback, который
/// конкретный моб подставляет при создании цели.
pub type PerformRangedAttack =
    Box<dyn FnMut(&mut Mob, &mut LivingEntity, f32)>;

/// `RangedAttackGoal` — дальняя атака (скелеты, пиллагеры и т.п.).
pub struct RangedAttackGoal {
    speed_modifier: f64,
    attack_interval_min: i32,
    attack_interval_max: i32,
    attack_radius: f32,
    attack_radius_sqr: f32,
    target: Option<LivingEntity>,
    attack_time: i32,
    see_time: i32,
    perform_ranged_attack: PerformRangedAttack,
}

impl RangedAttackGoal {
    /// `new(rangedAttackMob, speedModifier, attackInterval, attackRadius)`.
    pub fn new(
        _mob: &Mob,
        speed_modifier: f64,
        attack_interval: i32,
        attack_radius: f32,
        perform_ranged_attack: PerformRangedAttack,
    ) -> Self {
        Self::new_range(
            _mob,
            speed_modifier,
            attack_interval,
            attack_interval,
            attack_radius,
            perform_ranged_attack,
        )
    }

    /// `new(rangedAttackMob, speedModifier, attackIntervalMin,
    /// attackIntervalMax, attackRadius)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new_range(
        _mob: &Mob,
        speed_modifier: f64,
        attack_interval_min: i32,
        attack_interval_max: i32,
        attack_radius: f32,
        perform_ranged_attack: PerformRangedAttack,
    ) -> Self {
        Self {
            speed_modifier,
            attack_interval_min,
            attack_interval_max,
            attack_radius,
            attack_radius_sqr: attack_radius * attack_radius,
            target: None,
            attack_time: -1,
            see_time: 0,
            perform_ranged_attack,
        }
    }
}

impl Goal for RangedAttackGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        match mob.get_target() {
            Some(t) if t.is_alive() => {
                self.target = Some(t.clone());
                true
            }
            _ => false,
        }
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        if self.can_use(mob) {
            return true;
        }
        match &self.target {
            Some(t) => t.is_alive() && !mob.navigation_is_done(),
            None => false,
        }
    }

    fn stop(&mut self, _mob: &mut Mob) {
        self.target = None;
        self.see_time = 0;
        self.attack_time = -1;
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(mut target) = self.target.clone() else {
            return;
        };
        let d = mob.living.entity.distance_to_sqr_pos(
            target.entity.get_x(),
            target.entity.get_y(),
            target.entity.get_z(),
        );
        let has_line_of_sight = mob.has_line_of_sight(&target.entity);
        if has_line_of_sight {
            self.see_time += 1;
        } else {
            self.see_time = 0;
        }

        if !(d > self.attack_radius_sqr as f64) && self.see_time >= 5 {
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().stop(mob);
            }
        } else if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().move_to_entity(mob, &target, self.speed_modifier);
        }

        mob.look_control
            .set_look_at_entity_deltas(&target.entity, 30.0, 30.0);

        self.attack_time -= 1;
        if self.attack_time == 0 {
            if !has_line_of_sight {
                return;
            }
            let f = (d.sqrt() / self.attack_radius as f64) as f32;
            let f1 = f.clamp(0.1, 1.0);
            (self.perform_ranged_attack)(mob, &mut target, f1);
            self.attack_time = crate::util::mth::floor_f32(
                f * (self.attack_interval_max - self.attack_interval_min) as f32
                    + self.attack_interval_min as f32,
            );
        } else if self.attack_time < 0 {
            let delta = (d.sqrt() / self.attack_radius as f64) as f32;
            self.attack_time = crate::util::mth::floor_f32(crate::util::mth::lerp_f32(
                delta,
                self.attack_interval_min as f32,
                self.attack_interval_max as f32,
            ));
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move, GoalFlag::Look].into_iter().collect()
    }
}

/// `RangedBowAttackGoal` — скелет с луком: strafing-манёвр,
/// натягивание лука, выстрел по `getPowerForTime`.
pub struct RangedBowAttackGoal {
    speed_modifier: f64,
    attack_interval_min: i32,
    attack_radius_sqr: f32,
    attack_time: i32,
    see_time: i32,
    strafing_clockwise: bool,
    strafing_backwards: bool,
    strafing_time: i32,
    perform_ranged_attack: PerformRangedAttack,
}

impl RangedBowAttackGoal {
    /// `new(mob, speedModifier, attackIntervalMin, attackRadius)`.
    pub fn new(
        mob: &Mob,
        speed_modifier: f64,
        attack_interval_min: i32,
        attack_radius: f32,
        perform_ranged_attack: PerformRangedAttack,
    ) -> Self {
        Self {
            speed_modifier,
            attack_interval_min,
            attack_radius_sqr: attack_radius * attack_radius,
            attack_time: -1,
            see_time: 0,
            strafing_clockwise: false,
            strafing_backwards: false,
            strafing_time: -1,
            perform_ranged_attack,
        }
    }

    /// `setMinAttackInterval(attackCooldown)`.
    pub fn set_min_attack_interval(&mut self, attack_cooldown: i32) {
        self.attack_interval_min = attack_cooldown;
    }

    /// `isHoldingBow()`.
    fn is_holding_bow(&self, mob: &Mob) -> bool {
        mob.living
            .is_holding_item(&crate::resources::identifier::Identifier::minecraft("bow"))
    }
}

impl Goal for RangedBowAttackGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        mob.get_target().is_some() && self.is_holding_bow(mob)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        (self.can_use(mob) || !mob.navigation_is_done()) && self.is_holding_bow(mob)
    }

    fn start(&mut self, mob: &mut Mob) {
        mob.set_aggressive(true);
    }

    fn stop(&mut self, mob: &mut Mob) {
        mob.set_aggressive(false);
        self.see_time = 0;
        self.attack_time = -1;
        mob.living.stop_using_item();
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(target) = mob.get_target().cloned() else {
            return;
        };
        let d = mob.living.entity.distance_to_sqr_pos(
            target.entity.get_x(),
            target.entity.get_y(),
            target.entity.get_z(),
        );
        let has_line_of_sight = mob.has_line_of_sight(&target.entity);
        let flag = self.see_time > 0;
        if has_line_of_sight != flag {
            self.see_time = 0;
        }
        if has_line_of_sight {
            self.see_time += 1;
        } else {
            self.see_time -= 1;
        }

        if !(d > self.attack_radius_sqr as f64) && self.see_time >= 20 {
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().stop(mob);
            }
            self.strafing_time += 1;
        } else {
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().move_to_entity(mob, &target, self.speed_modifier);
            }
            self.strafing_time = -1;
        }

        if self.strafing_time >= 20 {
            if mob.random.borrow_mut().next_float() < 0.3 {
                self.strafing_clockwise = !self.strafing_clockwise;
            }
            if mob.random.borrow_mut().next_float() < 0.3 {
                self.strafing_backwards = !self.strafing_backwards;
            }
            self.strafing_time = 0;
        }

        // Gale fix MC-121706: в Leaf `galeConfig` управляет
        // дополнительным lookAt; сохраняем vanilla-ветку —
        // lookAt цели делается всегда (как при включённом фиксе).
        let has_strafing_time = self.strafing_time > -1;

        if has_strafing_time {
            if d > self.attack_radius_sqr as f64 * 0.75 {
                self.strafing_backwards = false;
            } else if d < self.attack_radius_sqr as f64 * 0.25 {
                self.strafing_backwards = true;
            }
            mob.move_control.strafe(
                if self.strafing_backwards { -0.5 } else { 0.5 },
                if self.strafing_clockwise { 0.5 } else { -0.5 },
            );
            if let Some(_vehicle) = mob.living.entity.get_controlled_vehicle() {
                // `instanceof Mob` → lookAt транспортом; ветка
                // станет активной при порте Boat/Minecart.
            }
            mob.look_at(&target.entity, 30.0, 30.0);
        }
        mob.look_control
            .set_look_at_entity_deltas(&target.entity, 30.0, 30.0);

        if mob.living.is_using_item() {
            if !has_line_of_sight && self.see_time < -60 {
                mob.living.stop_using_item();
            } else if has_line_of_sight {
                let ticks_using_item = mob.living.get_ticks_using_item();
                if ticks_using_item >= 20 {
                    mob.living.stop_using_item();
                    let power =
                        crate::world::item::bow_item::get_power_for_time(ticks_using_item);
                    let mut t = target.clone();
                    (self.perform_ranged_attack)(mob, &mut t, power);
                    self.attack_time = self.attack_interval_min;
                }
            }
        } else {
            self.attack_time -= 1;
            if self.attack_time <= 0 && self.see_time >= -60 {
                let bow = crate::resources::identifier::Identifier::minecraft("bow");
                let hand =
                    crate::world::entity::projectile::projectile_util::get_weapon_holding_hand(
                        &mob.living,
                        &bow,
                    );
                let arm = crate::world::entity::projectile::projectile_util::hand_to_arm(
                    &mob.living,
                    hand,
                );
                mob.living.start_using_item(arm);
            }
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move, GoalFlag::Look].into_iter().collect()
    }
}

/// `CrossbowState` — состояния перезарядки арбалета.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossbowState {
    /// `UNCHARGED`.
    Uncharged,
    /// `CHARGING`.
    Charging,
    /// `CHARGED`.
    Charged,
    /// `READY_TO_ATTACK`.
    ReadyToAttack,
}

/// `RangedCrossbowAttackGoal` — пиллагер/разбойник с арбалетом.
pub struct RangedCrossbowAttackGoal {
    speed_modifier: f64,
    attack_radius_sqr: f32,
    crossbow_state: CrossbowState,
    see_time: i32,
    attack_delay: i32,
    update_path_delay: i32,
    perform_ranged_attack: PerformRangedAttack,
}

impl RangedCrossbowAttackGoal {
    /// `new(mob, speedModifier, attackRadius)`.
    pub fn new(
        _mob: &Mob,
        speed_modifier: f64,
        attack_radius: f32,
        perform_ranged_attack: PerformRangedAttack,
    ) -> Self {
        Self {
            speed_modifier,
            attack_radius_sqr: attack_radius * attack_radius,
            crossbow_state: CrossbowState::Uncharged,
            see_time: 0,
            attack_delay: 0,
            update_path_delay: 0,
            perform_ranged_attack,
        }
    }

    /// `isValidTarget()`.
    fn is_valid_target(&self, mob: &Mob) -> bool {
        mob.get_target().map(|t| t.is_alive()).unwrap_or(false)
    }

    /// `isHoldingCrossbow()`.
    fn is_holding_crossbow(&self, mob: &Mob) -> bool {
        mob.living.is_holding_item(
            &crate::resources::identifier::Identifier::minecraft("crossbow"),
        )
    }

    /// `canRun()`.
    fn can_run(&self) -> bool {
        self.crossbow_state == CrossbowState::Uncharged
    }
}

impl Goal for RangedCrossbowAttackGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        self.is_valid_target(mob) && self.is_holding_crossbow(mob)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.is_valid_target(mob)
            && (self.can_use(mob) || !mob.navigation_is_done())
            && self.is_holding_crossbow(mob)
    }

    fn stop(&mut self, mob: &mut Mob) {
        mob.set_aggressive(false);
        mob.set_target(None);
        self.see_time = 0;
        if mob.living.is_using_item() {
            mob.living.stop_using_item();
            mob.set_charging_crossbow(false);
            // `useItem.set(CHARGED_PROJECTILES, ChargedProjectiles.EMPTY)`.
            if let Some(ty) = crate::core::component::data_component_type::data_component_type_by_identifier(
                &crate::resources::identifier::Identifier::minecraft("charged_projectiles"),
            ) {
                mob.living.use_item.components.set(
                    &ty,
                    Some(crate::world::item::component::charged_projectiles::ChargedProjectiles::empty()),
                );
            }
        }
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(target) = mob.get_target().cloned() else {
            return;
        };
        let has_line_of_sight = mob.has_line_of_sight(&target.entity);
        let flag = self.see_time > 0;
        if has_line_of_sight != flag {
            self.see_time = 0;
        }
        if has_line_of_sight {
            self.see_time += 1;
        } else {
            self.see_time -= 1;
        }

        let d = target.entity.distance_to_sqr(&mob.living.entity);
        let flag1 = (d > self.attack_radius_sqr as f64 || self.see_time < 5)
            && self.attack_delay == 0;
        if flag1 {
            self.update_path_delay -= 1;
            if self.update_path_delay <= 0 {
                let speed = if self.can_run() {
                    self.speed_modifier
                } else {
                    self.speed_modifier * 0.5
                };
                if let Some(nav) = mob.get_navigation() {
                    nav.borrow_mut().move_to_entity(mob, &target, speed);
                }
                // `PATHFINDING_DELAY_RANGE` = rangeOfSeconds(1,2)
                // = UniformInt(20, 40) inclusive → nextInt(20, 41).
                self.update_path_delay =
                    20 + mob.random.borrow_mut().next_int_range(0, 21);
            }
        } else {
            self.update_path_delay = 0;
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().stop(mob);
            }
        }

        mob.look_control
            .set_look_at_entity_deltas(&target.entity, 30.0, 30.0);

        match self.crossbow_state {
            CrossbowState::Uncharged => {
                if !flag1 {
                    let crossbow =
                        crate::resources::identifier::Identifier::minecraft("crossbow");
                    let hand =
                        crate::world::entity::projectile::projectile_util::get_weapon_holding_hand(
                            &mob.living,
                            &crossbow,
                        );
                    let arm = crate::world::entity::projectile::projectile_util::hand_to_arm(
                        &mob.living,
                        hand,
                    );
                    mob.living.start_using_item(arm);
                    self.crossbow_state = CrossbowState::Charging;
                    mob.set_charging_crossbow(true);
                }
            }
            CrossbowState::Charging => {
                if !mob.living.is_using_item() {
                    self.crossbow_state = CrossbowState::Uncharged;
                }
                let ticks_using_item = mob.living.get_ticks_using_item();
                let use_item = mob.living.get_use_item().clone();
                let charge_duration =
                    crate::world::item::crossbow_item::get_charge_duration(
                        &use_item,
                        &mut mob.living,
                    );
                if ticks_using_item >= charge_duration {
                    mob.living.release_using_item();
                    self.crossbow_state = CrossbowState::Charged;
                    self.attack_delay = 20 + mob.random.borrow_mut().next_int_bound(20);
                    mob.set_charging_crossbow(false);
                }
            }
            CrossbowState::Charged => {
                self.attack_delay -= 1;
                if self.attack_delay == 0 {
                    self.crossbow_state = CrossbowState::ReadyToAttack;
                }
            }
            CrossbowState::ReadyToAttack => {
                if has_line_of_sight {
                    let mut t = target.clone();
                    (self.perform_ranged_attack)(mob, &mut t, 1.0);
                    self.crossbow_state = CrossbowState::Uncharged;
                }
            }
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move, GoalFlag::Look].into_iter().collect()
    }
}
