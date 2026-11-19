//! Перенос animal/социальных поведений: `AnimalMakeLove`,
//! `AnimalPanic`, `BabyFollowAdult`, `FollowTemptation`,
//! `PlayTagWithOtherKids`, `SetWalkTargetAwayFrom`,
//! `SetEntityLookTargetSometimes`, `StopAttackingIfTargetInvalid`,
//! `LocateHidingPlace`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::global_pos::GlobalPos;
use crate::world::entity::ai::behavior::behavior::{BehaviorBase, BehaviorControl};
use crate::world::entity::ai::behavior::behavior_utils;
use crate::world::entity::ai::behavior::simple::{conds, PointBehavior};
use crate::world::entity::ai::memory::memory::{
    keys, EntityTracker, MemoryModuleType, MemoryStatus, NearestVisibleLivingEntities,
    PositionTracker, WalkTarget,
};
use crate::world::entity::ai::sensing::sensor::entity_type_is;
use crate::world::entity::ai::util::air_and_water_random_pos;
use crate::world::entity::ai::util::default_random_pos;
use crate::world::entity::ai::util::land_random_pos;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;
use crate::util::valueproviders::int_provider::IntProvider;

fn tracker_of(entity: &LivingEntity, track_eye_height: bool) -> Rc<dyn PositionTracker> {
    Rc::new(EntityTracker::new(entity.entity.clone(), track_eye_height))
        as Rc<dyn PositionTracker>
}

fn tracker_of_full(
    entity: &LivingEntity,
    track_eye_height: bool,
    target_eye_height: bool,
) -> Rc<dyn PositionTracker> {
    Rc::new(EntityTracker::full(
        entity.entity.clone(),
        track_eye_height,
        target_eye_height,
    )) as Rc<dyn PositionTracker>
}

// ===================== AnimalMakeLove =====================

/// `AnimalMakeLove` — BREED_TARGET mutual, lockGazeAndWalkToEachOther,
/// spawnChildAtTime = gameTime + 60+rand(50), спавн при dist<3.
pub struct AnimalMakeLove {
    base: BehaviorBase,
    partner_type_id: String,
    speed_modifier: f32,
    close_enough_distance: i32,
    spawn_child_at_time: i64,
}

impl AnimalMakeLove {
    /// `new(partnerType)` — speed=1.0, closeEnough=2.
    pub fn new(partner_type_id: impl Into<String>) -> Self {
        Self::with_params(partner_type_id, 1.0, 2)
    }

    /// `new(partnerType, speedModifier, closeEnoughDistance)`.
    pub fn with_params(
        partner_type_id: impl Into<String>,
        speed_modifier: f32,
        close_enough_distance: i32,
    ) -> Self {
        let base = BehaviorBase::with_range(
            BehaviorBase::conditions(&[
                (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
                (keys::breed_target(), MemoryStatus::ValueAbsent),
                (keys::walk_target(), MemoryStatus::Registered),
                (keys::look_target(), MemoryStatus::Registered),
                (keys::is_panicking(), MemoryStatus::ValueAbsent),
            ]),
            110,
            110,
        );
        Self {
            base,
            partner_type_id: partner_type_id.into(),
            speed_modifier,
            close_enough_distance,
            spawn_child_at_time: 0,
        }
    }

    fn is_panicking(living: &LivingEntity) -> bool {
        living
            .brain
            .as_ref()
            .map(|b| b.has_memory_value(&keys::is_panicking()))
            .unwrap_or(false)
    }

    /// `findValidBreedPartner` — ближайший NVLE нужного типа,
    /// canMate && !panicking.
    fn find_valid_breed_partner(
        mob: &Mob,
        partner_type_id: &str,
    ) -> Option<LivingEntity> {
        let nvle: NearestVisibleLivingEntities =
            mob.living.brain_get(&keys::nearest_visible_living_entities())?;
        nvle.find_closest(&|e| {
            entity_type_is(e, partner_type_id)
                && mob.living.entity.can_mate(&e.entity)
                && !Self::is_panicking(e)
        })
    }

    fn get_breed_target(mob: &Mob) -> Option<LivingEntity> {
        mob.living.brain_get(&keys::breed_target())
    }

    fn has_breed_target_of_right_type(&self, mob: &Mob) -> bool {
        Self::get_breed_target(mob)
            .map(|t| entity_type_is(&t, &self.partner_type_id))
            .unwrap_or(false)
    }
}

impl BehaviorControl for AnimalMakeLove {
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

    fn check_extra_start_conditions(
        &mut self,
        _level: &Arc<dyn LevelReader>,
        mob: &mut Mob,
    ) -> bool {
        mob.living.entity.is_in_love() && Self::find_valid_breed_partner(mob, &self.partner_type_id).is_some()
    }

    fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        let Some(partner) = Self::find_valid_breed_partner(mob, &self.partner_type_id) else {
            return;
        };
        mob.living.brain_set_value(&keys::breed_target(), partner.clone());
        partner.brain_set_value(&keys::breed_target(), mob.living.clone());
        behavior_utils::lock_gaze_and_walk_to_each_other_living(
            &mob.living,
            &partner,
            self.speed_modifier,
            self.close_enough_distance,
        );
        let i = 60 + mob.random.borrow_mut().next_int_bound(50);
        self.spawn_child_at_time = game_time + i as i64;
    }

    fn can_still_use(
        &mut self,
        _level: &Arc<dyn LevelReader>,
        mob: &mut Mob,
        game_time: i64,
    ) -> bool {
        if !self.has_breed_target_of_right_type(mob) {
            return false;
        }
        let Some(breed_target) = Self::get_breed_target(mob) else {
            return false;
        };
        breed_target.is_alive()
            && mob.living.entity.can_mate(&breed_target.entity)
            && mob
                .living
                .brain
                .as_deref()
                .map(|b| behavior_utils::entity_is_visible(b, &breed_target))
                .unwrap_or(false)
            && game_time <= self.spawn_child_at_time
            && !Self::is_panicking(&mob.living)
            && !Self::is_panicking(&breed_target)
    }

    fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        let Some(breed_target) = Self::get_breed_target(mob) else {
            return;
        };
        behavior_utils::lock_gaze_and_walk_to_each_other_living(
            &mob.living,
            &breed_target,
            self.speed_modifier,
            self.close_enough_distance,
        );
        if mob.living.entity.closer_than(&breed_target.entity, 3.0) {
            if game_time >= self.spawn_child_at_time {
                let level = mob.level();
                if let Some(level) = level {
                    let partner_uuid = breed_target.entity.uuid;
                    // `spawnChildFromBreeding` — &mut Mob.
                    let spawned = mob.spawn_child_from_breeding(&level, partner_uuid);
                    let _ = spawned;
                }
                mob.living.brain_erase(&keys::breed_target());
                breed_target.brain_erase(&keys::breed_target());
            }
        }
    }

    fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        mob.living.brain_erase(&keys::breed_target());
        mob.living.brain_erase(&keys::walk_target());
        mob.living.brain_erase(&keys::look_target());
        self.spawn_child_at_time = 0;
    }

    fn debug_string(&self) -> String {
        "AnimalMakeLove".to_string()
    }
}

// ===================== AnimalPanic =====================

/// `AnimalPanic` — IS_PANICKING registered + HURT_BY registered;
/// checkExtra: HURT_BY is panicCausingDamageTypes || IS_PANICKING.
/// panicCausingDamageTypes — `TagKey<DamageType>`-предикат.
pub struct AnimalPanic {
    base: BehaviorBase,
    speed_multiplier: f32,
    panic_causing_damage_types: Rc<dyn Fn(&Mob) -> crate::tags::tag_key::TagKey<crate::world::damagesource::damage_type::DamageType>>,
    position_getter: Rc<dyn Fn(&Mob) -> Option<Vec3>>,
}

impl AnimalPanic {
    /// `new(speedMultiplier)` — PANIC_CAUSES + LandRandomPos(5,4).
    pub fn new(speed_multiplier: f32) -> Self {
        Self::with_damage_types(
            speed_multiplier,
            Rc::new(|_mob| crate::tags::damage_type_tags::keys::panic_causes()),
        )
    }

    /// `new(speedMultiplier, panicCausingDamageTypes)`.
    pub fn with_damage_types(
        speed_multiplier: f32,
        panic_causing_damage_types: Rc<dyn Fn(&Mob) -> crate::tags::tag_key::TagKey<crate::world::damagesource::damage_type::DamageType>>,
    ) -> Self {
        let pg = panic_causing_damage_types.clone();
        let _ = pg;
        let pos_getter: Rc<dyn Fn(&Mob) -> Option<Vec3>> =
            Rc::new(|mob| land_random_pos::get_pos(mob, 5, 4));
        Self::with_all(speed_multiplier, panic_causing_damage_types, pos_getter)
    }

    /// Полная форма — positionGetter.
    pub fn with_all(
        speed_multiplier: f32,
        panic_causing_damage_types: Rc<dyn Fn(&Mob) -> crate::tags::tag_key::TagKey<crate::world::damagesource::damage_type::DamageType>>,
        position_getter: Rc<dyn Fn(&Mob) -> Option<Vec3>>,
    ) -> Self {
        let base = BehaviorBase::with_range(
            BehaviorBase::conditions(&[
                (keys::is_panicking(), MemoryStatus::Registered),
                (keys::hurt_by(), MemoryStatus::Registered),
            ]),
            100,
            120,
        );
        Self {
            base,
            speed_multiplier,
            panic_causing_damage_types,
            position_getter,
        }
    }

    /// `new(speedMultiplier, y)` — AirAndWaterRandomPos по viewVector.
    pub fn with_y(speed_multiplier: f32, y: i32) -> Self {
        let pos_getter: Rc<dyn Fn(&Mob) -> Option<Vec3>> = Rc::new(move |mob| {
            let view = mob.living.get_view_vector(0.0);
            air_and_water_random_pos::get_pos(mob, 5, 4, y, view.x, view.z, std::f64::consts::FRAC_PI_2)
        });
        Self::with_all(
            speed_multiplier,
            Rc::new(|_mob| crate::tags::damage_type_tags::keys::panic_causes()),
            pos_getter,
        )
    }

    /// `lookForWater` — isOnFire → ближайший water-блок
    /// (manhattan 5x1x5, squareOutSouthEast для ширины 2).
    fn look_for_water(&self, level: &Arc<dyn LevelReader>, mob: &Mob) -> Option<Vec3> {
        if !mob.living.entity.is_on_fire() {
            return None;
        }
        let block_pos = mob.block_position();
        // `!level.getBlockState(pos).getCollisionShape().isEmpty()` →
        // если внутри блока коллизия — воды нет. Упрощённо:
        // block_state collision через `collision_shape_unleveled`.
        if let Some(state) = level.get_block_state(block_pos) {
            if !state.block.properties.has_collision || !state.collision_shape_unleveled().is_empty() {
                return None;
            }
        }
        let width2 = mob.get_bb_width().ceil() as i32 == 2;
        BlockPos::find_closest_match(&block_pos, 5, 1, &mut |bp| {
            if width2 {
                BlockPos::square_out_south_east(bp).iter().all(|p| {
                    level
                        .get_fluid_state(*p)
                        .map(|f| f.is_water())
                        .unwrap_or(false)
                })
            } else {
                level
                    .get_fluid_state(*bp)
                    .map(|f| f.is_water())
                    .unwrap_or(false)
            }
        })
        .map(|bp| crate::world::phys::vec3::at_bottom_center_of(&bp.to_vec3i()))
    }
}

impl BehaviorControl for AnimalPanic {
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

    fn check_extra_start_conditions(
        &mut self,
        _level: &Arc<dyn LevelReader>,
        mob: &mut Mob,
    ) -> bool {
        let hurt_match = mob
            .living
            .brain_get::<crate::world::damagesource::damage_source::DamageSource>(
                &keys::hurt_by(),
            )
            .map(|ds| ds.is_tag(&(self.panic_causing_damage_types)(mob)))
            .unwrap_or(false);
        hurt_match || mob.living.brain_has(&keys::is_panicking())
    }

    fn can_still_use(
        &mut self,
        _level: &Arc<dyn LevelReader>,
        _mob: &mut Mob,
        _game_time: i64,
    ) -> bool {
        true
    }

    fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        mob.living.brain_set_value(&keys::is_panicking(), true);
        mob.living.brain_erase(&keys::walk_target());
        if let Some(nav) = mob.navigation.clone() {
            nav.borrow_mut().stop(mob);
        }
    }

    fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        mob.living.brain_erase(&keys::is_panicking());
    }

    fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        let done = mob
            .navigation
            .as_ref()
            .map(|n| n.borrow().is_done())
            .unwrap_or(true);
        if done {
            let panic_pos = self
                .look_for_water(level, mob)
                .or_else(|| (self.position_getter)(mob));
            if let Some(panic_pos) = panic_pos {
                let wt = WalkTarget::new_vec3(panic_pos, self.speed_multiplier, 0);
                mob.living.brain_set_value(&keys::walk_target(), wt);
            }
        }
    }

    fn debug_string(&self) -> String {
        "AnimalPanic".to_string()
    }
}

// ===================== BabyFollowAdult =====================

/// `BabyFollowAdult.create(followRange, speedModifier)` —
/// NEAREST_VISIBLE_ADULT present + LOOK registered + WALK absent;
/// !isBaby → false; в пределах [min, max+1] → WALK/LOOK на взрослого.
pub fn baby_follow_adult(
    follow_range_min: i32,
    follow_range_max: i32,
    speed_modifier: f32,
) -> PointBehavior {
    baby_follow_adult_with(
        follow_range_min,
        follow_range_max,
        Rc::new(move |_e| speed_modifier),
        keys::nearest_visible_adult(),
        false,
    )
}

/// Полная форма — `create(followRange, speedModifier fn,
/// nearestVisibleAdult, targetEyeHeight)`.
pub fn baby_follow_adult_with(
    follow_range_min: i32,
    follow_range_max: i32,
    speed_modifier: Rc<dyn Fn(&LivingEntity) -> f32>,
    nearest_visible_adult: MemoryModuleType,
    target_eye_height: bool,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (nearest_visible_adult.clone(), MemoryStatus::ValuePresent),
            (keys::look_target(), MemoryStatus::Registered),
            (keys::walk_target(), MemoryStatus::ValueAbsent),
        ]),
        move |_level, mob, _game_time| {
            if !mob.living.entity.is_baby() {
                return false;
            }
            let adult: Option<LivingEntity> =
                mob.living.brain_get(&nearest_visible_adult);
            let Some(adult) = adult else { return false };
            let max = follow_range_max + 1;
            let in_range = mob.living.entity.closer_than(&adult.entity, max as f64)
                && !mob
                    .living
                    .entity
                    .closer_than(&adult.entity, follow_range_min as f64);
            if !in_range {
                return false;
            }
            let wt = WalkTarget::of_tracker(
                tracker_of_full(&adult, target_eye_height, target_eye_height),
                speed_modifier(&mob.living),
                follow_range_min - 1,
            );
            mob.living.brain_set_value(
                &keys::look_target(),
                tracker_of_full(&adult, true, target_eye_height),
            );
            mob.living.brain_set_value(&keys::walk_target(), wt);
            true
        },
    )
}

// ===================== FollowTemptation =====================

/// `FollowTemptation` — TEMPTING_PLAYER present; timedOut всегда
/// false (переопределяем `timed_out`).
pub struct FollowTemptation {
    base: BehaviorBase,
    speed_modifier: Rc<dyn Fn(&LivingEntity) -> f32>,
    close_enough_distance: Rc<dyn Fn(&LivingEntity) -> f64>,
    look_in_the_eyes: bool,
}

impl FollowTemptation {
    /// `new(speedModifier)` — closeEnough=2.5, look=false.
    pub fn new(speed_modifier: impl Fn(&LivingEntity) -> f32 + 'static) -> Self {
        Self::with_close_enough(
            Rc::new(speed_modifier),
            Rc::new(|_| 2.5),
            false,
        )
    }

    /// `new(speedModifier, closeEnoughDistance)`.
    pub fn with_close_enough(
        speed_modifier: Rc<dyn Fn(&LivingEntity) -> f32>,
        close_enough_distance: Rc<dyn Fn(&LivingEntity) -> f64>,
        look_in_the_eyes: bool,
    ) -> Self {
        let base = BehaviorBase::with_range(
            BehaviorBase::conditions(&[
                (keys::look_target(), MemoryStatus::Registered),
                (keys::walk_target(), MemoryStatus::Registered),
                (keys::temptation_cooldown_ticks(), MemoryStatus::ValueAbsent),
                (keys::is_tempted(), MemoryStatus::ValueAbsent),
                (keys::tempting_player(), MemoryStatus::ValuePresent),
                (keys::breed_target(), MemoryStatus::ValueAbsent),
                (keys::is_panicking(), MemoryStatus::ValueAbsent),
            ]),
            0,
            0,
        );
        Self {
            base,
            speed_modifier,
            close_enough_distance,
            look_in_the_eyes,
        }
    }
}

impl BehaviorControl for FollowTemptation {
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

    /// `timedOut` — всегда false.
    fn timed_out(&self, _game_time: i64) -> bool {
        false
    }

    fn can_still_use(
        &mut self,
        _level: &Arc<dyn LevelReader>,
        mob: &mut Mob,
        _game_time: i64,
    ) -> bool {
        mob.living.brain_has(&keys::tempting_player())
            && !mob.living.brain_has(&keys::breed_target())
            && !mob.living.brain_has(&keys::is_panicking())
    }

    fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        mob.living.brain_set_value(&keys::is_tempted(), true);
    }

    fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        mob.living.brain_set_value(&keys::temptation_cooldown_ticks(), 100i32);
        mob.living.brain_erase(&keys::is_tempted());
        mob.living.brain_erase(&keys::walk_target());
        mob.living.brain_erase(&keys::look_target());
    }

    fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        let player: Option<LivingEntity> =
            mob.living.brain_get(&keys::tempting_player());
        let Some(player) = player else { return };
        mob.living
            .brain_set_value(&keys::look_target(), tracker_of(&player, true));
        let d = (self.close_enough_distance)(&mob.living);
        if mob.living.entity.distance_to_sqr(&player.entity) < d * d {
            mob.living.brain_erase(&keys::walk_target());
        } else {
            let wt = WalkTarget::of_tracker(
                tracker_of_full(&player, self.look_in_the_eyes, self.look_in_the_eyes),
                (self.speed_modifier)(&mob.living),
                2,
            );
            mob.living.brain_set_value(&keys::walk_target(), wt);
        }
    }

    fn debug_string(&self) -> String {
        "FollowTemptation".to_string()
    }
}

// ===================== PlayTagWithOtherKids =====================

/// `PlayTagWithOtherKids.create()` — VISIBLE_VILLAGER_BABIES present +
/// WALK absent + LOOK/INTERACTION registered; 1/10 шанс.
pub fn play_tag_with_other_kids() -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::visible_villager_babies(), MemoryStatus::ValuePresent),
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::look_target(), MemoryStatus::Registered),
            (keys::interaction_target(), MemoryStatus::Registered),
        ]),
        move |level, mob, _game_time| {
            if mob.random.borrow_mut().next_int_bound(10) != 0 {
                return false;
            }
            let kids: Option<Vec<LivingEntity>> =
                mob.living.brain_get(&keys::visible_villager_babies());
            let Some(kids) = kids else { return true };
            // isFriendChasingMe — INTERACTION_TARGET == mob.
            let self_uuid = mob.living.entity.uuid;
            let chased_by_me = kids
                .iter()
                .any(|kid| {
                    kid
                        .brain_get::<LivingEntity>(&keys::interaction_target())
                        .map(|t| t.entity.uuid == self_uuid)
                        .unwrap_or(false)
                });
            if chased_by_me {
                // бежим прочь — 10 попыток LandRandomPos в деревне.
                for _ in 0..10 {
                    if let Some(pos) = land_random_pos::get_pos(mob, 20, 8) {
                        if level.is_village(BlockPos::containing_vec3(&pos)) {
                            let wt = WalkTarget::new_vec3(pos, 0.6, 0);
                            mob.living.brain_set_value(&keys::walk_target(), wt);
                            break;
                        }
                    }
                }
                return true;
            }
            // findSomeoneBeingChased — chasers count 1..=5.
            let mut chased: Vec<LivingEntity> = Vec::new();
            for kid in &kids {
                if let Some(target) = kid
                    .brain_get::<LivingEntity>(&keys::interaction_target())
                {
                    if !chased.iter().any(|c| c.entity.uuid == target.entity.uuid) {
                        chased.push(target);
                    }
                }
            }
            let pick = chased.into_iter().next();
            let target = match pick {
                Some(t) => Some(t),
                None => kids.first().cloned(),
            };
            if let Some(kid) = target {
                // chaseKid.
                mob.living
                    .brain_set_value(&keys::interaction_target(), kid.clone());
                mob.living
                    .brain_set_value(&keys::look_target(), tracker_of(&kid, true));
                let wt = WalkTarget::of_tracker(
                    tracker_of(&kid, false),
                    0.6,
                    1,
                );
                mob.living.brain_set_value(&keys::walk_target(), wt);
            }
            true
        },
    )
}

// ===================== SetWalkTargetAwayFrom =====================

/// `SetWalkTargetAwayFrom.pos(memory, speed, desiredDistance, hasTarget)`
/// — walkTargetAwayFromMemory = BlockPos.
pub fn set_walk_target_away_from_pos(
    walk_target_away_from_memory: MemoryModuleType,
    speed_modifier: f32,
    desired_distance: i32,
    has_target: bool,
) -> PointBehavior {
    set_walk_target_away_from_impl(
        walk_target_away_from_memory,
        speed_modifier,
        desired_distance,
        has_target,
        Rc::new(|bp: &BlockPos| crate::world::phys::vec3::at_bottom_center_of(&bp.to_vec3i())),
    )
}

/// `SetWalkTargetAwayFrom.entity(memory, speed, desiredDistance,
/// hasTarget)` — memory = Entity.
pub fn set_walk_target_away_from_entity(
    walk_target_away_from_memory: MemoryModuleType,
    speed_modifier: f32,
    desired_distance: i32,
    has_target: bool,
) -> PointBehavior {
    set_walk_target_away_from_impl(
        walk_target_away_from_memory,
        speed_modifier,
        desired_distance,
        has_target,
        Rc::new(|e: &LivingEntity| e.entity.position()),
    )
}

fn set_walk_target_away_from_impl<T: Clone + 'static>(
    walk_target_away_from_memory: MemoryModuleType,
    speed_modifier: f32,
    desired_distance: i32,
    has_target: bool,
    to_position: Rc<dyn Fn(&T) -> Vec3>,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::walk_target(), MemoryStatus::Registered),
            (walk_target_away_from_memory.clone(), MemoryStatus::ValuePresent),
        ]),
        move |_level, mob, _game_time| {
            let walk_target: Option<WalkTarget> = mob.living.brain_get(&keys::walk_target());
            if walk_target.is_some() && !has_target {
                return false;
            }
            let Some(away): Option<T> = mob.living.brain_get(&walk_target_away_from_memory) else {
                return false;
            };
            let vec3 = mob.living.entity.position();
            let vec31 = to_position(&away);
            if !vec3.closer_than(&vec31, desired_distance as f64) {
                return false;
            }
            if let Some(wt) = &walk_target {
                if wt.get_speed_modifier() == speed_modifier {
                    let vec32 = wt.get_target().current_position().subtract_vec(&vec3);
                    let vec33 = vec31.subtract_vec(&vec3);
                    if vec32.dot_vec(&vec33) < 0.0 {
                        return false;
                    }
                }
            }
            for _ in 0..10 {
                if let Some(vec33) = land_random_pos::get_pos_away(mob, 16, 7, vec31) {
                    let wt = WalkTarget::new_vec3(vec33, speed_modifier, 0);
                    mob.living.brain_set_value(&keys::walk_target(), wt);
                    break;
                }
            }
            true
        },
    )
}

// ===================== SetEntityLookTargetSometimes =====================

/// `SetEntityLookTargetSometimes.create(maxDist, interval)` —
/// LOOK absent + NVLE present; Ticker(UniformInt).
pub fn set_entity_look_target_sometimes(
    max_dist: f32,
    interval_min: i32,
    interval_max: i32,
) -> PointBehavior {
    set_entity_look_target_sometimes_pred(max_dist, interval_min, interval_max, Rc::new(|_| true))
}

/// Полная форма — `create(maxDist, interval, canLookAtTarget)`.
pub fn set_entity_look_target_sometimes_pred(
    max_dist: f32,
    interval_min: i32,
    interval_max: i32,
    can_look_at_target: Rc<dyn Fn(&LivingEntity) -> bool>,
) -> PointBehavior {
    debug_assert!(interval_min > 1);
    let f = max_dist * max_dist;
    // `Ticker.ticksUntilNextStart`.
    let ticks_until_next = Rc::new(RefCell::new(0i32));
    let tun = ticks_until_next.clone();
    PointBehavior::new(
        conds(&[
            (keys::look_target(), MemoryStatus::ValueAbsent),
            (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
        ]),
        move |_level, mob, _game_time| {
            let nvle: Option<NearestVisibleLivingEntities> =
                mob.living.brain_get(&keys::nearest_visible_living_entities());
            let Some(nvle) = nvle else { return false };
            let found = nvle.find_closest(&|e| {
                can_look_at_target(e)
                    && e.distance_to_sqr(&mob.living.entity) <= f as f64
            });
            let Some(target) = found else { return false };
            // tickDownAndCheck(level.random).
            let mut t = tun.borrow_mut();
            let fired = if *t == 0 {
                *t = {
                    let mut r = mob.random.borrow_mut();
                    crate::util::valueproviders::uniform_int::UniformInt::of(interval_min, interval_max)
                        .sample(&mut **r)
                } - 1;
                false
            } else {
                *t -= 1;
                *t == 0
            };
            drop(t);
            if !fired {
                return false;
            }
            mob.living
                .brain_set_value(&keys::look_target(), tracker_of(&target, true));
            true
        },
    )
}

// ===================== StopAttackingIfTargetInvalid =====================

/// `StopAttackingIfTargetInvalid.create(canStopAttacking,
/// onStopAttacking, canGrowTired)` — ATTACK_TARGET present +
/// CANT_REACH registered.
pub fn stop_attacking_if_target_invalid(
    can_stop_attacking: Rc<dyn Fn(&Arc<dyn LevelReader>, &LivingEntity) -> bool>,
    on_stop_attacking: Rc<dyn Fn(&Arc<dyn LevelReader>, &mut Mob, &LivingEntity)>,
    can_grow_tired_of_trying_to_reach_target: bool,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::attack_target(), MemoryStatus::ValuePresent),
            (keys::cant_reach_walk_target_since(), MemoryStatus::Registered),
        ]),
        move |level, mob, game_time| {
            let target: Option<LivingEntity> =
                mob.living.brain_get(&keys::attack_target());
            let Some(target) = target else { return true };
            let cant_reach: Option<i64> =
                mob.living.brain_get(&keys::cant_reach_walk_target_since());
            let tired = can_grow_tired_of_trying_to_reach_target
                && cant_reach
                    .map(|t| level.game_time() - t > 200)
                    .unwrap_or(false);
            let same_level = mob
                .level()
                .map(|l| {
                    target
                        .entity
                        .level
                        .as_ref()
                        .map(|tl| std::ptr::eq(tl.as_ref(), l.as_ref()))
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            if mob.living.can_attack(&target)
                && !tired
                && target.is_alive()
                && same_level
                && !can_stop_attacking(level, &target)
            {
                return true;
            }
            on_stop_attacking(level, mob, &target);
            mob.living.brain_erase(&keys::attack_target());
            true
        },
    )
}

// ===================== LocateHidingPlace =====================

/// `LocateHidingPlace.create(radius, speedModifier, closeEnoughDist)` —
/// WALK absent + HOME/HIDING_PLACE/PATH/LOOK/BREED/INTERACTION
/// registered; HOME-POI рядом или random → HIDING_PLACE + WALK.
pub fn locate_hiding_place(
    radius: i32,
    speed_modifier: f32,
    close_enough_dist: i32,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::home(), MemoryStatus::Registered),
            (keys::hiding_place(), MemoryStatus::Registered),
            (keys::path(), MemoryStatus::Registered),
            (keys::look_target(), MemoryStatus::Registered),
            (keys::breed_target(), MemoryStatus::Registered),
            (keys::interaction_target(), MemoryStatus::Registered),
        ]),
        move |level, mob, _game_time| {
            let bp = mob.block_position();
            // find(HOME, ..., closeEnough+1, ANY) filtered by
            // closerToCenterThan → orElse getRandom(HOME, radius).
            let mut walk_pos: Option<BlockPos> = level
                .home_poi_near(bp, close_enough_dist + 1)
                .filter(|p| {
                    p.closer_to_center_than(
                        mob.living.entity.position(),
                        close_enough_dist as f64,
                    )
                });
            if walk_pos.is_none() {
                // `getRandom(HOME, ..., radius, random)` — упрощённо:
                // home_poi_near с большим радиусом.
                walk_pos = level.random_home_poi_near(bp, radius);
            }
            if walk_pos.is_none() {
                // orElse HOME memory → pos.
                let home: Option<GlobalPos> = mob.living.brain_get(&keys::home());
                walk_pos = home.map(|g| g.pos);
            }
            if let Some(walk_pos) = walk_pos {
                mob.living.brain_erase(&keys::path());
                mob.living.brain_erase(&keys::look_target());
                mob.living.brain_erase(&keys::breed_target());
                mob.living.brain_erase(&keys::interaction_target());
                let dim = level
                    .dimension_id()
                    .cloned()
                    .unwrap_or_else(|| crate::resources::identifier::Identifier::with_default_namespace("overworld").unwrap());
                let gpos = GlobalPos::of(
                    crate::resources::ResourceKey::<crate::world::level::Level>::create_with_registry_name(
                        crate::core::registry::registries::dimension(),
                        dim,
                    ),
                    walk_pos,
                );
                mob.living.brain_set_value(&keys::hiding_place(), gpos);
                if !walk_pos.closer_to_center_than(
                    mob.living.entity.position(),
                    close_enough_dist as f64,
                ) {
                    let wt = WalkTarget::new_block(walk_pos, speed_modifier, close_enough_dist);
                    mob.living.brain_set_value(&keys::walk_target(), wt);
                }
            }
            true
        },
    )
}
