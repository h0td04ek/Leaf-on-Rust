//! Перенос простых `BehaviorBuilder`-поведений:
//! `OneShot`-семантика + entry-conditions + trigger-лямбда.
//!
//! Java `BehaviorBuilder.create(instance -> instance.group(...)
//! .apply(instance, (mems) -> (level, entity, gameTime) -> bool))`
//! отображается в `PointBehavior { entry_conditions, trigger }`:
//! `tryStart` = hasRequiredMemories + trigger; `tickOrStop` =
//! немедленный `doStop` (OneShot завершается за 1 тик).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::entity::schedule::activity::{activity_by_name, Activity};
use crate::world::entity::ai::behavior::behavior::{
    BehaviorControl, BehaviorStatus,
};
use crate::world::entity::ai::memory::memory::{
    keys, MemoryModuleType, MemoryStatus, WalkTarget,
};
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;

/// `Trigger<E>` — лямбда поведения `(level, entity, gameTime) -> bool`.
pub type TriggerFn = Rc<RefCell<dyn FnMut(&Arc<dyn LevelReader>, &mut Mob, i64) -> bool>>;

/// `Predicate<E>` для shouldErase/canCopy.
pub type MobPredicate = Rc<dyn Fn(&Mob) -> bool>;

/// `PointBehavior` — OneShot + entryConditions + trigger.
/// Покрывает `BehaviorBuilder.point(...)` и `group(...).apply`
/// с немедленным boolean-результатом.
pub struct PointBehavior {
    /// `entryCondition` — MemoryModuleType → MemoryStatus.
    pub entry_condition: HashMap<MemoryModuleType, MemoryStatus>,
    /// `status`.
    status: BehaviorStatus,
    /// `trigger(level, entity, gameTime)`.
    trigger: TriggerFn,
}

impl PointBehavior {
    /// `new(entryCondition, trigger)`.
    pub fn new(
        entry_condition: HashMap<MemoryModuleType, MemoryStatus>,
        trigger: impl FnMut(&Arc<dyn LevelReader>, &mut Mob, i64) -> bool + 'static,
    ) -> Self {
        Self {
            entry_condition,
            status: BehaviorStatus::Stopped,
            trigger: Rc::new(RefCell::new(trigger)),
        }
    }

    /// `BehaviorBuilder.point(trigger)` — без memory-условий.
    pub fn point(
        trigger: impl FnMut(&Arc<dyn LevelReader>, &mut Mob, i64) -> bool + 'static,
    ) -> Self {
        Self::new(HashMap::new(), trigger)
    }

    /// Условия `present`/`absent`/`registered`.
    pub fn with_conditions(mut self, pairs: &[(MemoryModuleType, MemoryStatus)]) -> Self {
        for (ty, st) in pairs {
            self.entry_condition.insert(ty.clone(), *st);
        }
        self
    }
}

impl BehaviorControl for PointBehavior {
    fn status(&self) -> BehaviorStatus {
        self.status
    }
    fn set_status(&mut self, status: BehaviorStatus) {
        self.status = status;
    }
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
        &self.entry_condition
    }
    fn durations(&self) -> (i32, i32) {
        (0, 0)
    }
    fn end_timestamp(&self) -> i64 {
        0
    }
    fn set_end_timestamp(&mut self, _timestamp: i64) {}

    /// OneShot.tryStart: hasRequiredMemories + trigger → RUNNING.
    fn try_start(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) -> bool {
        if self.has_required_memories(mob) {
            let fired = (self.trigger.borrow_mut())(level, mob, game_time);
            if fired {
                self.status = BehaviorStatus::Running;
            }
            fired
        } else {
            false
        }
    }

    /// OneShot.tickOrStop → doStop сразу.
    fn tick_or_stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        self.do_stop(level, mob, game_time);
    }

    /// OneShot.doStop → STOPPED.
    fn do_stop(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) {
        self.status = BehaviorStatus::Stopped;
    }

    fn debug_string(&self) -> String {
        "PointBehavior".to_string()
    }
}

/// `conditions!`-хелпер: пары (type, status) → HashMap.
pub fn conds(pairs: &[(MemoryModuleType, MemoryStatus)]) -> HashMap<MemoryModuleType, MemoryStatus> {
    pairs.iter().cloned().collect()
}

// ===================== BehaviorBuilder-точки =====================

/// `UpdateActivityFromSchedule.create()`.
pub fn update_activity_from_schedule() -> PointBehavior {
    PointBehavior::point(|level, mob, _game_time| {
        let day_time = level.game_time();
        let pos = mob.living.entity.position();
        if let Some(brain) = &mob.living.brain {
            brain.update_activity_from_schedule(day_time, pos, level.as_ref());
        }
        true
    })
}

/// `WakeUp.create()` — не REST && sleeping → stopSleeping.
pub fn wake_up() -> PointBehavior {
    PointBehavior::point(|_level, mob, _game_time| {
        let is_rest = mob
            .living
            .brain
            .as_ref()
            .map(|b| b.is_active(&activity_by_name("rest").unwrap_or_else(|| activity_by_name("idle").unwrap())))
            .unwrap_or(false);
        // isSleeping/stopSleeping — хуки LivingEntity.
        if !is_rest && mob.living.is_sleeping_entity() {
            mob.living.stop_sleeping();
            return true;
        }
        false
    })
}

/// `ReactToBell.create()` — HEARD_BELL_TIME present; raid==null →
/// setActiveActivityIfPossible(HIDE). Возвращает true всегда
/// (Java: point-безусловный true после проверки памяти).
pub fn react_to_bell() -> PointBehavior {
    PointBehavior::new(
        conds(&[(keys::heard_bell_time(), MemoryStatus::ValuePresent)]),
        |_level, mob, _game_time| {
            if let Some(brain) = &mob.living.brain {
                brain.set_active_activity_if_possible(
                    activity_by_name("hide").unwrap_or_else(|| activity_by_name("idle").unwrap()),
                );
            }
            true
        },
    )
}

/// `BecomePassiveIfMemoryPresent.create(pacifyingMemory, pacifyDuration)`.
pub fn become_passive_if_memory_present(
    pacifying_memory: MemoryModuleType,
    pacify_duration: i64,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::attack_target(), MemoryStatus::Registered),
            (keys::pacified(), MemoryStatus::ValueAbsent),
            (pacifying_memory.clone(), MemoryStatus::ValuePresent),
        ]),
        move |_level, mob, _game_time| {
            mob.living
                .brain_set_with_expiry(&keys::pacified(), true, pacify_duration);
            mob.living.brain_erase(&keys::attack_target());
            true
        },
    )
}

/// `EraseMemoryIf.create(shouldEraseMemory, erasingMemory)`.
pub fn erase_memory_if(
    should_erase_memory: MobPredicate,
    erasing_memory: MemoryModuleType,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[(erasing_memory.clone(), MemoryStatus::ValuePresent)]),
        move |_level, mob, _game_time| {
            if should_erase_memory(mob) {
                mob.living.brain_erase(&erasing_memory);
                true
            } else {
                false
            }
        },
    )
}

/// `CopyMemoryWithExpiry.create(canCopy, source, target, ttl)`.
/// `ttl` — UniformInt-эквивалент (min,max); sample через mob.random.
pub fn copy_memory_with_expiry<T: Clone + 'static>(
    can_copy_memory: MobPredicate,
    source_memory: MemoryModuleType,
    target_memory: MemoryModuleType,
    ttl_min: i32,
    ttl_max: i32,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (source_memory.clone(), MemoryStatus::ValuePresent),
            (target_memory.clone(), MemoryStatus::ValueAbsent),
        ]),
        move |_level, mob, _game_time| {
            if !can_copy_memory(mob) {
                return false;
            }
            let value: Option<T> = mob.living.brain_get(&source_memory);
            let Some(v) = value else { return false };
            let ttl = {
                let mut r = mob.random.borrow_mut();
                (ttl_min + r.next_int_bound((ttl_max + 1 - ttl_min).max(1))) as i64
            };
            mob.living.brain_set_with_expiry(&target_memory, v, ttl);
            true
        },
    )
}

/// `CountDownCooldownTicks` — Behavior с VALUE_PRESENT на
/// cooldownTicks; tick декрементит, stop стирает.
pub struct CountDownCooldownTicks {
    base: crate::world::entity::ai::behavior::behavior::BehaviorBase,
    cooldown_ticks: MemoryModuleType,
}

impl CountDownCooldownTicks {
    /// `new(cooldownTicks)`.
    pub fn new(cooldown_ticks: MemoryModuleType) -> Self {
        Self {
            base: crate::world::entity::ai::behavior::behavior::BehaviorBase::new(conds(&[(
                cooldown_ticks.clone(),
                MemoryStatus::ValuePresent,
            )])),
            cooldown_ticks,
        }
    }
}

impl BehaviorControl for CountDownCooldownTicks {
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

    /// timedOut → false (бесконечно пока cooldown > 0).
    fn timed_out(&self, _game_time: i64) -> bool {
        false
    }

    /// canStillUse: memory present && > 0.
    fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
        mob.living
            .brain_get::<i32>(&self.cooldown_ticks)
            .map(|v| v > 0)
            .unwrap_or(false)
    }

    /// tick: value - 1.
    fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        let v = mob.living.brain_get::<i32>(&self.cooldown_ticks);
        if let Some(v) = v {
            mob.living.brain_set_value(&self.cooldown_ticks, v - 1);
        }
    }

    /// stop: eraseMemory.
    fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        mob.living.brain_erase(&self.cooldown_ticks);
    }
}

/// `ResetRaidStatus.create()` — random 1/20: raid null/stopped/loss →
/// default+update IDLE.
pub fn reset_raid_status() -> PointBehavior {
    PointBehavior::point(|level, mob, _game_time| {
        if mob.random.borrow_mut().next_int_bound(20) != 0 {
            return false;
        }
        let pos = mob.living.entity.block_position();
        let raid = level.raid_at(pos);
        let should_idle = raid
            .as_ref()
            .map(|r| {
                let r = r.borrow();
                r.is_stopped() || r.is_loss()
            })
            .unwrap_or(true);
        if should_idle {
            if let Some(brain) = &mob.living.brain {
                brain.set_default_activity(
                    activity_by_name("idle").unwrap_or_else(|| activity_by_name("core").unwrap()),
                );
                brain.update_activity_from_schedule(
                    level.game_time(),
                    mob.living.entity.position(),
                    level.as_ref(),
                );
            }
        }
        true
    })
}

/// `SetRaidStatus.create()` — random 1/20: raid active →
/// default+active RAID или PRE_RAID.
pub fn set_raid_status() -> PointBehavior {
    PointBehavior::point(|level, mob, _game_time| {
        if mob.random.borrow_mut().next_int_bound(20) != 0 {
            return false;
        }
        let pos = mob.living.entity.block_position();
        if let Some(raid) = level.raid_at(pos) {
            let r = raid.borrow();
            let name = if r.has_first_wave_spawned() && !r.is_between_waves(0) {
                "raid"
            } else {
                "pre_raid"
            };
            drop(r);
            if let Some(brain) = &mob.living.brain {
                if let Some(act) = activity_by_name(name) {
                    brain.set_default_activity(act.clone());
                    brain.set_active_activity_if_possible(act);
                }
            }
        }
        true
    })
}

/// `StopBeingAngryIfTargetDead.create()` — ANGRY_AT (UUID) →
/// entity мертва/игрок-simple → erase.
pub fn stop_being_angry_if_target_dead() -> PointBehavior {
    PointBehavior::new(
        conds(&[(keys::angry_at(), MemoryStatus::ValuePresent)]),
        |level, mob, _game_time| {
            // ANGRY_AT хранит Uuid (UniversalAnger target).
            let uuid: Option<crate::util::uuid::Uuid> = mob.living.brain_get(&keys::angry_at());
            let Some(uuid) = uuid else { return true };
            let dead = level
                .entity_by_uuid(uuid)
                .map(|e| {
                    if e.is_living_entity() {
                        let le = crate::world::entity::living_entity::LivingEntity::from_entity(e.clone());
                        let is_player = crate::world::entity::ai::sensing::sensor::entity_type_is(&le, "player");
                        (le.is_dead_or_dying() || !le.is_alive())
                            && (!is_player || level.forgive_dead_players())
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
            if dead {
                mob.living.brain_erase(&keys::angry_at());
            }
            true
        },
    )
}

/// `SetWalkTargetFromLookTarget.create(speedModifier, closeEnoughDist)`.
pub fn set_walk_target_from_look_target(speed_modifier: f32, close_enough_dist: i32) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::look_target(), MemoryStatus::ValuePresent),
        ]),
        move |_level, mob, _game_time| {
            // LOOK_TARGET → Rc<dyn PositionTracker>.
            let tracker: Option<Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>> =
                mob.living.brain_get(&keys::look_target());
            if let Some(tracker) = tracker {
                let wt = WalkTarget::of_tracker(tracker, speed_modifier, close_enough_dist);
                mob.living.brain_set_value(&keys::walk_target(), wt);
                true
            } else {
                false
            }
        },
    )
}

/// `StrollToPoi.create(poiPosMemory, speedModifier, closeEnoughDist, maxDistFromPoi)`.
pub fn stroll_to_poi(
    poi_pos_memory: MemoryModuleType,
    speed_modifier: f32,
    close_enough_dist: i32,
    max_dist_from_poi: i32,
) -> PointBehavior {
    // mutableLong — последний раз когда цель обновлялась.
    let lu = Rc::new(RefCell::new(0i64));
    PointBehavior::new(
        conds(&[
            (keys::walk_target(), MemoryStatus::Registered),
            (poi_pos_memory.clone(), MemoryStatus::ValuePresent),
        ]),
        move |level, mob, game_time| {
            let gpos: Option<crate::core::global_pos::GlobalPos> =
                mob.living.brain_get(&poi_pos_memory);
            let Some(gpos) = gpos else { return false };
            // dimension mismatch → false (dimension_id → ResourceKey).
            let dim_ok = level
                .dimension_id()
                .map(|d| {
                    d == &gpos.dimension.identifier
                })
                .unwrap_or(false);
            if !dim_ok {
                return false;
            }
            // closerToCenterThan(mob.position(), maxDist) — расстояние
            // от центра блока POI до моба <= maxDist.
            let center = Vec3::new(
                gpos.pos.x as f64 + 0.5,
                gpos.pos.y as f64 + 0.5,
                gpos.pos.z as f64 + 0.5,
            );
            let dist = mob.living.entity.position().distance_to(&center);
            if dist > max_dist_from_poi as f64 {
                return false;
            }
            let mut lu = lu.borrow_mut();
            if game_time <= *lu {
                return true;
            }
            *lu = game_time + 80;
            let wt = WalkTarget::new_block(gpos.pos, speed_modifier, close_enough_dist);
            mob.living.brain_set_value(&keys::walk_target(), wt);
            true
        },
    )
}

/// `DismountOrSkipMounting.create(maxDistanceFromVehicle, shouldStopRiding)`.
pub fn dismount_or_skip_mounting(
    max_distance_from_vehicle: i32,
    should_stop_riding: impl Fn(&Mob, &crate::world::entity::entity::Entity) -> bool + 'static,
) -> PointBehavior {
    let pred = Rc::new(should_stop_riding);
    PointBehavior::new(
        conds(&[(keys::ride_target(), MemoryStatus::Registered)]),
        move |_level, mob, _game_time| {
            let vehicle = mob.living.entity.get_vehicle();
            let ride_target: Option<crate::world::entity::entity::Entity> =
                mob.living.brain_get(&keys::ride_target());
            if vehicle.is_none() && ride_target.is_none() {
                return false;
            }
            let entity2 = vehicle.or(ride_target);
            let Some(entity2) = entity2 else { return false };
            let same_level = match (&entity2.level, &mob.living.entity.level) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            };
            let valid = entity2.is_alive()
                && entity2.distance_to_sqr(&mob.living.entity)
                    <= (max_distance_from_vehicle * max_distance_from_vehicle) as f64
                && same_level;
            if valid && !pred(mob, &entity2) {
                return false;
            }
            mob.living.entity.stop_riding();
            mob.living.brain_erase(&keys::ride_target());
            true
        },
    )
}

/// `VillagerCalmDown.create()` — HURT_BY/HURT_BY_ENTITY/NEAREST_HOSTILE
/// registered; все "спокойны" → erase hurt-памяти + update schedule.
pub fn villager_calm_down() -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::hurt_by(), MemoryStatus::Registered),
            (keys::hurt_by_entity(), MemoryStatus::Registered),
            (keys::nearest_hostile(), MemoryStatus::Registered),
        ]),
        |_level, mob, _game_time| {
            let hurt_by = mob.living.brain_has(&keys::hurt_by());
            let nearest_hostile = mob.living.brain_has(&keys::nearest_hostile());
            let hurt_by_entity_close = mob
                .living
                .brain_get::<crate::world::entity::living_entity::LivingEntity>(&keys::hurt_by_entity())
                .map(|e| e.distance_to_sqr(&mob.living.entity) <= 36.0)
                .unwrap_or(false);
            let flag = hurt_by || nearest_hostile || hurt_by_entity_close;
            if !flag {
                mob.living.brain_erase(&keys::hurt_by());
                mob.living.brain_erase(&keys::hurt_by_entity());
                if let Some(brain) = &mob.living.brain {
                    brain.update_activity_from_schedule(
                        _level.game_time(),
                        mob.living.entity.position(),
                        _level.as_ref(),
                    );
                }
            }
            true
        },
    )
}

/// `SetLookAndInteract.create(entityTypeId, maxDist)` — ближайшая
/// видимая сущность нужного типа → INTERACTION_TARGET + LOOK_TARGET.
pub fn set_look_and_interact(entity_type_id: &str, max_dist: i32) -> PointBehavior {
    let id = entity_type_id.to_string();
    PointBehavior::new(
        conds(&[
            (keys::look_target(), MemoryStatus::Registered),
            (keys::interaction_target(), MemoryStatus::ValueAbsent),
            (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
        ]),
        move |_level, mob, _game_time| {
            let max_sq = (max_dist * max_dist) as f64;
            let nvle: Option<crate::world::entity::ai::memory::memory::NearestVisibleLivingEntities> =
                mob.living.brain_get(&keys::nearest_visible_living_entities());
            let found = nvle.and_then(|nv| {
                nv.find_closest(&|e| {
                    e.distance_to_sqr(&mob.living.entity) <= max_sq
                        && crate::world::entity::ai::sensing::sensor::entity_type_is(e, &id)
                })
            });
            if let Some(target) = found {
                mob.living.brain_set_value(&keys::interaction_target(), target.clone());
                let tracker = crate::world::entity::ai::memory::memory::EntityTracker::new(
                    target.entity.clone(),
                    true,
                );
                mob.living.brain_set_value(
                    &keys::look_target(),
                    std::rc::Rc::new(tracker) as std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>,
                );
                true
            } else {
                false
            }
        },
    )
}

/// `BackUpIfTooClose.create(tooCloseDistance, strafeSpeed)` — OneShot.
pub fn back_up_if_too_close(too_close_distance: i32, strafe_speed: f32) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::look_target(), MemoryStatus::Registered),
            (keys::attack_target(), MemoryStatus::ValuePresent),
            (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
        ]),
        move |_level, mob, _game_time| {
            let target: Option<crate::world::entity::living_entity::LivingEntity> =
                mob.living.brain_get(&keys::attack_target());
            let Some(target) = target else { return false };
            let close = target.entity.distance_to_sqr(&mob.living.entity)
                <= (too_close_distance * too_close_distance) as f64;
            let visible = mob
                .living
                .brain_get::<crate::world::entity::ai::memory::memory::NearestVisibleLivingEntities>(
                    &keys::nearest_visible_living_entities(),
                )
                .map(|nv| nv.nearby_entities.iter().any(|e| e.get_uuid() == target.get_uuid()))
                .unwrap_or(false);
            if close && visible {
                let tracker = crate::world::entity::ai::memory::memory::EntityTracker::new(
                    target.entity.clone(),
                    true,
                );
                mob.living.brain_set_value(
                    &keys::look_target(),
                    std::rc::Rc::new(tracker) as std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>,
                );
                // strafe(-strafeSpeed, 0) — MoveControl.
                mob.move_control.strafe(-strafe_speed, 0.0);
                true
            } else {
                false
            }
        },
    )
}

/// `Mount.create(speedModifier)`.
pub fn mount(speed_modifier: f32) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::look_target(), MemoryStatus::Registered),
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::ride_target(), MemoryStatus::ValuePresent),
        ]),
        move |_level, mob, _game_time| {
            if mob.living.entity.is_passenger() {
                return false;
            }
            let target: Option<crate::world::entity::entity::Entity> =
                mob.living.brain_get(&keys::ride_target());
            let Some(target) = target else { return false };
            if target.distance_to_sqr(&mob.living.entity) <= 1.0 {
                mob.living.entity.start_riding(&target);
            } else {
                let tracker = crate::world::entity::ai::memory::memory::EntityTracker::new(
                    target.clone(),
                    true,
                );
                mob.living.brain_set_value(
                    &keys::look_target(),
                    std::rc::Rc::new(tracker) as std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>,
                );
                let wt = WalkTarget::new_entity(target, speed_modifier, 1);
                mob.living.brain_set_value(&keys::walk_target(), wt);
            }
            true
        },
    )
}
