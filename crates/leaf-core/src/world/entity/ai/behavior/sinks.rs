//! Перенос Behavior-sink'ов и walk/look-поведений:
//! `MoveToTargetSink`, `LookAtTargetSink`, `MoveToTargetSink`-семейство,
//! `RandomLookAround`, `InsideBrownianWalk`, `StayCloseToTarget`,
//! `DoNothing`, `SetEntityLookTarget`, `SetWalkTargetFromAttackTargetIfTargetOutOfReach`,
//! `StartAttacking`, `StartCelebratingIfTargetDead`, `Swim`,
//! `SetHiddenState`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::entity::schedule::activity::{activity_by_name, Activity};
use crate::util::mth;
use crate::util::valueproviders::int_provider::IntProvider;
use crate::world::entity::ai::behavior::behavior::{
    BehaviorBase, BehaviorControl, BehaviorStatus,
};
use crate::world::entity::ai::behavior::simple::{conds, PointBehavior};
use crate::world::entity::ai::memory::memory::{
    keys, EntityTracker, MemoryModuleType, MemoryStatus, NearestVisibleLivingEntities,
    PositionTracker, WalkTarget,
};
use crate::world::entity::ai::util::default_random_pos;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::pathfinder::path::Path;
use crate::world::phys::vec3::Vec3;

// ===================== MoveToTargetSink =====================

/// `MoveToTargetSink` — Behavior: WALK_TARGET → PATH → navigation.
pub struct MoveToTargetSink {
    base: BehaviorBase,
    /// `remainingCooldown`.
    remaining_cooldown: i32,
    /// `path`.
    path: Option<Path>,
    /// `lastTargetPos`.
    last_target_pos: Option<BlockPos>,
    /// `speedModifier`.
    speed_modifier: f32,
}

/// Хелперы для PositionTracker → BlockPos.
fn tracker_block_pos(tracker: &dyn PositionTracker) -> BlockPos {
    BlockPos::containing_vec3(&tracker.current_position())
}

fn tracker_is_spectator(tracker: &dyn PositionTracker) -> bool {
    // EntityTracker с entity.isSpectator — через Any не достать;
    // EntityTracker.current_position уже учитывает entity. Для
    // точности храним uuid в EntityTracker — см. memory.rs;
    // упрощённо: false (spectator-ветка редка).
    let _ = tracker;
    false
}

impl MoveToTargetSink {
    /// `new()` — 150..250.
    pub fn new() -> Self {
        Self::with_duration(150, 250)
    }

    /// `new(minDuration, maxDuration)`.
    pub fn with_duration(min_duration: i32, max_duration: i32) -> Self {
        Self {
            base: BehaviorBase::with_range(
                conds(&[
                    (keys::cant_reach_walk_target_since(), MemoryStatus::Registered),
                    (keys::path(), MemoryStatus::ValueAbsent),
                    (keys::walk_target(), MemoryStatus::ValuePresent),
                ]),
                min_duration,
                max_duration,
            ),
            remaining_cooldown: 0,
            path: None,
            last_target_pos: None,
            speed_modifier: 0.0,
        }
    }

    /// `reachedTarget(mob, target)`.
    fn reached_target(mob: &Mob, target: &WalkTarget) -> bool {
        let tp = tracker_block_pos(target.get_target());
        tp.dist_manhattan(mob.block_position()) <= target.get_close_enough_dist()
    }

    /// `tryComputePath(mob, target, time)`.
    fn try_compute_path(
        &mut self,
        mob: &mut Mob,
        target: &WalkTarget,
        time: i64,
    ) -> bool {
        let block_pos = tracker_block_pos(target.get_target());
        let nav = mob.navigation.clone();
        self.path = nav.as_ref().and_then(|n| {
            n.borrow_mut().create_path_pos(mob, block_pos, 0)
        });
        self.speed_modifier = target.get_speed_modifier();
        if Self::reached_target(mob, target) {
            mob.living.brain_erase(&keys::cant_reach_walk_target_since());
        } else {
            let path_can_reach = self.path.as_ref().map(|p| p.can_reach()).unwrap_or(false);
            if path_can_reach {
                mob.living.brain_erase(&keys::cant_reach_walk_target_since());
            } else if !mob.living.brain_has(&keys::cant_reach_walk_target_since()) {
                mob.living.brain_set_value(&keys::cant_reach_walk_target_since(), time);
            }
            if self.path.is_some() {
                return true;
            }
            // fallback: getPosTowards в случайную точку рядом с целью.
            let pos_towards = default_random_pos::get_pos_towards(
                mob,
                10,
                7,
                crate::world::phys::vec3::at_bottom_center_of(&block_pos.to_vec3i()),
                std::f64::consts::FRAC_PI_2,
            );
            if let Some(p) = pos_towards {
                self.path = nav.as_ref().and_then(|n| {
                    n.borrow_mut().create_path(mob, p.x, p.y, p.z, 0)
                });
                return self.path.is_some();
            }
        }
        false
    }
}

impl BehaviorControl for MoveToTargetSink {
    fn status(&self) -> BehaviorStatus {
        self.base.status
    }
    fn set_status(&mut self, status: BehaviorStatus) {
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

    /// `checkExtraStartConditions`.
    fn check_extra_start_conditions(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
        if self.remaining_cooldown > 0 {
            self.remaining_cooldown -= 1;
            return false;
        }
        let walk_target: Option<WalkTarget> = mob.living.brain_get(&keys::walk_target());
        let Some(walk_target) = walk_target else { return false };
        let reached = Self::reached_target(mob, &walk_target);
        if !reached && self.try_compute_path(mob, &walk_target, level.game_time()) {
            self.last_target_pos = Some(tracker_block_pos(walk_target.get_target()));
            true
        } else {
            mob.living.brain_erase(&keys::walk_target());
            if reached {
                mob.living.brain_erase(&keys::cant_reach_walk_target_since());
            }
            false
        }
    }

    /// `canStillUse`.
    fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
        if self.path.is_some() && self.last_target_pos.is_some() {
            let memory: Option<WalkTarget> = mob.living.brain_get(&keys::walk_target());
            let is_spectator = memory
                .as_ref()
                .map(|wt| tracker_is_spectator(wt.get_target()))
                .unwrap_or(false);
            let nav_done = mob
                .navigation
                .as_ref()
                .map(|n| n.borrow().is_done())
                .unwrap_or(true);
            memory.is_some() && !nav_done && !Self::reached_target(mob, memory.as_ref().unwrap()) && !is_spectator
        } else {
            false
        }
    }

    /// `start` — PATH memory + navigation.moveTo.
    fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        mob.living.brain_set(&keys::path(), self.path.clone());
        if let Some(nav) = &mob.navigation {
            nav.borrow_mut().move_to_path(mob, self.path.clone(), self.speed_modifier as f64);
        }
    }

    /// `tick` — синхронизация PATH memory + recompute при сдвиге цели.
    fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        let nav_path = mob
            .navigation
            .as_ref()
            .and_then(|n| n.borrow().path().cloned());
        if self.path != nav_path {
            self.path = nav_path.clone();
            mob.living.brain_set(&keys::path(), nav_path);
        }
        if let (Some(_), Some(last)) = (&self.path, self.last_target_pos) {
            let walk_target: Option<WalkTarget> = mob.living.brain_get(&keys::walk_target());
            if let Some(walk_target) = walk_target {
                let cur = tracker_block_pos(walk_target.get_target());
                if cur.dist_sqr(last) as f64 > 4.0
                    && self.try_compute_path(mob, &walk_target, level.game_time())
                {
                    self.last_target_pos = Some(tracker_block_pos(walk_target.get_target()));
                    self.start(level, mob, game_time);
                }
            }
        }
    }

    /// `stop`.
    fn stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        let reached = mob
            .living
            .brain_get::<WalkTarget>(&keys::walk_target())
            .map(|wt| Self::reached_target(mob, &wt))
            .unwrap_or(false);
        let stuck = mob
            .navigation
            .as_ref()
            .map(|n| n.borrow().is_stuck())
            .unwrap_or(false);
        if mob.living.brain_has(&keys::walk_target()) && !reached && stuck {
            self.remaining_cooldown = {
                let mut r = mob.random.borrow_mut();
                r.next_int_bound(40)
            };
        }
        let nav = mob.navigation.clone();
        if let Some(nav) = nav {
            nav.borrow_mut().stop(mob);
        }
        mob.living.brain_erase(&keys::walk_target());
        mob.living.brain_erase(&keys::path());
        self.path = None;
    }
}

impl Default for MoveToTargetSink {
    fn default() -> Self {
        Self::new()
    }
}

// ===================== LookAtTargetSink =====================

/// `LookAtTargetSink` — LOOK_TARGET present → lookControl.setLookAt.
pub struct LookAtTargetSink {
    base: BehaviorBase,
}

impl LookAtTargetSink {
    /// `new(minDuration, maxDuration)`.
    pub fn new(min_duration: i32, max_duration: i32) -> Self {
        Self {
            base: BehaviorBase::with_range(
                conds(&[(keys::look_target(), MemoryStatus::ValuePresent)]),
                min_duration,
                max_duration,
            ),
        }
    }
}

impl BehaviorControl for LookAtTargetSink {
    fn status(&self) -> BehaviorStatus {
        self.base.status
    }
    fn set_status(&mut self, status: BehaviorStatus) {
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

    /// canStillUse: LOOK_TARGET present && isVisibleBy(entity).
    fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
        let tracker: Option<Rc<dyn PositionTracker>> = mob.living.brain_get(&keys::look_target());
        tracker
            .map(|t| t.is_visible_by(&mob.living))
            .unwrap_or(false)
    }

    /// stop: eraseMemory(LOOK_TARGET).
    fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        mob.living.brain_erase(&keys::look_target());
    }

    /// tick: lookControl.setLookAt(currentPosition).
    fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        let tracker: Option<Rc<dyn PositionTracker>> = mob.living.brain_get(&keys::look_target());
        if let Some(t) = tracker {
            let pos = t.current_position();
            mob.look_control_set_look_at(pos);
        }
    }
}

// ===================== RandomLookAround =====================

/// `RandomLookAround` — interval IntProvider, look в случайном
/// направлении → LOOK_TARGET + GAZE_COOLDOWN_TICKS.
pub struct RandomLookAround {
    base: BehaviorBase,
    /// `interval` — IntProvider (хранится как Arc).
    interval: Rc<dyn IntProvider>,
    /// `maxYaw`/`minPitch`/`pitchRange`.
    max_yaw: f32,
    min_pitch: f32,
    pitch_range: f32,
}

impl RandomLookAround {
    /// `new(interval, maxYaw, minPitch, maxPitch)`.
    pub fn new(interval: Rc<dyn IntProvider>, max_yaw: f32, min_pitch: f32, max_pitch: f32) -> Self {
        assert!(min_pitch <= max_pitch, "Minimum pitch is larger than maximum pitch!");
        Self {
            base: BehaviorBase::new(conds(&[
                (keys::look_target(), MemoryStatus::ValueAbsent),
                (keys::gaze_cooldown_ticks(), MemoryStatus::ValueAbsent),
            ])),
            interval,
            max_yaw,
            min_pitch,
            pitch_range: max_pitch - min_pitch,
        }
    }
}

impl BehaviorControl for RandomLookAround {
    fn status(&self) -> BehaviorStatus {
        self.base.status
    }
    fn set_status(&mut self, status: BehaviorStatus) {
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

    /// start: случайный pitch/yaw → BlockPosTracker(eyePos+dir),
    /// GAZE_COOLDOWN_TICKS = interval.sample(random).
    fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        let (pitch, yaw_add) = {
            let mut r = mob.random.borrow_mut();
            let pitch = mth::clamp_f32(
                r.next_float() * self.pitch_range + self.min_pitch,
                -90.0,
                90.0,
            );
            let yaw_add = 2.0 * r.next_float() * self.max_yaw - self.max_yaw;
            (pitch, yaw_add)
        };
        let yaw = mth::wrap_degrees_f32(mob.living.entity.get_y_rot() + yaw_add);
        let dir = Vec3::direction_from_rotation(pitch, yaw);
        let eye = mob.living.entity.get_eye_position().add_vec(&dir);
        let tracker = crate::world::entity::ai::memory::memory::BlockPosTracker::of_vec3(eye);
        mob.living.brain_set_value(
            &keys::look_target(),
            Rc::new(tracker) as Rc<dyn PositionTracker>,
        );
        let cooldown = {
            let mut r = mob.random.borrow_mut();
            self.interval.sample(&mut **r)
        };
        mob.living.brain_set_value(&keys::gaze_cooldown_ticks(), cooldown);
    }
}

// ===================== InsideBrownianWalk =====================

/// `InsideBrownianWalk.create(speedModifier)` — WALK_TARGET absent;
/// canSeeSky → false; иначе случайный соседний блок без неба.
pub fn inside_brownian_walk(speed_modifier: f32) -> PointBehavior {
    PointBehavior::new(
        conds(&[(keys::walk_target(), MemoryStatus::ValueAbsent)]),
        move |level, mob, _game_time| {
            if level.can_see_sky(mob.block_position()) {
                return false;
            }
            let center = mob.block_position();
            // 3x3x3 вокруг, без неба, loadedAndEntityCanStandOn,
            // noCollision(mob) — собираем и выбираем первый
            // (vanilla shuffle → детерминированно по индексу через random).
            let mut candidates: Vec<BlockPos> = Vec::new();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let pos = BlockPos::new(center.x + dx, center.y + dy, center.z + dz);
                        if !level.can_see_sky(pos)
                            && level.loaded_and_entity_can_stand_on(pos, mob)
                        {
                            candidates.push(pos);
                        }
                    }
                }
            }
            if candidates.is_empty() {
                return true;
            }
            // Collections.shuffle(list) — Fisher-Yates через mob.random.
            {
                let mut r = mob.random.borrow_mut();
                for i in (1..candidates.len()).rev() {
                    let j = r.next_int_bound((i + 1) as i32) as usize;
                    candidates.swap(i, j);
                }
            }
            let bb = mob.living.entity.get_bounding_box();
            let no_collision = level.no_collision(&bb);
            let chosen = candidates
                .into_iter()
                .find(|pos| no_collision || level.no_collision(&bb))
                .or_else(|| candidates_fallback(mob, level));
            if let Some(pos) = chosen {
                let wt = WalkTarget::new_block(pos, speed_modifier, 0);
                mob.living.brain_set_value(&keys::walk_target(), wt);
            }
            true
        },
    )
}

/// Fallback для noCollision-проверки (vanilla фильтрует по
/// `level.noCollision(mob)` — bounding box в точке; упрощённо —
/// берём первый кандидат).
fn candidates_fallback(mob: &Mob, _level: &Arc<dyn LevelReader>) -> Option<BlockPos> {
    let _ = mob;
    None
}

// ===================== StayCloseToTarget =====================

/// `targetPositionGetter` — PositionTracker из памяти/сущности.
pub type TargetPositionGetter =
    Rc<dyn Fn(&Mob) -> Option<Rc<dyn PositionTracker>>>;

/// `StayCloseToTarget.create(getter, predicate, closeEnoughDist,
/// tooClose, speedModifier)`.
pub fn stay_close_to_target(
    target_position_getter: TargetPositionGetter,
    predicate: impl Fn(&Mob) -> bool + 'static,
    close_enough_dist: i32,
    too_close: i32,
    speed_modifier: f32,
) -> PointBehavior {
    let pred = Rc::new(predicate);
    PointBehavior::new(
        conds(&[
            (keys::look_target(), MemoryStatus::Registered),
            (keys::walk_target(), MemoryStatus::Registered),
        ]),
        move |_level, mob, _game_time| {
            let optional = target_position_getter(mob);
            if let Some(tracker) = optional {
                if !pred(mob) {
                    return false;
                }
                let pos = tracker.current_position();
                if mob.living.entity.position().closer_than(&pos, too_close as f64) {
                    return false;
                }
                mob.living.brain_set_value(
                    &keys::look_target(),
                    tracker.clone(),
                );
                let wt = WalkTarget::of_tracker(tracker, speed_modifier, close_enough_dist);
                mob.living.brain_set_value(&keys::walk_target(), wt);
                true
            } else {
                false
            }
        },
    )
}

// ===================== DoNothing =====================

/// `DoNothing` — RUNNING на random[min,max] тиков, tickOrStop по
/// endTimestamp.
pub struct DoNothing {
    min_duration: i32,
    max_duration: i32,
    status: BehaviorStatus,
    end_timestamp: i64,
}

impl DoNothing {
    /// `new(minDuration, maxDuration)`.
    pub fn new(min_duration: i32, max_duration: i32) -> Self {
        Self {
            min_duration,
            max_duration,
            status: BehaviorStatus::Stopped,
            end_timestamp: 0,
        }
    }
}

impl BehaviorControl for DoNothing {
    fn status(&self) -> BehaviorStatus {
        self.status
    }
    fn set_status(&mut self, status: BehaviorStatus) {
        self.status = status;
    }
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
        // Пустая — статический EMPTY через thread_local не нужен:
        // возвращаем ссылку на статическую пустую map.
        static EMPTY: std::sync::OnceLock<HashMap<MemoryModuleType, MemoryStatus>> =
            std::sync::OnceLock::new();
        EMPTY.get_or_init(HashMap::new)
    }
    fn durations(&self) -> (i32, i32) {
        (0, 0)
    }
    fn end_timestamp(&self) -> i64 {
        self.end_timestamp
    }
    fn set_end_timestamp(&mut self, timestamp: i64) {
        self.end_timestamp = timestamp;
    }

    /// tryStart — всегда RUNNING, duration = min + rand(max+1-min).
    fn try_start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) -> bool {
        self.status = BehaviorStatus::Running;
        let i = {
            let mut r = mob.random.borrow_mut();
            self.min_duration + r.next_int_bound(self.max_duration + 1 - self.min_duration)
        };
        self.end_timestamp = game_time + i as i64;
        true
    }

    /// tickOrStop — gameTime > endTimestamp → doStop.
    fn tick_or_stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        if game_time > self.end_timestamp {
            self.do_stop(_level, mob, game_time);
        }
    }

    /// doStop — STOPPED.
    fn do_stop(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) {
        self.status = BehaviorStatus::Stopped;
    }

    fn debug_string(&self) -> String {
        "DoNothing".to_string()
    }
}

// ===================== SetEntityLookTarget =====================

/// `SetEntityLookTarget.create(canLootAtTarget, maxDist)` —
/// LOOK_TARGET absent + NVLE present → ближайшая подходящая.
pub fn set_entity_look_target(
    can_loot_at_target: impl Fn(&LivingEntity) -> bool + 'static,
    max_dist: f32,
) -> PointBehavior {
    let pred = Rc::new(can_loot_at_target);
    let f = max_dist * max_dist;
    PointBehavior::new(
        conds(&[
            (keys::look_target(), MemoryStatus::ValueAbsent),
            (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
        ]),
        move |_level, mob, _game_time| {
            let nvle: Option<NearestVisibleLivingEntities> =
                mob.living.brain_get(&keys::nearest_visible_living_entities());
            let found = nvle.and_then(|nv| {
                nv.find_closest(&|target| {
                    pred(target)
                        && target.distance_to_sqr(&mob.living.entity) <= f as f64
                        && !mob.living.entity.has_passenger(&target.entity)
                })
            });
            if let Some(target) = found {
                let tracker = EntityTracker::new(target.entity.clone(), true);
                mob.living.brain_set_value(
                    &keys::look_target(),
                    Rc::new(tracker) as Rc<dyn PositionTracker>,
                );
                true
            } else {
                false
            }
        },
    )
}

/// `create(entityTypeId, maxDist)` — фильтр по типу.
pub fn set_entity_look_target_type(entity_type_id: &str, max_dist: f32) -> PointBehavior {
    let id = entity_type_id.to_string();
    set_entity_look_target(
        move |e| crate::world::entity::ai::sensing::sensor::entity_type_is(e, &id),
        max_dist,
    )
}

/// `create(maxDist)` — без фильтра.
pub fn set_entity_look_target_any(max_dist: f32) -> PointBehavior {
    set_entity_look_target(|_| true, max_dist)
}

// ===================== SetWalkTargetFromAttackTargetIfTargetOutOfReach =====================

/// `SetWalkTargetFromAttackTargetIfTargetOutOfReach.create(speedModifier)`.
/// WALK_TARGET/LOOK_TARGET registered + ATTACK_TARGET present +
/// NVLE registered: если NVLE содержит цель и она в attack-range
/// (buffer=1) → erase WALK_TARGET; иначе look+walk на цель.
pub fn set_walk_target_from_attack_target_if_out_of_reach(
    speed_modifier: impl Fn(&LivingEntity) -> f32 + 'static,
) -> PointBehavior {
    let speed_fn = Rc::new(speed_modifier);
    PointBehavior::new(
        conds(&[
            (keys::walk_target(), MemoryStatus::Registered),
            (keys::look_target(), MemoryStatus::Registered),
            (keys::attack_target(), MemoryStatus::ValuePresent),
            (keys::nearest_visible_living_entities(), MemoryStatus::Registered),
        ]),
        move |_level, mob, _game_time| {
            let target: Option<LivingEntity> = mob.living.brain_get(&keys::attack_target());
            let Some(target) = target else { return true };
            let nvle: Option<NearestVisibleLivingEntities> =
                mob.living.brain_get(&keys::nearest_visible_living_entities());
            let contains = nvle
                .map(|nv| nv.nearby_entities.iter().any(|e| e.get_uuid() == target.get_uuid()))
                .unwrap_or(false);
            let within = crate::world::entity::ai::behavior::behavior_utils::is_within_attack_range(
                mob, &target, 1,
            );
            if contains && within {
                mob.living.brain_erase(&keys::walk_target());
            } else {
                let look = EntityTracker::new(target.entity.clone(), true);
                let walk = EntityTracker::new(target.entity.clone(), false);
                mob.living.brain_set_value(
                    &keys::look_target(),
                    Rc::new(look) as Rc<dyn PositionTracker>,
                );
                let sm = speed_fn(&mob.living);
                mob.living.brain_set_value(
                    &keys::walk_target(),
                    WalkTarget::of_tracker(
                        Rc::new(walk) as Rc<dyn PositionTracker>,
                        sm,
                        0,
                    ),
                );
            }
            true
        },
    )
}

// ===================== StartAttacking =====================

/// `StartAttacking.TargetFinder` — поиск цели.
pub type StartAttackingTargetFinder =
    Rc<dyn Fn(&Arc<dyn LevelReader>, &Mob) -> Option<LivingEntity>>;

/// `StartAttacking.StartAttackingCondition`.
pub type StartAttackingCondition = Rc<dyn Fn(&Arc<dyn LevelReader>, &Mob) -> bool>;

/// `StartAttacking.create(targetFinder)` — condition=true.
pub fn start_attacking(target_finder: StartAttackingTargetFinder) -> PointBehavior {
    start_attacking_with(Rc::new(|_level, _mob| true), target_finder)
}

/// `StartAttacking.create(condition, targetFinder)` —
/// ATTACK_TARGET absent + CANT_REACH registered; condition →
/// targetFinder → canAttack → set ATTACK_TARGET + erase CANT_REACH.
pub fn start_attacking_with(
    condition: StartAttackingCondition,
    target_finder: StartAttackingTargetFinder,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::attack_target(), MemoryStatus::ValueAbsent),
            (keys::cant_reach_walk_target_since(), MemoryStatus::Registered),
        ]),
        move |level, mob, _game_time| {
            if !condition(level, mob) {
                return false;
            }
            let Some(target) = target_finder(level, mob) else {
                return false;
            };
            if !mob.living.can_attack(&target) {
                return false;
            }
            mob.living.brain_set_value(&keys::attack_target(), target);
            mob.living.brain_erase(&keys::cant_reach_walk_target_since());
            true
        },
    )
}

// ===================== StartCelebratingIfTargetDead =====================
// ===================== StartCelebratingIfTargetDead =====================

/// `StartCelebratingIfTargetDead.create()` — ATTACK_TARGET present;
/// target мёртв → CELEBRATE activity + erase ATTACK_TARGET.
pub fn start_celebrating_if_target_dead() -> PointBehavior {
    PointBehavior::new(
        conds(&[(keys::attack_target(), MemoryStatus::ValuePresent)]),
        |level, mob, _game_time| {
            let target: Option<LivingEntity> = mob.living.brain_get(&keys::attack_target());
            let Some(target) = target else { return true };
            if target.is_dead_or_dying() || !target.is_alive() {
                if let Some(brain) = &mob.living.brain {
                    if let Some(act) = activity_by_name("celebrate") {
                        brain.set_active_activity_if_possible(act);
                    }
                }
                mob.living.brain_erase(&keys::attack_target());
            }
            true
        },
    )
}

// ===================== Swim =====================

/// `Swim` — Behavior: shouldSwim check; tick → jumpControl.jump
/// с вероятностью chance.
pub struct Swim {
    base: BehaviorBase,
    /// `chance`.
    chance: f32,
}

impl Swim {
    /// `new(chance)`.
    pub fn new(chance: f32) -> Self {
        Self {
            base: BehaviorBase::new(HashMap::new()),
            chance,
        }
    }

    /// `shouldSwim(mob)` — static.
    pub fn should_swim(mob: &Mob) -> bool {
        // isInWater && fluidHeight(WATER) > fluidJumpThreshold
        // || isInLava — упрощённо через was_touching_water/is_in_lava хуки.
        (mob.living.entity.is_in_water()
            && mob.living.entity.fluid_height_water() > mob.living.entity.fluid_jump_threshold())
            || mob.living.entity.is_in_lava()
    }
}

impl BehaviorControl for Swim {
    fn status(&self) -> BehaviorStatus {
        self.base.status
    }
    fn set_status(&mut self, status: BehaviorStatus) {
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

    /// checkExtraStartConditions → shouldSwim.
    fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
        Self::should_swim(mob)
    }

    /// canStillUse → shouldSwim.
    fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
        Self::should_swim(mob)
    }

    /// tick — random < chance → jump.
    fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        if mob.random.borrow_mut().next_float() < self.chance {
            mob.jump_control_jump();
        }
    }
}

// ===================== SetHiddenState =====================

/// `SetHiddenState.create(stayHiddenSeconds, closeEnoughDist)` —
/// HIDING_PLACE + HEARD_BELL_TIME present; считаем тики внутри
/// closeEnoughDist пока не timeout → HIDE activity.
pub fn set_hidden_state(stay_hidden_seconds: i32, close_enough_dist: i32) -> PointBehavior {
    let hide_ticks = (stay_hidden_seconds * 20) as i64;
    let inside_ticks = Rc::new(RefCell::new(0i32));
    let it = inside_ticks.clone();
    PointBehavior::new(
        conds(&[
            (keys::hiding_place(), MemoryStatus::ValuePresent),
            (keys::heard_bell_time(), MemoryStatus::ValuePresent),
        ]),
        move |level, mob, game_time| {
            let heard: Option<i64> = mob.living.brain_get(&keys::heard_bell_time());
            let Some(heard) = heard else { return true };
            let flag = heard + 300 <= game_time;
            let mut inside = it.borrow_mut();
            if *inside as i64 <= hide_ticks && !flag {
                // HIDING_PLACE → PositionTracker/BlockPosTracker.
                let tracker: Option<Rc<dyn PositionTracker>> =
                    mob.living.brain_get(&keys::hiding_place());
                if let Some(t) = tracker {
                    let pos = t.current_position();
                    let bp = BlockPos::containing_vec3(&pos);
                    if bp.dist_manhattan(mob.block_position()) <= close_enough_dist {
                        *inside += 1;
                    }
                }
            } else {
                drop(inside);
                // timeout → HIDE activity (Java ставит activity через
                // brain.updateActivityFromSchedule? Нет — setHiddenState
                // просто считает; activity ставится отдельно).
                if let Some(brain) = &mob.living.brain {
                    if let Some(act) = activity_by_name("hide") {
                        brain.set_active_activity_if_possible(act);
                    }
                }
            }
            true
        },
    )
}
