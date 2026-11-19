//! Перенос POI/villager поведений: `AcquirePoi`,
//! `GoToPotentialJobSite`, `SetClosestHomeAsWalkTarget`,
//! `ResetProfession`, `AssignProfessionFromJobSite`.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::global_pos::GlobalPos;
use crate::core::holder::Holder;
use crate::world::entity::ai::behavior::behavior::{BehaviorBase, BehaviorControl, BehaviorStatus};
use crate::world::entity::ai::behavior::simple::{conds, PointBehavior};
use crate::world::entity::ai::memory::memory::{self, keys, MemoryModuleType, MemoryStatus, WalkTarget};
use crate::world::entity::ai::village::poi::poi_manager::{Occupancy, PoiManager, PoiTypePredicate};
use crate::world::entity::ai::village::poi::poi_type::PoiType;
use crate::world::entity::ai::village::poi::poi_types;
use crate::world::entity::mob::Mob;
use crate::world::entity::npc::villager::villager;
use crate::world::entity::npc::villager::villager_data;
use crate::world::entity::npc::villager::villager_profession;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::pathfinder::path::Path;

/// `AcquirePoi.findPathToPois` — путь к ближайшему из набора
/// POI-позиций с `maxValidRange` (максимум validRange типов).
pub fn find_path_to_pois(
    mob: &mut Mob,
    poi_positions: &HashSet<(Holder<PoiType>, BlockPos)>,
) -> Option<Path> {
    if poi_positions.is_empty() {
        return None;
    }
    let mut targets = HashSet::new();
    let mut i = 1;
    for (holder, pos) in poi_positions {
        i = i.max(crate::world::entity::ai::village::poi::poi_manager::valid_range_of(holder));
        targets.insert(*pos);
    }
    mob.navigation
        .as_ref()
        .and_then(|n| n.borrow_mut().create_path_set(mob, &targets, i))
}

/// `AcquirePoi.JitteredLinearRetry` — backoff для неудачных
/// POI-позиций.
#[derive(Clone)]
pub struct JitteredLinearRetry {
    previous_attempt_timestamp: i64,
    next_scheduled_attempt_timestamp: i64,
    current_delay: i32,
}

impl JitteredLinearRetry {
    const MIN_INTERVAL_INCREASE: i32 = 40;
    const MAX_INTERVAL_INCREASE: i32 = 80;
    const MAX_RETRY_PATHFINDING_INTERVAL: i32 = 400;

    /// `new(random, timestamp)` — markAttempt.
    pub fn new(mob_random: &Rc<RefCell<Box<dyn crate::random::RandomSource>>>, timestamp: i64) -> Self {
        let mut r = Self {
            previous_attempt_timestamp: 0,
            next_scheduled_attempt_timestamp: 0,
            current_delay: 0,
        };
        r.mark_attempt(mob_random, timestamp);
        r
    }

    /// `markAttempt`.
    pub fn mark_attempt(&mut self, mob_random: &Rc<RefCell<Box<dyn crate::random::RandomSource>>>, timestamp: i64) {
        self.previous_attempt_timestamp = timestamp;
        let mut r = mob_random.borrow_mut();
        let i = self.current_delay
            + r.next_int_bound(Self::MAX_INTERVAL_INCREASE - Self::MIN_INTERVAL_INCREASE + 1)
            + Self::MIN_INTERVAL_INCREASE;
        self.current_delay = i.min(Self::MAX_RETRY_PATHFINDING_INTERVAL);
        self.next_scheduled_attempt_timestamp = timestamp + self.current_delay as i64;
    }

    /// `isStillValid`.
    pub fn is_still_valid(&self, timestamp: i64) -> bool {
        timestamp - self.previous_attempt_timestamp < Self::MAX_RETRY_PATHFINDING_INTERVAL as i64
    }

    /// `shouldRetry`.
    pub fn should_retry(&self, timestamp: i64) -> bool {
        timestamp >= self.next_scheduled_attempt_timestamp
    }
}

/// Состояние `AcquirePoi` (замыкания + retry-map).
pub struct AcquirePoiState {
    /// `mutableLong` — следующий gameTime сканирования.
    pub next_scan: Rc<RefCell<i64>>,
    /// `map` — pos.asLong() → JitteredLinearRetry.
    pub retries: Rc<RefCell<HashMap<i64, JitteredLinearRetry>>>,
}

/// `AcquirePoi.create(acquirablePois, acquiringMemory,
/// existingAbsentMemory, onlyIfAdult, entityEventId, predicate)`.
/// `predicate` — доп. фильтр (level, pos) → bool.
#[allow(clippy::too_many_arguments)]
pub fn acquire_poi(
    acquirable_pois: Rc<dyn Fn(&Holder<PoiType>) -> bool + Send + Sync>,
    existing_absent_memory: MemoryModuleType,
    acquiring_memory: MemoryModuleType,
    only_if_adult: bool,
    entity_event_id: Option<u8>,
    predicate: Rc<dyn Fn(&Arc<dyn LevelReader>, BlockPos) -> bool + Send + Sync>,
) -> PointBehavior {
    acquire_poi_with_state(
        Rc::new(AcquirePoiState {
            next_scan: Rc::new(RefCell::new(0)),
            retries: Rc::new(RefCell::new(HashMap::new())),
        }),
        acquirable_pois,
        existing_absent_memory,
        acquiring_memory,
        only_if_adult,
        entity_event_id,
        predicate,
    )
}

/// Вариант с явным state (для shared между create-вызовами).
#[allow(clippy::too_many_arguments)]
pub fn acquire_poi_with_state(
    state: Rc<AcquirePoiState>,
    acquirable_pois: Rc<dyn Fn(&Holder<PoiType>) -> bool + Send + Sync>,
    existing_absent_memory: MemoryModuleType,
    acquiring_memory: MemoryModuleType,
    only_if_adult: bool,
    entity_event_id: Option<u8>,
    predicate: Rc<dyn Fn(&Arc<dyn LevelReader>, BlockPos) -> bool + Send + Sync>,
) -> PointBehavior {
    let state2 = state.clone();
    let acquiring_memory_inner = acquiring_memory.clone();
    let trigger = move |level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64| {
        if only_if_adult && mob.living.entity.is_baby() {
            return false;
        }
        let Some(pm) = level.poi_manager() else { return false };
        {
            let mut next = state2.next_scan.borrow_mut();
            if *next == 0 {
                let mut r = mob.random.borrow_mut();
                *next = level.game_time() + r.next_int_bound(20) as i64;
                return false;
            }
            if level.game_time() < *next {
                return false;
            }
            let mut r = mob.random.borrow_mut();
            *next = game_time + 20 + r.next_int_bound(20) as i64;
        }
        // чистка retries.
        {
            let mut retries = state2.retries.borrow_mut();
            retries.retain(|_, v| v.is_still_valid(game_time));
        }
        let pos_predicate = {
            let retries = state2.retries.clone();
            let random = mob.random.clone();
            move |bp: BlockPos| -> bool {
                let key = bp.as_long();
                let mut retries = retries.borrow_mut();
                match retries.get_mut(&key) {
                    None => true,
                    Some(v) => {
                        if !v.should_retry(game_time) {
                            false
                        } else {
                            v.mark_attempt(&random, game_time);
                            true
                        }
                    }
                }
            }
        };
        let center = mob.block_position();
        let mut set: HashSet<(Holder<PoiType>, BlockPos)> = HashSet::new();
        for pair in pm.find_all_closest_first_with_type(
            &*acquirable_pois,
            &|bp| pos_predicate(bp),
            center,
            48,
            Occupancy::HasSpace,
        ) {
            if set.len() >= 5 {
                break;
            }
            if predicate(level, pair.1) {
                set.insert(pair);
            }
        }
        let path = find_path_to_pois(mob, &set);
        let reached = path.as_ref().map(|p| p.can_reach()).unwrap_or(false);
        if reached {
            let target = path.unwrap().target_pos();
            if let Some(poi_type) = pm.get_type(target) {
                // poiManager.take(..., target, 1) — занять слот.
                let _ = pm.take(
                    &*acquirable_pois,
                    &|_h, bp| bp == target,
                    target,
                    1,
                );
                mob.living.brain_set_value(
                    &acquiring_memory_inner,
                    GlobalPos::of(
                        crate::resources::ResourceKey::<crate::world::level::Level>::create_with_registry_name(
                            crate::core::registry::registries::dimension(),
                            level
                                .dimension_id()
                                .cloned()
                                .unwrap_or_else(|| {
                                    crate::resources::identifier::Identifier::with_default_namespace("overworld").unwrap()
                                }),
                        ),
                        target,
                    ),
                );
                if let Some(id) = entity_event_id {
                    level.broadcast_entity_event(mob.living.entity.id, id);
                }
                state2.retries.borrow_mut().clear();
            }
        } else {
            let mut retries = state2.retries.borrow_mut();
            for (_, bp) in &set {
                let key = bp.as_long();
                retries
                    .entry(key)
                    .or_insert_with(|| {
                        JitteredLinearRetry::new(&mob.random, game_time)
                    });
            }
        }
        true
    };
    // entry conditions: absent(acquiringMemory) [+ absent(existingAbsentMemory)].
    let mut conds_map = conds(&[(acquiring_memory.clone(), MemoryStatus::ValueAbsent)]);
    if existing_absent_memory != acquiring_memory {
        conds_map.insert(existing_absent_memory.clone(), MemoryStatus::ValueAbsent);
    }
    let mut pb = PointBehavior::new(conds_map, trigger);
    pb
}

/// `GoToPotentialJobSite(speedModifier)`.
pub fn go_to_potential_job_site(speed_modifier: f32) -> impl BehaviorControl {
    struct GoToPotentialJobSite {
        base: BehaviorBase,
        speed_modifier: f32,
    }
    impl BehaviorControl for GoToPotentialJobSite {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }

        fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            mob.living
                .brain_get_active_non_core_activity()
                .map(|a| {
                    let n = a.get_name();
                    n == "idle" || n == "work" || n == "play"
                })
                .unwrap_or(true)
        }

        fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
            mob.living.brain_has(&keys::potential_job_site())
        }

        fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let mem: Option<GlobalPos> = mob.living.brain_get(&keys::potential_job_site());
            if let Some(gp) = mem {
                // setWalkAndLookTargetMemories(pos, speed, 1).
                mob.living.brain_set_value(
                    &keys::look_target(),
                    Rc::new(crate::world::entity::ai::memory::memory::BlockPosTracker::new(gp.pos))
                        as Rc<dyn memory::PositionTracker>,
                );
                let wt = WalkTarget::of_tracker(
                    Rc::new(crate::world::entity::ai::memory::memory::BlockPosTracker::new(gp.pos))
                        as Rc<dyn memory::PositionTracker>,
                    self.speed_modifier,
                    1,
                );
                mob.living.brain_set_value(&keys::walk_target(), wt);
            }
        }

        fn stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let mem: Option<GlobalPos> = mob.living.brain_get(&keys::potential_job_site());
            if let Some(gp) = mem {
                // level1 = server.getLevel(dimension) — у нас один
                // level; poi_manager текущего level.
                if let Some(pm) = level.poi_manager() {
                    if pm.exists(gp.pos, &|_| true) {
                        let _ = pm.release(gp.pos);
                    }
                }
            }
            mob.living.brain_erase(&keys::potential_job_site());
        }

        fn debug_string(&self) -> String { "GoToPotentialJobSite".to_string() }
    }
    GoToPotentialJobSite {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[(keys::potential_job_site(), MemoryStatus::ValuePresent)]),
            1200,
        ),
        speed_modifier,
    }
}

/// `SetClosestHomeAsWalkTarget.create(speedModifier)`.
pub fn set_closest_home_as_walk_target(speed_modifier: f32) -> PointBehavior {
    // Состояние: cache (pos→expiry) + next scan time.
    let cache = Rc::new(RefCell::new(HashMap::<i64, i64>::new()));
    let next_scan = Rc::new(RefCell::new(0i64));
    let cache2 = cache.clone();
    let next_scan2 = next_scan.clone();
    PointBehavior::new(
        conds(&[
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::home(), MemoryStatus::ValueAbsent),
        ]),
        move |level, mob, game_time| {
            if level.game_time() - *next_scan2.borrow() < 20 {
                return false;
            }
            let Some(pm) = level.poi_manager() else { return false };
            let home_key = poi_types::home();
            let home_pred: &PoiTypePredicate = &|h| {
                crate::world::entity::ai::village::poi::poi_record::holder_is(h, &home_key)
            };
            let center = mob.block_position();
            let Some(closest) = pm.find_closest(home_pred, center, 48, Occupancy::Any) else {
                return false;
            };
            if closest.dist_sqr(center) <= 4.0 {
                return false;
            }
            {
                let mut r = mob.random.borrow_mut();
                *next_scan2.borrow_mut() = level.game_time() + r.next_int_bound(20) as i64;
            }
            // findAllWithType(HOME, predicate(batch 5 + cache), ...).
            let taken = Rc::new(RefCell::new(0i32));
            let pos_predicate = {
                let cache = cache2.clone();
                let taken = taken.clone();
                let next_scan_val = *next_scan2.borrow() + 40;
                move |bp: BlockPos| -> bool {
                    let key = bp.as_long();
                    let mut cache = cache.borrow_mut();
                    if cache.contains_key(&key) {
                        return false;
                    }
                    let mut t = taken.borrow_mut();
                    *t += 1;
                    if *t >= 5 {
                        return false;
                    }
                    cache.insert(key, next_scan_val);
                    true
                }
            };
            let set: HashSet<(Holder<PoiType>, BlockPos)> = pm
                .find_all_with_type(home_pred, &|bp| pos_predicate(bp), center, 48, Occupancy::Any)
                .into_iter()
                .collect();
            let path = find_path_to_pois(mob, &set);
            let reached = path.as_ref().map(|p| p.can_reach()).unwrap_or(false);
            if reached {
                let target = path.unwrap().target_pos();
                if let Some(poi_type) = pm.get_type(target) {
                    let tracker = crate::world::entity::ai::memory::memory::BlockPosTracker::new(target);
                    mob.living.brain_set_value(
                        &keys::walk_target(),
                        WalkTarget::of_tracker(
                            Rc::new(tracker) as Rc<dyn memory::PositionTracker>,
                            speed_modifier,
                            1,
                        ),
                    );
                }
            } else if *taken.borrow() < 5 {
                // mutableInt < BATCH_SIZE → чистка cache.
                let now = *next_scan2.borrow();
                cache2.borrow_mut().retain(|_, v| *v >= now);
            }
            true
        },
    )
}

/// `ResetProfession.create()`.
pub fn reset_profession() -> PointBehavior {
    PointBehavior::new(
        conds(&[(keys::job_site(), MemoryStatus::ValueAbsent)]),
        |level, mob, _game_time| {
            let Some(data) = mob.villager_data.clone() else { return false };
            let is_none = villager_data::is_profession_none(&data.profession);
            let is_nitwit = villager_data::is_profession_nitwit(&data.profession);
            let flag = !is_none && !is_nitwit;
            if flag && mob.get_villager_xp() == 0 && data.level <= 1 {
                // withProfession(NONE) + refreshBrain.
                let new_data = data.with_profession(
                    villager_data::holder_for_profession(&villager_profession::none()),
                );
                mob.set_villager_data(new_data);
                mob.refresh_brain(level);
                return true;
            }
            false
        },
    )
}

/// `AssignProfessionFromJobSite.create()`.
pub fn assign_profession_from_job_site() -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::potential_job_site(), MemoryStatus::ValuePresent),
            (keys::job_site(), MemoryStatus::Registered),
        ]),
        |level, mob, _game_time| {
            let Some(gp): Option<GlobalPos> = mob.living.brain_get(&keys::potential_job_site()) else {
                return false;
            };
            // !globalPos.pos().closerToCenterThan(villager.position(), 2.0)
            // && !assignProfessionWhenSpawned → false.
            let center_close = {
                let c = crate::world::phys::vec3::at_center_of(&gp.pos.to_vec3i());
                let d = c.subtract_vec(&mob.living.entity.position());
                d.length_sqr() < 4.0
            };
            if !center_close && !mob.assign_profession_when_spawned() {
                return false;
            }
            mob.living.brain_erase(&keys::potential_job_site());
            mob.living.brain_set_value(&keys::job_site(), gp.clone());
            level.broadcast_entity_event(mob.living.entity.id, crate::world::entity::entity_event::VILLAGER_HAPPY);
            let data = mob.get_villager_data().clone();
            let already_has = !villager_data::is_profession_none(&data.profession);
            if already_has {
                return true;
            }
            // poi type → profession с heldJobSite(test).
            if let Some(pm) = level.poi_manager() {
                if let Some(poi) = pm.get_type(gp.pos) {
                    if let Some(key) = villager_profession::professions_for_poi(&poi)
                        .into_iter()
                        .next()
                    {
                        let new_data = data.with_profession(
                            villager_data::holder_for_profession(&key),
                        );
                        mob.set_villager_data(new_data);
                        mob.refresh_brain(level);
                    }
                }
            }
            true
        },
    )
}
