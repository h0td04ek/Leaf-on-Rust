//! Перенос боевых/поисковых поведений: `MeleeAttack`,
//! `CrossbowAttack`, `ChargeAttack`, `TryFindLand`, `TryFindWater`,
//! `TryFindLandNearWater`, `TryLaySpawnOnWaterNearLand`,
//! `PrepareRamNearestTarget`, `RamTarget`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::direction::{Direction, Plane};
use crate::entity::humanoid_arm::HumanoidArm;
use crate::entity::ai::attributes::attributes as attrs;
use crate::resources::identifier::Identifier;
use crate::world::entity::ai::behavior::behavior::{BehaviorBase, BehaviorControl};
use crate::world::entity::ai::behavior::behavior_utils;
use crate::world::entity::ai::behavior::simple::{conds, PointBehavior};
use crate::world::entity::ai::memory::memory::{
    keys, EntityTracker, MemoryModuleType, MemoryStatus, NearestVisibleLivingEntities,
    PositionTracker, WalkTarget,
};
use crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions;
use crate::world::entity::ai::util::land_random_pos;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::entity::projectile::projectile_util;
use crate::world::interaction_hand::InteractionHand;
use crate::world::item::crossbow_item;
use crate::world::item::item_stack::ItemStack;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;
use crate::world::phys::aabb::Aabb;

fn crossbow_id() -> Identifier {
    Identifier::with_default_namespace("crossbow").unwrap()
}

fn charged_projectiles_id() -> Identifier {
    Identifier::with_default_namespace("charged_projectiles").unwrap()
}

fn tracker_of(e: &LivingEntity, eye: bool) -> Rc<dyn PositionTracker> {
    Rc::new(EntityTracker::new(e.entity.clone(), eye)) as Rc<dyn PositionTracker>
}

// ===================== MeleeAttack =====================

/// `MeleeAttack.create(attackCooldown)`.
pub fn melee_attack(attack_cooldown: i32) -> PointBehavior {
    melee_attack_pred(Rc::new(|_| true), attack_cooldown)
}

/// `MeleeAttack.create(canAttack, attackCooldown)` —
/// LOOK registered + ATTACK present + ATTACK_COOLING_DOWN absent +
/// NVLE present; в melee-радиусе и виден → look+swing+doHurtTarget+
/// ATTACK_COOLING_DOWN=expiry(cooldown).
pub fn melee_attack_pred(
    can_attack: Rc<dyn Fn(&Mob) -> bool>,
    attack_cooldown: i32,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::look_target(), MemoryStatus::Registered),
            (keys::attack_target(), MemoryStatus::ValuePresent),
            (keys::attack_cooling_down(), MemoryStatus::ValueAbsent),
            (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
        ]),
        move |_level, mob, _game_time| {
            let Some(target): Option<LivingEntity> =
                mob.living.brain_get(&keys::attack_target())
            else {
                return false;
            };
            let usable_non_melee = mob
                .living
                .is_holding(|_s| mob.living.can_use_non_melee_weapon());
            if !can_attack(mob) || usable_non_melee { return false }
            if !mob.is_within_melee_attack_range(&target) { return false }
            let nvle: Option<NearestVisibleLivingEntities> =
                mob.living.brain_get(&keys::nearest_visible_living_entities());
            let Some(nvle) = nvle else { return false };
            if !nvle.contains(&target) { return false }
            mob.living
                .brain_set_value(&keys::look_target(), tracker_of(&target, true));
            let arm = mob.living.get_main_arm();
            mob.living.swing(arm);
            let mut t = target.clone();
            mob.do_hurt_target(&mut t);
            mob.living
                .brain_set_with_expiry(&keys::attack_cooling_down(), true, attack_cooldown as i64);
            true
        },
    )
}

// ===================== CrossbowAttack =====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CrossbowState {
    Uncharged,
    Charging,
    Charged,
    ReadyToAttack,
}

/// `CrossbowAttack` — crossbow charge → performRangedAttack.
pub struct CrossbowAttack {
    base: BehaviorBase,
    attack_delay: i32,
    state: CrossbowState,
}

impl CrossbowAttack {
    /// `new()`.
    pub fn new() -> Self {
        Self {
            base: BehaviorBase::with_duration(
                BehaviorBase::conditions(&[
                    (keys::look_target(), MemoryStatus::Registered),
                    (keys::attack_target(), MemoryStatus::ValuePresent),
                ]),
                1200,
            ),
            attack_delay: 0,
            state: CrossbowState::Uncharged,
        }
    }

    fn get_attack_target(mob: &Mob) -> Option<LivingEntity> {
        mob.living.brain_get(&keys::attack_target())
    }

    fn check_extra(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
        let Some(target) = Self::get_attack_target(mob) else { return false };
        let holding = mob
            .living
            .is_holding(|s| s.is(&crossbow_id()));
        holding
            && behavior_utils::can_see(&mob.living, &target)
            && behavior_utils::is_within_attack_range(mob, &target, 0)
    }

    fn look_at_target(mob: &mut Mob, target: &LivingEntity) {
        mob.living
            .brain_set_value(&keys::look_target(), tracker_of(target, true));
    }

    fn crossbow_attack(&mut self, mob: &mut Mob, target: &LivingEntity) {
        match self.state {
            CrossbowState::Uncharged => {
                let hand = projectile_util::get_weapon_holding_hand(
                    &mob.living,
                    &crossbow_id(),
                );
                let arm = projectile_util::hand_to_arm(&mob.living, hand);
                mob.living.start_using_item(arm);
                self.state = CrossbowState::Charging;
                mob.set_charging_crossbow(true);
            }
            CrossbowState::Charging => {
                if !mob.living.is_using_item() {
                    self.state = CrossbowState::Uncharged;
                }
                let ticks_using = mob.living.get_ticks_using_item();
                let charge = {
                    let stack = mob.living.get_use_item().clone();
                    crossbow_item::get_charge_duration(&stack, &mut mob.living)
                };
                if ticks_using >= charge {
                    mob.living.release_using_item();
                    self.state = CrossbowState::Charged;
                    self.attack_delay = 20 + mob.random.borrow_mut().next_int_bound(20);
                    mob.set_charging_crossbow(false);
                }
            }
            CrossbowState::Charged => {
                self.attack_delay -= 1;
                if self.attack_delay == 0 {
                    self.state = CrossbowState::ReadyToAttack;
                }
            }
            CrossbowState::ReadyToAttack => {
                mob.living.perform_ranged_attack(target, 1.0);
                self.state = CrossbowState::Uncharged;
            }
        }
    }
}

impl BehaviorControl for CrossbowAttack {
    fn status(&self) -> crate::world::entity::ai::behavior::behavior::BehaviorStatus {
        self.base.status
    }
    fn set_status(&mut self, status: crate::world::entity::ai::behavior::behavior::BehaviorStatus) {
        self.base.status = status;
    }
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
        &self.base.entry_condition
    }
    fn durations(&self) -> (i32, i32) {
        (self.base.min_duration, self.base.max_duration)
    }
    fn end_timestamp(&self) -> i64 {
        self.base.end_timestamp
    }
    fn set_end_timestamp(&mut self, timestamp: i64) {
        self.base.end_timestamp = timestamp;
    }

    fn check_extra_start_conditions(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
        self.check_extra(level, mob)
    }

    fn can_still_use(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
        mob.living.brain_has(&keys::attack_target()) && self.check_extra(level, mob)
    }

    fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        let Some(target) = Self::get_attack_target(mob) else { return };
        Self::look_at_target(mob, &target);
        self.crossbow_attack(mob, &target);
    }

    fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        if mob.living.is_using_item() {
            mob.living.stop_using_item();
        }
        let holding = mob.living.is_holding(|s| s.is(&crossbow_id()));
        if holding {
            mob.set_charging_crossbow(false);
            mob.living
                .use_item
                .set_component(&charged_projectiles_id(), Some(crate::world::item::component::charged_projectiles::ChargedProjectiles::empty()));
        }
    }

    fn debug_string(&self) -> String {
        "CrossbowAttack".to_string()
    }
}

// ===================== ChargeAttack =====================

/// `ChargeAttack` — прямой рывок к цели с уроном+отбросом.
pub struct ChargeAttack {
    base: BehaviorBase,
    time_between_attacks: i32,
    charge_targeting: Option<Rc<TargetingConditions>>,
    speed: f32,
    knockback_force: f32,
    max_target_detection_distance: f64,
    max_charge_distance: f64,
    charge_sound: Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>,
    charge_velocity_vector: Vec3,
    start_position: Vec3,
}

impl ChargeAttack {
    /// `new(timeBetweenAttacks, chargeTargeting, speed, knockbackForce,
    /// maxChargeDistance, maxTargetDetectionDistance, chargeSound)`.
    pub fn new(
        time_between_attacks: i32,
        charge_targeting: Option<Rc<TargetingConditions>>,
        speed: f32,
        knockback_force: f32,
        max_charge_distance: f64,
        max_target_detection_distance: f64,
        charge_sound: Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>,
    ) -> Self {
        Self {
            base: BehaviorBase::with_duration(
                BehaviorBase::conditions(&[
                    (keys::charge_cooldown_ticks(), MemoryStatus::ValueAbsent),
                    (keys::attack_target(), MemoryStatus::ValuePresent),
                ]),
                60,
            ),
            time_between_attacks,
            charge_targeting,
            speed,
            knockback_force,
            max_target_detection_distance,
            max_charge_distance,
            charge_sound,
            charge_velocity_vector: Vec3::ZERO,
            start_position: Vec3::ZERO,
        }
    }
}

impl BehaviorControl for ChargeAttack {
    fn status(&self) -> crate::world::entity::ai::behavior::behavior::BehaviorStatus {
        self.base.status
    }
    fn set_status(&mut self, status: crate::world::entity::ai::behavior::behavior::BehaviorStatus) {
        self.base.status = status;
    }
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
        &self.base.entry_condition
    }
    fn durations(&self) -> (i32, i32) {
        (self.base.min_duration, self.base.max_duration)
    }
    fn end_timestamp(&self) -> i64 {
        self.base.end_timestamp
    }
    fn set_end_timestamp(&mut self, timestamp: i64) {
        self.base.end_timestamp = timestamp;
    }

    fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
        mob.living.brain_has(&keys::attack_target())
    }

    fn can_still_use(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
        let Some(target): Option<LivingEntity> = mob.living.brain_get(&keys::attack_target()) else { return false };
        // !(isTame) — упрощённо через is_tame hook.
        if mob.living.is_tame() { return false }
        if mob.living.entity.position().subtract_vec(&self.start_position).length_sqr()
            >= self.max_charge_distance * self.max_charge_distance
        {
            return false;
        }
        if target.entity.position().subtract_vec(&mob.living.entity.position()).length_sqr()
            >= self.max_target_detection_distance * self.max_target_detection_distance
        {
            return false;
        }
        if !mob.living.has_line_of_sight(&target.entity) { return false }
        !mob.living.brain_has(&keys::charge_cooldown_ticks())
    }

    fn start(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        self.start_position = mob.living.entity.position();
        let Some(target): Option<LivingEntity> = mob.living.brain_get(&keys::attack_target()) else { return };
        let vec3 = target
            .entity
            .position()
            .subtract_vec(&mob.living.entity.position())
            .normalize()
            .scale(self.speed as f64);
        self.charge_velocity_vector = vec3;
        if self.can_still_use(level, mob, game_time) {
            if let Some(sound) = &self.charge_sound {
                mob.living.entity.play_sound(Some(sound), 1.0, 1.0);
            }
        }
    }

    fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        let Some(target): Option<LivingEntity> = mob.living.brain_get(&keys::attack_target()) else { return };
        // lookAt(target, 360, 360) — упрощённо LOOK_TARGET на цель.
        mob.living
            .brain_set_value(&keys::look_target(), tracker_of(&target, true));
        mob.living.entity.set_delta_movement(
            self.charge_velocity_vector.x,
            self.charge_velocity_vector.y,
            self.charge_velocity_vector.z,
        );
        // entities в bounding box, фильтр chargeTargeting.
        let bb = mob.living.entity.get_bounding_box().inflate_all(1.0);
        let candidates = level.entities_in_box(&bb);
        let mut hit: Option<LivingEntity> = None;
        if let Some(cond) = &self.charge_targeting {
            for e in candidates {
                if !e.is_living_entity() { continue }
                let le = LivingEntity::from_entity(e);
                if cond.test(level, Some(mob), &le) {
                    hit = Some(le);
                    break;
                }
            }
        } else {
            let mut best: Option<(LivingEntity, f64)> = None;
            for e in candidates {
                if !e.is_living_entity() { continue }
                let le = LivingEntity::from_entity(e);
                let d = le.entity.distance_to_sqr(&mob.living.entity);
                if best.as_ref().map(|(_, bd)| d < *bd).unwrap_or(true) {
                    best = Some((le, d));
                }
            }
            hit = best.map(|(l, _)| l);
        }
        if let Some(target1) = hit {
            if mob.living.entity.has_passenger(&target1.entity) {
                return;
            }
            // dealDamageToTarget.
            let ds = crate::world::damagesource::damage_sources::DamageSources::vanilla()
                .mob_attack(&mob.living.entity);
            let f = mob
                .living
                .get_attribute_value_by_id_ro(&attrs::attack_damage());
            let mut t1 = target1.clone();
            let hurt = t1.hurt_server(ds.clone(), f as f32);
            if hurt {
                crate::world::item::enchantment::enchantment_helper::do_post_attack_effects(
                    &mut t1, &ds,
                );
            }
            // dealKnockBack.
            let i = mob
                .living
                .get_effect_instance(&crate::world::effect::mob_effects::speed())
                .map(|e| e.amplifier() + 1)
                .unwrap_or(0);
            let i1 = mob
                .living
                .get_effect_instance(&crate::world::effect::mob_effects::slowness())
                .map(|e| e.amplifier() + 1)
                .unwrap_or(0);
            let f_speed = 0.25 * (i - i1) as f32;
            let movement = mob
                .living
                .get_attribute_value_by_id_ro(&attrs::movement_speed());
            let f1 = crate::util::mth::clamp_f32(
                self.speed * movement as f32,
                0.2,
                2.0,
            ) + f_speed;
            let delta = mob.living.entity.get_delta_movement();
            let mut t1m = t1.clone();
            t1m.cause_extra_knockback(&mut t1, f1 * self.knockback_force, delta);
            self.stop(level, mob, _game_time);
        }
    }

    fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        mob.living
            .brain_set_value(&keys::charge_cooldown_ticks(), self.time_between_attacks);
        mob.living.brain_erase(&keys::attack_target());
    }

    fn debug_string(&self) -> String {
        "ChargeAttack".to_string()
    }
}

// ===================== TryFindLand / TryFindWater /
// TryFindLandNearWater / TryLaySpawnOnWaterNearLand =====================

fn block_is(state: &crate::world::level::block::state::block_state::BlockState, id: &str) -> bool {
    state.block.id == Identifier::with_default_namespace(id).unwrap()
}

/// `TryFindLand.create(range, speedModifier)` — ATTACK absent +
/// WALK absent + LOOK registered; в воде → ищет землю (60 cd).
pub fn try_find_land(range: i32, speed_modifier: f32) -> PointBehavior {
    let next_update = Rc::new(RefCell::new(0i64));
    let nu = next_update.clone();
    PointBehavior::new(
        conds(&[
            (keys::attack_target(), MemoryStatus::ValueAbsent),
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::look_target(), MemoryStatus::Registered),
        ]),
        move |level, mob, game_time| {
            let in_water = level
                .get_fluid_state(mob.block_position())
                .map(|f| f.is_water())
                .unwrap_or(false);
            if !in_water {
                return false;
            }
            let mut nu = nu.borrow_mut();
            if game_time < *nu {
                *nu = game_time + 60;
                return true;
            }
            let block_pos = mob.block_position();
            let mut found: Option<BlockPos> = None;
            for bp in BlockPos::within_manhattan(&block_pos, range, range, range) {
                if bp.x == block_pos.x && bp.z == block_pos.z { continue }
                // collisionShape(above).isEmpty && collisionShape(down) !empty
                let above = bp.above();
                let down = bp.below();
                let above_empty = level
                    .get_block_state(above)
                    .map(|s| s.collision_shape_unleveled().is_empty())
                    .unwrap_or(true);
                let down_solid = level
                    .get_block_state(down)
                    .map(|s| !s.collision_shape_unleveled().is_empty())
                    .unwrap_or(false);
                if above_empty && down_solid {
                    found = Some(bp);
                    break;
                }
            }
            drop(nu);
            if let Some(bp) = found {
                let tracker: Rc<dyn PositionTracker> = Rc::new(
                    crate::world::entity::ai::memory::memory::BlockPosTracker::new(bp),
                );
                mob.living.brain_set_value(&keys::look_target(), tracker.clone());
                let wt = WalkTarget::of_tracker(tracker, speed_modifier, 0);
                mob.living.brain_set_value(&keys::walk_target(), wt);
            }
            *next_update.borrow_mut() = game_time + 60;
            true
        },
    )
}

/// `TryFindWater.create(range, speedModifier)` — не в воде →
/// ищет воду (manhattan range); cooldown 40 + jitter 2.
pub fn try_find_water(range: i32, speed_modifier: f32) -> PointBehavior {
    let next_update = Rc::new(RefCell::new(0i64));
    let nu = next_update.clone();
    PointBehavior::new(
        conds(&[
            (keys::attack_target(), MemoryStatus::ValueAbsent),
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::look_target(), MemoryStatus::Registered),
        ]),
        move |level, mob, game_time| {
            let in_water = level
                .get_fluid_state(mob.block_position())
                .map(|f| f.is_water())
                .unwrap_or(false);
            if in_water {
                return false;
            }
            let mut nu = nu.borrow_mut();
            if game_time < *nu {
                *nu = game_time + 22;
                return true;
            }
            let block_pos = mob.block_position();
            let mut found_air_above: Option<BlockPos> = None;
            let mut found_other: Option<BlockPos> = None;
            for bp in BlockPos::within_manhattan(&block_pos, range, range, range) {
                if bp.x == block_pos.x && bp.z == block_pos.z { continue }
                let above_state = level.get_block_state(bp.above());
                let state = level.get_block_state(bp);
                let is_water = state
                    .as_ref()
                    .map(|s| block_is(&s, "water"))
                    .unwrap_or(false);
                if is_water {
                    let above_air = above_state
                        .as_ref()
                        .map(|s| s.is_air())
                        .unwrap_or(false);
                    if above_air {
                        found_air_above = Some(bp);
                        break;
                    }
                    if found_other.is_none()
                        && !bp.closer_to_center_than(mob.living.entity.position(), 1.5)
                    {
                        found_other = Some(bp);
                    }
                }
            }
            drop(nu);
            let target = found_air_above.or(found_other);
            if let Some(bp) = target {
                let tracker: Rc<dyn PositionTracker> = Rc::new(
                    crate::world::entity::ai::memory::memory::BlockPosTracker::new(bp),
                );
                mob.living.brain_set_value(&keys::look_target(), tracker.clone());
                let wt = WalkTarget::of_tracker(tracker, speed_modifier, 0);
                mob.living.brain_set_value(&keys::walk_target(), wt);
            }
            *next_update.borrow_mut() = game_time + 40;
            true
        },
    )
}

/// `TryFindLandNearWater.create(range, speedModifier)` — не в воде →
/// ищет землю у воды (40 cd).
pub fn try_find_land_near_water(range: i32, speed_modifier: f32) -> PointBehavior {
    let next_update = Rc::new(RefCell::new(0i64));
    let nu = next_update.clone();
    PointBehavior::new(
        conds(&[
            (keys::attack_target(), MemoryStatus::ValueAbsent),
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::look_target(), MemoryStatus::Registered),
        ]),
        move |level, mob, game_time| {
            let in_water = level
                .get_fluid_state(mob.block_position())
                .map(|f| f.is_water())
                .unwrap_or(false);
            if in_water {
                return false;
            }
            let mut nu = nu.borrow_mut();
            if game_time < *nu {
                *nu = game_time + 40;
                return true;
            }
            let block_pos = mob.block_position();
            let mut found: Option<BlockPos> = None;
            'outer: for bp in BlockPos::within_manhattan(&block_pos, range, range, range) {
                if bp.x == block_pos.x && bp.z == block_pos.z { continue }
                // collisionShape(bp).isEmpty && !collisionShape(down).isEmpty
                let self_empty = level
                    .get_block_state(bp)
                    .map(|s| s.collision_shape_unleveled().is_empty())
                    .unwrap_or(true);
                let down_solid = level
                    .get_block_state(bp.below())
                    .map(|s| !s.collision_shape_unleveled().is_empty())
                    .unwrap_or(false);
                if !(self_empty && down_solid) { continue }
                // горизонтальный сосед: air + below == water
                for dir in Plane::HORIZONTAL.faces() {
                    let side = bp.relative(*dir);
                    let side_air = level
                        .get_block_state(side)
                        .map(|s| s.is_air())
                        .unwrap_or(false);
                    let side_below_water = level
                        .get_block_state(side.below())
                        .map(|s| block_is(&s, "water"))
                        .unwrap_or(false);
                    if side_air && side_below_water {
                        found = Some(bp);
                        break 'outer;
                    }
                }
            }
            drop(nu);
            if let Some(bp) = found {
                let tracker: Rc<dyn PositionTracker> = Rc::new(
                    crate::world::entity::ai::memory::memory::BlockPosTracker::new(bp),
                );
                mob.living.brain_set_value(&keys::look_target(), tracker.clone());
                let wt = WalkTarget::of_tracker(tracker, speed_modifier, 0);
                mob.living.brain_set_value(&keys::walk_target(), wt);
            }
            *next_update.borrow_mut() = game_time + 40;
            true
        },
    )
}

/// `TryLaySpawnOnWaterNearLand.create(spawnBlockId)` —
/// IS_PREGNANT + WALK present; на суше → ищет воду рядом,
/// ставит spawn-блок, erase IS_PREGNANT.
pub fn try_lay_spawn_on_water_near_land(spawn_block_id: &str) -> PointBehavior {
    let spawn_id = spawn_block_id.to_string();
    PointBehavior::new(
        conds(&[
            (keys::attack_target(), MemoryStatus::ValueAbsent),
            (keys::walk_target(), MemoryStatus::ValuePresent),
            (keys::is_pregnant(), MemoryStatus::ValuePresent),
        ]),
        move |level, mob, _game_time| {
            if mob.living.entity.is_in_water() || !mob.living.entity.is_on_ground() {
                return false;
            }
            let block_pos = mob.block_position().below();
            let spawn_block = Identifier::with_default_namespace(&spawn_id).unwrap();
            let mut placed = false;
            for dir in Plane::HORIZONTAL.faces() {
                let bp1 = block_pos.relative(*dir);
                // collisionShape.getFaceShape(UP).isEmpty ≈ !entity_can_stand_on
                let face_open = level
                    .get_block_state(bp1)
                    .map(|s| {
                        !(s.entity_can_stand_on(level.as_ref(), bp1))
                    })
                    .unwrap_or(true);
                let water = level
                    .get_fluid_state(bp1)
                    .map(|f| f.is_water())
                    .unwrap_or(false);
                if !(face_open && water) { continue }
                let bp2 = bp1.above();
                let air = level
                    .get_block_state(bp2)
                    .map(|s| s.is_air())
                    .unwrap_or(false);
                if !air { continue }
                let _placed = level.set_block_by_id(bp2, &spawn_block);
                // gameEvent BLOCK_PLACE at bp2.
                level.game_event(
                    &crate::world::level::gameevent::game_event::block_place(),
                    bp2,
                );
                mob.living.entity.play_sound(
                    Some(&crate::sounds::sound_events::frog_lay_spawn()),
                    1.0,
                    1.0,
                );
                mob.living.brain_erase(&keys::is_pregnant());
                placed = true;
                break;
            }
            let _ = placed;
            true
        },
    )
}

// ===================== PrepareRamNearestTarget =====================

/// `PrepareRamNearestTarget` — выбор позиции тарана по NVLE.
pub struct PrepareRamNearestTarget {
    base: BehaviorBase,
    get_cooldown_on_fail: Rc<dyn Fn(&Mob) -> i32>,
    min_ram_distance: i32,
    max_ram_distance: i32,
    walk_speed: f32,
    ram_targeting: Rc<TargetingConditions>,
    ram_prepare_time: i32,
    get_prepare_ram_sound: Rc<dyn Fn(&Mob) -> Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>>,
    reached_ram_position_timestamp: Option<i64>,
    ram_candidate: Option<RamCandidate>,
}

#[derive(Clone)]
/// `RamCandidate(startPosition, targetPosition, target)`.
pub struct RamCandidate {
    pub start_position: BlockPos,
    pub target_position: BlockPos,
    pub target: LivingEntity,
}

impl PrepareRamNearestTarget {
    #[allow(clippy::too_many_arguments)]
    /// `new(getCooldownOnFail, minRamDistance, maxRamDistance,
    /// walkSpeed, ramTargeting, ramPrepareTime, getPrepareRamSound)`.
    pub fn new(
        get_cooldown_on_fail: Rc<dyn Fn(&Mob) -> i32>,
        min_ram_distance: i32,
        max_ram_distance: i32,
        walk_speed: f32,
        ram_targeting: Rc<TargetingConditions>,
        ram_prepare_time: i32,
        get_prepare_ram_sound: Rc<dyn Fn(&Mob) -> Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>>,
    ) -> Self {
        Self {
            base: BehaviorBase::with_duration(
                BehaviorBase::conditions(&[
                    (keys::look_target(), MemoryStatus::Registered),
                    (keys::ram_cooldown_ticks(), MemoryStatus::ValueAbsent),
                    (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
                    (keys::ram_target(), MemoryStatus::ValueAbsent),
                ]),
                160,
            ),
            get_cooldown_on_fail,
            min_ram_distance,
            max_ram_distance,
            walk_speed,
            ram_targeting,
            ram_prepare_time,
            get_prepare_ram_sound,
            reached_ram_position_timestamp: None,
            ram_candidate: None,
        }
    }

    fn edge_of_block(&self, pos: BlockPos, other: BlockPos) -> Vec3 {
        let d1 = 0.5 * crate::util::mth::sign((other.x - pos.x) as f64) as f64;
        let d2 = 0.5 * crate::util::mth::sign((other.z - pos.z) as f64) as f64;
        crate::world::phys::vec3::at_bottom_center_of(&other.to_vec3i())
            .add(d1, 0.0, d2)
    }

    fn is_walkable_block(&self, level: &Arc<dyn LevelReader>, mob: &Mob, pos: BlockPos) -> bool {
        let stable = mob
            .navigation
            .as_ref()
            .map(|n| n.borrow().is_stable_destination(mob, pos))
            .unwrap_or(false);
        let pt = crate::world::entity::ai::util::goal_utils::walk_path_type_static(mob, pos);
        use crate::world::entity::mob::MobPathing;
        let malus = mob.get_pathfinding_malus(pt);
        stable && malus == 0.0
    }

    fn calculate_ramming_start_position(
        &self,
        level: &Arc<dyn LevelReader>,
        mob: &Mob,
        entity: &LivingEntity,
    ) -> Option<BlockPos> {
        let block_pos = entity.block_position();
        if !self.is_walkable_block(level, mob, block_pos) {
            return None;
        }
        let mut list: Vec<BlockPos> = Vec::new();
        for dir in Plane::HORIZONTAL.faces() {
            let mut cur = block_pos;
            for _ in 0..self.max_ram_distance {
                let next = cur.relative(*dir);
                if !self.is_walkable_block(level, mob, next) {
                    break;
                }
                cur = next;
            }
            if cur.dist_manhattan(block_pos) >= self.min_ram_distance {
                list.push(cur);
            }
        }
        let own_pos = mob.block_position();
        list.sort_by(|a, b| {
            a.dist_sqr(own_pos)
                .partial_cmp(&b.dist_sqr(own_pos))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for pos in list {
            let path = mob
                .navigation
                .as_ref()
                .and_then(|n| n.borrow_mut().create_path_pos(mob, pos, 0));
            if let Some(path) = path {
                if path.can_reach() {
                    return Some(pos);
                }
            }
        }
        None
    }

    fn choose_ram_position(&mut self, level: &Arc<dyn LevelReader>, mob: &Mob, entity: &LivingEntity) {
        self.reached_ram_position_timestamp = None;
        self.ram_candidate = self
            .calculate_ramming_start_position(level, mob, entity)
            .map(|bp| RamCandidate {
                start_position: bp,
                target_position: entity.block_position(),
                target: entity.clone(),
            });
    }
}

impl BehaviorControl for PrepareRamNearestTarget {
    fn status(&self) -> crate::world::entity::ai::behavior::behavior::BehaviorStatus {
        self.base.status
    }
    fn set_status(&mut self, status: crate::world::entity::ai::behavior::behavior::BehaviorStatus) {
        self.base.status = status;
    }
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
        &self.base.entry_condition
    }
    fn durations(&self) -> (i32, i32) {
        (self.base.min_duration, self.base.max_duration)
    }
    fn end_timestamp(&self) -> i64 {
        self.base.end_timestamp
    }
    fn set_end_timestamp(&mut self, timestamp: i64) {
        self.base.end_timestamp = timestamp;
    }

    fn start(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        let nvle: Option<NearestVisibleLivingEntities> =
            mob.living.brain_get(&keys::nearest_visible_living_entities());
        if let Some(nvle) = nvle {
            // findClosest по ramTargeting — test требует &mut Mob;
            // делаем через клон-итерацию (Mob view).
            let mut found: Option<LivingEntity> = None;
            for e in &nvle.nearby_entities {
                let le = e.clone();
                // targeting на копии mob не мутирует состояние —
                // передаём None (non-combat ветки) + отдельный mob.
                let _ = mob;
                if self.ram_targeting.test(level, None, &le) {
                    found = Some(le);
                    break;
                }
            }
            if let Some(e) = found {
                self.choose_ram_position(level, mob, &e);
            }
        }
    }

    fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) -> bool {
        self.ram_candidate
            .as_ref()
            .map(|c| c.target.is_alive())
            .unwrap_or(false)
    }

    fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        let Some(cand) = self.ram_candidate.clone() else { return };
        let wt = WalkTarget::new_block(cand.start_position, self.walk_speed, 0);
        mob.living.brain_set_value(&keys::walk_target(), wt);
        mob.living
            .brain_set_value(&keys::look_target(), tracker_of(&cand.target, true));
        let target_moved = cand.target.block_position() != cand.target_position;
        if target_moved {
            level.broadcast_entity_event(mob.living.entity.id, crate::world::entity::entity_event::END_RAM);
            if let Some(nav) = mob.navigation.clone() {
                nav.borrow_mut().stop(mob);
            }
            self.choose_ram_position(level, mob, &cand.target);
            return;
        }
        let block_pos = mob.block_position();
        if block_pos == cand.start_position {
            level.broadcast_entity_event(mob.living.entity.id, crate::world::entity::entity_event::START_RAM);
            if self.reached_ram_position_timestamp.is_none() {
                self.reached_ram_position_timestamp = Some(game_time);
            }
            if game_time - self.reached_ram_position_timestamp.unwrap() >= self.ram_prepare_time as i64 {
                let edge = self.edge_of_block(block_pos, cand.target_position);
                mob.living.brain_set_value(&keys::ram_target(), edge);
                if let Some(sound) = (self.get_prepare_ram_sound)(mob) {
                    mob.living.entity.play_sound(Some(&sound), 1.0, mob.living.get_voice_pitch());
                }
                self.ram_candidate = None;
            }
        }
    }

    fn stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        if !mob.living.brain_has(&keys::ram_target()) {
            level.broadcast_entity_event(mob.living.entity.id, crate::world::entity::entity_event::END_RAM);
            let cd = (self.get_cooldown_on_fail)(mob);
            mob.living.brain_set_value(&keys::ram_cooldown_ticks(), cd);
        }
    }

    fn debug_string(&self) -> String {
        "PrepareRamNearestTarget".to_string()
    }
}

// ===================== RamTarget =====================

/// `RamTarget` — тараним по RAM_TARGET (Vec3), урон+отброс,
/// поломка рога о SNAPS_GOAT_HORN.
pub struct RamTarget {
    base: BehaviorBase,
    get_time_between_rams: Rc<dyn Fn(&Mob) -> crate::util::valueproviders::uniform_int::UniformInt>,
    ram_targeting: Rc<TargetingConditions>,
    speed: f32,
    get_knockback_force: Rc<dyn Fn(&Mob) -> f64>,
    ram_direction: Vec3,
    get_impact_sound: Rc<dyn Fn(&Mob) -> Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>>,
    get_horn_break_sound: Rc<dyn Fn(&Mob) -> Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>>,
}

impl RamTarget {
    #[allow(clippy::too_many_arguments)]
    /// `new(getTimeBetweenRams, ramTargeting, speed, getKnockbackForce,
    /// getImpactSound, getHornBreakSound)`.
    pub fn new(
        get_time_between_rams: Rc<dyn Fn(&Mob) -> crate::util::valueproviders::uniform_int::UniformInt>,
        ram_targeting: Rc<TargetingConditions>,
        speed: f32,
        get_knockback_force: Rc<dyn Fn(&Mob) -> f64>,
        get_impact_sound: Rc<dyn Fn(&Mob) -> Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>>,
        get_horn_break_sound: Rc<dyn Fn(&Mob) -> Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>>,
    ) -> Self {
        Self {
            base: BehaviorBase::with_duration(
                BehaviorBase::conditions(&[
                    (keys::ram_cooldown_ticks(), MemoryStatus::ValueAbsent),
                    (keys::ram_target(), MemoryStatus::ValuePresent),
                ]),
                200,
            ),
            get_time_between_rams,
            ram_targeting,
            speed,
            get_knockback_force,
            ram_direction: Vec3::ZERO,
            get_impact_sound,
            get_horn_break_sound,
        }
    }

    fn has_rammed_horn_breaking_block(&self, level: &Arc<dyn LevelReader>, mob: &Mob) -> bool {
        let vec3 = mob
            .living
            .entity
            .get_delta_movement()
            .multiply(1.0, 0.0, 1.0)
            .normalize();
        let pos = mob.living.entity.position().add_vec(&vec3);
        let block_pos = BlockPos::containing(pos.x, pos.y, pos.z);
        let tag = Identifier::with_default_namespace("snaps_goat_horn").unwrap();
        level
            .get_block_state(block_pos)
            .map(|s| s.is_block_tag(&tag))
            .unwrap_or(false)
            || level
                .get_block_state(block_pos.above())
                .map(|s| s.is_block_tag(&tag))
                .unwrap_or(false)
    }

    fn finish_ram(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        level.broadcast_entity_event(mob.living.entity.id, crate::world::entity::entity_event::END_RAM);
        let interval = (self.get_time_between_rams)(mob);
        let sample = {
            let mut r = mob.random.borrow_mut();
            crate::util::valueproviders::int_provider::IntProvider::sample(&interval, &mut **r)
        };
        mob.living.brain_set_value(&keys::ram_cooldown_ticks(), sample);
        mob.living.brain_erase(&keys::ram_target());
    }
}

impl BehaviorControl for RamTarget {
    fn status(&self) -> crate::world::entity::ai::behavior::behavior::BehaviorStatus {
        self.base.status
    }
    fn set_status(&mut self, status: crate::world::entity::ai::behavior::behavior::BehaviorStatus) {
        self.base.status = status;
    }
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
        &self.base.entry_condition
    }
    fn durations(&self) -> (i32, i32) {
        (self.base.min_duration, self.base.max_duration)
    }
    fn end_timestamp(&self) -> i64 {
        self.base.end_timestamp
    }
    fn set_end_timestamp(&mut self, timestamp: i64) {
        self.base.end_timestamp = timestamp;
    }

    fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
        mob.living.brain_has(&keys::ram_target())
    }

    fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
        mob.living.brain_has(&keys::ram_target())
    }

    fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        let block_pos = mob.block_position();
        let Some(vec3): Option<Vec3> = mob.living.brain_get(&keys::ram_target()) else { return };
        self.ram_direction = Vec3::new(
            block_pos.x as f64 - vec3.x,
            0.0,
            block_pos.z as f64 - vec3.z,
        )
        .normalize();
        let wt = WalkTarget::new_vec3(vec3, self.speed, 0);
        mob.living.brain_set_value(&keys::walk_target(), wt);
    }

    fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        // nearbyEntities по ramTargeting.
        let bb = mob.living.entity.get_bounding_box();
        let mut nearby: Vec<LivingEntity> = Vec::new();
        for e in level.entities_in_box(&bb) {
            if !e.is_living_entity() { continue }
            let le = LivingEntity::from_entity(e);
            if self.ram_targeting.test(level, None, &le) {
                nearby.push(le);
            }
        }
        if let Some(target) = nearby.into_iter().next() {
            let ds = crate::world::damagesource::damage_sources::DamageSources::vanilla()
                .no_aggro_mob_attack(&mob.living.entity);
            let f = mob
                .living
                .get_attribute_value_by_id_ro(&attrs::attack_damage());
            let mut t = target.clone();
            let hurt = t.hurt_server(ds.clone(), f as f32);
            if hurt {
                crate::world::item::enchantment::enchantment_helper::do_post_attack_effects(
                    &mut t, &ds,
                );
            }
            let i = mob
                .living
                .get_effect_instance(&crate::world::effect::mob_effects::speed())
                .map(|e| e.amplifier() + 1)
                .unwrap_or(0);
            let i1 = mob
                .living
                .get_effect_instance(&crate::world::effect::mob_effects::slowness())
                .map(|e| e.amplifier() + 1)
                .unwrap_or(0);
            let f1 = 0.25 * (i - i1) as f32;
            let speed = mob.living.get_speed();
            let mut f2 = crate::util::mth::clamp_f32(speed * 1.65, 0.2, 3.0) + f1;
            let ds1 = crate::world::damagesource::damage_sources::DamageSources::vanilla()
                .mob_attack(&mob.living.entity);
            let f_blocked = t.apply_item_blocking(&ds1, f as f32);
            let f4 = if f_blocked > 0.0 { 0.5 } else { 1.0 };
            let kb = (self.get_knockback_force)(mob);
            t.knockback(
                (f4 * f2 * kb as f32) as f64,
                self.ram_direction.x,
                self.ram_direction.z,
            );
            self.finish_ram(level, mob);
            if let Some(sound) = (self.get_impact_sound)(mob) {
                mob.living.entity.play_sound(Some(&sound), 1.0, 1.0);
            }
        } else if self.has_rammed_horn_breaking_block(level, mob) {
            if let Some(sound) = (self.get_impact_sound)(mob) {
                mob.living.entity.play_sound(Some(&sound), 1.0, 1.0);
            }
            let flag = mob.living.drop_horn();
            if flag {
                if let Some(sound) = (self.get_horn_break_sound)(mob) {
                    mob.living.entity.play_sound(Some(&sound), 1.0, 1.0);
                }
            }
            self.finish_ram(level, mob);
        } else {
            let walk: Option<WalkTarget> = mob.living.brain_get(&keys::walk_target());
            let ram: Option<Vec3> = mob.living.brain_get(&keys::ram_target());
            let reached = match (&walk, &ram) {
                (Some(w), Some(r)) => {
                    w.get_target()
                        .current_position()
                        .closer_than(r, 0.25)
                }
                _ => true,
            };
            if reached {
                self.finish_ram(level, mob);
            }
        }
    }

    fn debug_string(&self) -> String {
        "RamTarget".to_string()
    }
}
