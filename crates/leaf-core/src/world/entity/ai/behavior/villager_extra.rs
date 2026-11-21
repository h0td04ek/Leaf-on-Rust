//! Перенос оставшихся villager/social/POI поведений:
//! `InteractWith`, `RingBell`, `SocializeAtBell`,
//! `SetWalkTargetFromBlockMemory`, `ValidateNearbyPoi`,
//! `YieldJobSite`, `VillagerPanicTrigger`, `WorkAtPoi`,
//! `WorkAtComposter`, `UseBonemeal`, `HarvestFarmland`,
//! `SleepInBed`, `JumpOnBed`, `InteractWithDoor`,
//! `LookAndFollowTradingPlayerSink`, `TradeWithVillager`,
//! `VillagerMakeLove`, `Croak`, `SpearApproach`,
//! `SpearAttack`, `SpearRetreat`.
//!
//! Ограничение порта: memory хранит клоны `LivingEntity`,
//! поэтому villager-поля *другой* сущности (профессия,
//! инвентарь партнёра) недоступны — partner-сторона
//! `TradeWithVillager`/`VillagerMakeLove` работает по
//! entity-полям (age/love/brain), как и везде в порте.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::global_pos::GlobalPos;
use crate::core::holder::Holder;
use crate::entity::humanoid_arm::HumanoidArm;
use crate::world::entity::pose::Pose;
use crate::resources::identifier::Identifier;
use crate::tags::block_tags::BlockTags;
use crate::util::uuid::Uuid;
use crate::world::entity::ai::behavior::behavior::{
    BehaviorBase, BehaviorControl, BehaviorStatus,
};
use crate::world::entity::ai::behavior::behavior_utils;
use crate::world::entity::ai::behavior::simple::{conds, PointBehavior};
use crate::world::entity::ai::memory::memory::{
    self, keys, MemoryModuleType, MemoryStatus, WalkTarget,
};
use crate::world::entity::ai::village::poi::poi_manager::{Occupancy, PoiManager};
use crate::world::entity::ai::village::poi::poi_type::PoiType;
use crate::world::entity::ai::village::poi::poi_types;
use crate::world::entity::mob::Mob;
use crate::world::entity::npc::villager::villager_data;
use crate::world::entity::npc::villager::villager_profession;
use crate::world::item::item_stack::ItemStack;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;

// ---------- helpers ----------

/// `EntityType.VILLAGER`-проверка по id типа сущности.
/// `entity.getType()` path (`minecraft:villager` -> "villager").
fn entity_type_path(e: &crate::world::entity::entity::Entity) -> String {
    e.get_type()
        .key()
        .map(|k| k.identifier.get_path().to_string())
        .unwrap_or_default()
}

fn is_villager_entity(living: &crate::world::entity::living_entity::LivingEntity) -> bool {
    entity_type_path(&living.entity) == "villager"
}

/// `ItemStack` из registry-id предмета.
fn item_stack_of(id: &Identifier, count: i32) -> ItemStack {
    crate::world::item::item::item_by_identifier(id)
        .map(|item| {
            ItemStack::new(
                crate::core::holder::Holder::Direct(item.as_ref().clone()),
                count,
            )
        })
        .unwrap_or_else(ItemStack::empty)
}

/// `state.block.id` path.
fn block_id_path(state: &crate::world::level::block::state::block_state::BlockState) -> String {
    state.block.id.get_path().to_string()
}

/// `level.getBlockState(pos).is(BlockTags.X)`.
fn block_is_tag(level: &Arc<dyn LevelReader>, pos: BlockPos, tag: &Identifier) -> bool {
    level
        .get_block_state(pos)
        .map(|s| s.is_block_tag(tag))
        .unwrap_or(false)
}

/// `closerToCenterThan(pos, dist)` — squared distance от
/// центра блока до позиции.
fn pos_closer_to_center(pos: BlockPos, entity_pos: Vec3, dist: f64) -> bool {
    let center = crate::world::phys::vec3::at_center_of(&pos.to_vec3i());
    center.distance_to_sqr(&entity_pos) < dist * dist
}

/// `GlobalPos.dimension() == level.dimension()`.
fn same_dimension(level: &Arc<dyn LevelReader>, gp: &GlobalPos) -> bool {
    level
        .dimension_id()
        .map(|d| gp.dimension.identifier == *d)
        .unwrap_or(true)
}

fn brain_of(mob: &Mob) -> Option<Rc<crate::world::entity::ai::brain::Brain>> {
    mob.living.brain.clone()
}

// ---------- InteractWith ----------

/// `InteractWith.of(type, interactionRange, selfFilter,
/// targetFilter, memory, speedModifier, maxDist)`.
/// `type_id` — id типа цели (`"villager"` и т.д.);
/// фильтры — `Rc<dyn Fn(&Mob)->bool>` / `Rc<dyn Fn(&LivingEntity)->bool>`.
pub fn interact_with(
    type_id: &'static str,
    interaction_range: i32,
    self_filter: std::sync::Arc<dyn Fn(&Mob) -> bool + Send + Sync>,
    target_filter: std::sync::Arc<dyn Fn(
        &crate::world::entity::living_entity::LivingEntity,
    ) -> bool
        + Send
        + Sync>,
    memory: MemoryModuleType,
    speed_modifier: f32,
    max_dist: i32,
) -> PointBehavior {
    let i = (interaction_range * interaction_range) as f64;
    let type_id_owned = type_id.to_string();
    let predicate: std::sync::Arc<dyn Fn(&crate::world::entity::living_entity::LivingEntity) -> bool + Send + Sync> =
        std::sync::Arc::new(move |e| {
            entity_type_path(&e.entity) == type_id_owned && target_filter(e)
        });
    PointBehavior::new(
        conds(&[
            (memory.clone(), MemoryStatus::Registered),
            (keys::look_target(), MemoryStatus::Registered),
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
        ]),
        move |level, mob, _game_time| {
            if !self_filter(mob) {
                return false;
            }
            let Some(nvle) = mob
                .living
                .brain_get::<memory::NearestVisibleLivingEntities>(
                    &keys::nearest_visible_living_entities(),
                )
            else {
                return false;
            };
            if !nvle.contains_predicate(&|e| predicate(e)) {
                return false;
            }
            let self_pos = mob.living.entity.position();
            let found = nvle.find_closest(&|e| {
                e.entity.position().distance_to_sqr(&self_pos) <= i && predicate(e)
            });
            if let Some(target) = found {
                mob.living.brain_set_value(&memory, target.clone());
                mob.living.brain_set_value(
                    &keys::look_target(),
                    Rc::new(memory::EntityTracker::new(target.entity.clone(), true))
                        as Rc<dyn memory::PositionTracker>,
                );
                mob.living.brain_set_value(
                    &keys::walk_target(),
                    WalkTarget::of_tracker(
                        Rc::new(memory::EntityTracker::new(target.entity.clone(), false))
                            as Rc<dyn memory::PositionTracker>,
                        speed_modifier,
                        max_dist,
                    ),
                );
            }
            true
        },
    )
}

// ---------- RingBell ----------

/// `RingBell.create()` — MEMORIES: meeting_point PRESENT.
/// Шанс 5%/тик? В Java `nextFloat() <= 0.95 → false`,
/// т.е. звонит с вероятностью 0.05 когда ближе 3 блоков.
pub fn ring_bell() -> PointBehavior {
    PointBehavior::new(
        conds(&[(keys::meeting_point(), MemoryStatus::ValuePresent)]),
        |level, mob, _game_time| {
            {
                let mut r = mob.random.borrow_mut();
                if r.next_float() <= 0.95 {
                    return false;
                }
            }
            let Some(mp): Option<GlobalPos> = mob.living.brain_get(&keys::meeting_point()) else {
                return false;
            };
            let bp = mp.pos;
            if bp.dist_manhattan(mob.block_position()) < 3 {
                // BellBlock.attemptToRing — звенит если блок bell
                // (id содержит "bell") и нет cooldown.
                if let Some(state) = level.get_block_state(bp) {
                    if block_id_path(&state).contains("bell") {
                        level.game_event(
                            &crate::world::level::gameevent::game_event::block_change(),
                            mob.living.entity.block_position(),
                        );
                    }
                }
            }
            true
        },
    )
}

// ---------- SocializeAtBell ----------

/// `SocializeAtBell.create()`.
pub fn socialize_at_bell() -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::walk_target(), MemoryStatus::Registered),
            (keys::look_target(), MemoryStatus::Registered),
            (keys::meeting_point(), MemoryStatus::ValuePresent),
            (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
            (keys::interaction_target(), MemoryStatus::ValueAbsent),
        ]),
        |level, mob, _game_time| {
            let Some(mp): Option<GlobalPos> = mob.living.brain_get(&keys::meeting_point()) else {
                return false;
            };
            let Some(nvle) = mob
                .living
                .brain_get::<memory::NearestVisibleLivingEntities>(
                    &keys::nearest_visible_living_entities(),
                )
            else {
                return false;
            };
            let roll = {
                let mut r = mob.random.borrow_mut();
                r.next_int_bound(100)
            };
            if roll != 0
                || !same_dimension(level, &mp)
                || !pos_closer_to_center(mp.pos, mob.living.entity.position(), 4.0)
                || !nvle.contains_predicate(&|e| is_villager_entity(e))
            {
                return false;
            }
            let found = nvle.find_closest(&|e| {
                is_villager_entity(e)
                    && e.entity
                        .position()
                        .distance_to_sqr(&mob.living.entity.position())
                        <= 32.0
            });
            if let Some(near) = found {
                mob.living
                    .brain_set_value(&keys::interaction_target(), near.clone());
                mob.living.brain_set_value(
                    &keys::look_target(),
                    Rc::new(memory::EntityTracker::new(near.entity.clone(), true))
                        as Rc<dyn memory::PositionTracker>,
                );
                mob.living.brain_set_value(
                    &keys::walk_target(),
                    WalkTarget::of_tracker(
                        Rc::new(memory::EntityTracker::new(near.entity.clone(), false))
                            as Rc<dyn memory::PositionTracker>,
                        0.3,
                        1,
                    ),
                );
            }
            true
        },
    )
}

// ---------- SetWalkTargetFromBlockMemory ----------

/// `SetWalkTargetFromBlockMemory.create(blockTargetMemory,
/// speedModifier, closeEnoughDist, tooFarDistance,
/// tooLongUnreachableDuration)` — generic по memory-ключу.
pub fn set_walk_target_from_block_memory(
    block_target_memory: MemoryModuleType,
    speed_modifier: f32,
    close_enough_dist: i32,
    too_far_distance: i32,
    too_long_unreachable_duration: i64,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::cant_reach_walk_target_since(), MemoryStatus::Registered),
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (block_target_memory.clone(), MemoryStatus::ValuePresent),
        ]),
        move |level, mob, game_time| {
            let Some(gp): Option<GlobalPos> =
                mob.living.brain_get(&block_target_memory)
            else {
                return false;
            };
            let since: Option<i64> = mob
                .living
                .brain_get(&keys::cant_reach_walk_target_since());
            if !same_dimension(level, &gp)
                || since
                    .map(|s| level.game_time() - s > too_long_unreachable_duration)
                    .unwrap_or(false)
            {
                // releasePoi + erase + set cantReach.
                if let Some(pm) = level.poi_manager() {
                    let _ = pm.release(gp.pos);
                }
                mob.living.brain_erase(&block_target_memory);
                mob.living
                    .brain_set_value(&keys::cant_reach_walk_target_since(), game_time);
                return true;
            }
            let dist = gp.pos.dist_manhattan(mob.block_position());
            if dist > too_far_distance {
                // getPosTowards ×1000 пока не подберёт позицию
                // в пределах tooFarDistance от жителя.
                let target_center = crate::world::phys::vec3::at_center_of(&gp.pos.to_vec3i());
                let mut found: Option<Vec3> = None;
                for _ in 0..1000 {
                    let pos = crate::world::entity::ai::util::default_random_pos::get_pos_towards(
                        mob,
                        15,
                        7,
                        target_center,
                        std::f64::consts::FRAC_PI_2,
                    );
                    if let Some(p) = pos {
                        let bp = BlockPos::containing(p.x, p.y, p.z);
                        if bp.dist_manhattan(mob.block_position()) <= too_far_distance {
                            found = Some(p);
                            break;
                        }
                    }
                }
                if let Some(vec3) = found {
                    mob.living.brain_set_value(
                        &keys::walk_target(),
                        WalkTarget::new_vec3(vec3, speed_modifier, close_enough_dist),
                    );
                } else {
                    if let Some(pm) = level.poi_manager() {
                        let _ = pm.release(gp.pos);
                    }
                    mob.living.brain_erase(&block_target_memory);
                    mob.living
                        .brain_set_value(&keys::cant_reach_walk_target_since(), game_time);
                }
            } else if dist > close_enough_dist {
                mob.living.brain_set_value(
                    &keys::walk_target(),
                    WalkTarget::new_block(gp.pos, speed_modifier, close_enough_dist),
                );
            }
            true
        },
    )
}

// ---------- ValidateNearbyPoi ----------

/// `ValidateNearbyPoi.create(poiValidator, poiPosMemory)`.
/// `poiValidator` — `Rc<dyn Fn(&Holder<PoiType>)->bool>`.
pub fn validate_nearby_poi(
    poi_validator: Rc<dyn Fn(&Holder<PoiType>) -> bool + Send + Sync>,
    poi_pos_memory: MemoryModuleType,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[(poi_pos_memory.clone(), MemoryStatus::ValuePresent)]),
        move |level, mob, _game_time| {
            let Some(gp): Option<GlobalPos> = mob.living.brain_get(&poi_pos_memory) else {
                return false;
            };
            let bp = gp.pos;
            if !same_dimension(level, &gp) || !pos_closer_to_center(bp, mob.living.entity.position(), 16.0) {
                return false;
            }
            let exists = level
                .poi_manager()
                .map(|pm| pm.exists(bp, &|h| poi_validator(h)))
                .unwrap_or(false);
            if !exists {
                mob.living.brain_erase(&poi_pos_memory);
                return true;
            }
            // bedIsOccupied: BEDS + occupied=true + !isSleeping.
            let mut occupied = false;
            if let Some(state) = level.get_block_state(bp) {
                if state.is_block_tag(&BlockTags::beds()) {
                    occupied = matches!(
                        state.values.get_by_name("occupied"),
                        Some(crate::world::level::block::state::property::StateValue::Bool(true))
                    ) && !mob.living.is_sleeping_entity();
                }
            }
            if occupied {
                mob.living.brain_erase(&poi_pos_memory);
                // bedIsOccupiedByVillager требует entity-scan со
                // sleeping-флагом — entities_in_box отдаёт Entity
                // без LivingEntity; release откладываем до runtime
                // (консервативно: POI не освобождаем).
            }
            true
        },
    )
}

// ---------- YieldJobSite ----------

/// `YieldJobSite.create(speedModifier)`.
pub fn yield_job_site(speed_modifier: f32) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::potential_job_site(), MemoryStatus::ValuePresent),
            (keys::job_site(), MemoryStatus::ValueAbsent),
            (keys::nearest_living_entities(), MemoryStatus::ValuePresent),
            (keys::walk_target(), MemoryStatus::Registered),
            (keys::look_target(), MemoryStatus::Registered),
        ]),
        move |level, mob, _game_time| {
            if mob.living.entity.is_baby() {
                return false;
            }
            let data = mob.get_villager_data().clone();
            if !villager_data::is_profession_none(&data.profession) {
                return false;
            }
            let Some(gp): Option<GlobalPos> =
                mob.living.brain_get(&keys::potential_job_site())
            else {
                return false;
            };
            let bp = gp.pos;
            let poi_type = level
                .poi_manager()
                .and_then(|pm| pm.get_type(bp));
            let Some(poi) = poi_type else {
                return true;
            };
            // NEAREST_LIVING_ENTITIES — Vec<LivingEntity>.
            let nle: Option<Vec<crate::world::entity::living_entity::LivingEntity>> =
                mob.living
                    .brain_get(&keys::nearest_living_entities());
            let Some(list) = nle else { return true };
            // Ближайший другой житель, желающий этот job site.
            let mut found: Option<crate::world::entity::living_entity::LivingEntity> = None;
            for near in &list {
                if !is_villager_entity(near) {
                    continue;
                }
                if near.entity.position() == mob.living.entity.position()
                    && near.entity.id == mob.living.entity.id
                {
                    continue;
                }
                // nearbyWantsJobsite: профессия NONE + нет JOB_SITE
                // + POI heldJobSite подходит (упрощённо: всегда true
                // дляHolder-матчинга типа ниже).
                let _ = poi;
                found = Some(near.clone());
                break;
            }
            if let Some(_near) = found {
                mob.living.brain_erase(&keys::walk_target());
                mob.living.brain_erase(&keys::look_target());
                mob.living.brain_erase(&keys::potential_job_site());
                // У near-жителя нет доступа к Mob — память
                // POTENTIAL_JOB_SITE ставится через brain клона
                // (shared Rc — сработает если это тот же Brain).
                if let Some(nb) = _near.brain.clone() {
                    if nb.get_memory::<GlobalPos>(&keys::job_site()).is_none() {
                        behavior_utils::set_walk_and_look_target_memories_pos(
                            mob,
                            bp,
                            speed_modifier,
                            1,
                        );
                        nb.set_memory_value(
                            &keys::potential_job_site(),
                            GlobalPos::of(
                                crate::resources::ResourceKey::<crate::world::level::LevelKey>::create_with_registry_name(
                                    crate::core::registry::registries::dimension(),
                                    level
                                        .dimension_id()
                                        .cloned()
                                        .unwrap_or_else(|| {
                                            Identifier::with_default_namespace("overworld")
                                                .unwrap()
                                        }),
                                ),
                                bp,
                            ),
                        );
                    }
                }
            }
            true
        },
    )
}

// ---------- VillagerPanicTrigger ----------

/// `VillagerPanicTrigger` — Behavior с maxDuration=i32::MAX.
pub fn villager_panic_trigger() -> impl BehaviorControl {
    struct VillagerPanicTrigger {
        base: BehaviorBase,
    }
    impl BehaviorControl for VillagerPanicTrigger {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "VillagerPanicTrigger".to_string() }

        fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            is_hurt(mob) || has_hostile(mob)
        }

        fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
            is_hurt(mob) || has_hostile(mob)
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            if !(is_hurt(mob) || has_hostile(mob)) {
                return;
            }
            if let Some(brain) = brain_of(mob) {
                let panic = crate::entity::schedule::activity::activity_by_name("panic");
                let already = panic
                    .as_ref()
                    .map(|a| brain.is_active(a))
                    .unwrap_or(false);
                if !already {
                    brain.erase_memory(&keys::path());
                    brain.erase_memory(&keys::walk_target());
                    brain.erase_memory(&keys::look_target());
                    brain.erase_memory(&keys::breed_target());
                    brain.erase_memory(&keys::interaction_target());
                }
                if let Some(a) = panic {
                    brain.set_active_activity_if_possible(a);
                }
            }
        }

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            if game_time % 100 == 0 {
                mob.spawn_golem_if_needed(level, game_time, 3);
            }
        }
    }
    VillagerPanicTrigger {
        base: BehaviorBase::with_duration(BehaviorBase::conditions(&[]), i32::MAX),
    }
}

/// `VillagerPanicTrigger.hasHostile`.
pub fn has_hostile(entity: &Mob) -> bool {
    entity
        .living
        .brain_has(&keys::nearest_hostile())
}

/// `VillagerPanicTrigger.isHurt`.
pub fn is_hurt(entity: &Mob) -> bool {
    entity.living.brain_has(&keys::hurt_by())
}

// ---------- WorkAtPoi (база для job-site поведений) ----------

/// `WorkAtPoi` — базовый Behavior; `useWorkstation`
/// переопределяется через `workstation_hook`.
pub fn work_at_poi(
    workstation_hook: Option<Rc<dyn Fn(&Arc<dyn LevelReader>, &mut Mob) + Send + Sync>>,
) -> impl BehaviorControl {
    struct WorkAtPoi {
        base: BehaviorBase,
        last_check: i64,
        workstation_hook: Option<Rc<dyn Fn(&Arc<dyn LevelReader>, &mut Mob) + Send + Sync>>,
    }
    impl BehaviorControl for WorkAtPoi {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "WorkAtPoi".to_string() }

        fn check_extra_start_conditions(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            if level.game_time() - self.last_check < 300 {
                return false;
            }
            {
                let mut r = mob.random.borrow_mut();
                if r.next_int_bound(2) != 0 {
                    return false;
                }
            }
            self.last_check = level.game_time();
            let Some(gp): Option<GlobalPos> = mob.living.brain_get(&keys::job_site()) else {
                return false;
            };
            same_dimension(level, &gp) && pos_closer_to_center(gp.pos, mob.living.entity.position(), 1.73)
        }

        fn can_still_use(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
            let Some(gp): Option<GlobalPos> = mob.living.brain_get(&keys::job_site()) else {
                return false;
            };
            same_dimension(level, &gp) && pos_closer_to_center(gp.pos, mob.living.entity.position(), 1.73)
        }

        fn start(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            mob.living
                .brain_set_value(&keys::last_worked_at_poi(), game_time);
            let gp: Option<GlobalPos> = mob.living.brain_get(&keys::job_site());
            if let Some(g) = gp {
                mob.living.brain_set_value(
                    &keys::look_target(),
                    Rc::new(memory::BlockPosTracker::new(g.pos))
                        as Rc<dyn memory::PositionTracker>,
                );
            }
            // playWorkSound.
            if let Some(sound) = mob.villager_work_sound() {
                level.play_sound(
                    mob.living.entity.position(),
                    &sound,
                    1.0,
                    1.0,
                );
            }
            if let Some(hook) = &self.workstation_hook {
                hook(level, mob);
            }
            if mob.should_restock(level) {
                mob.restock(level);
            }
        }
    }
    WorkAtPoi {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[
                (keys::job_site(), MemoryStatus::ValuePresent),
                (keys::look_target(), MemoryStatus::Registered),
            ]),
            60,
        ),
        last_check: 0,
        workstation_hook,
    }
}

/// `WorkAtComposter.useWorkstation` — компостер: хлеб из
/// пшеницы + компостирование семян (ComposterBlock-логика
/// через block-state `level` свойство).
pub fn work_at_composter() -> impl BehaviorControl {
    work_at_poi(Some(Rc::new(|level, mob| {
        let gp: Option<GlobalPos> = mob.living.brain_get(&keys::job_site());
        let Some(g) = gp else { return };
        let Some(state) = level.get_block_state(g.pos) else { return };
        let id = format!("minecraft:{}", block_id_path(&state));
        if !id.ends_with("composter") {
            return;
        }
        make_bread(mob);
        compost_items(level, mob, g.pos, state);
    })))
}

/// `WorkAtComposter.makeBread` — <=36 хлеба: wheat/3 → bread.
fn make_bread(mob: &mut Mob) {
    let bread = Identifier::minecraft("bread");
    let wheat = Identifier::minecraft("wheat");
    let bread_count = mob.get_inventory_ref().map(|i| i.count_item(&bread)).unwrap_or(0);
    if bread_count > 36 {
        return;
    }
    let wheat_count = mob.get_inventory_ref().map(|i| i.count_item(&wheat)).unwrap_or(0);
    let min = (3i32).min(wheat_count / 3);
    if min == 0 {
        return;
    }
    let i3 = min * 3;
    mob.get_inventory().remove_item_type(&wheat, i3);
    let leftover = mob
        .get_inventory()
        .add_item(item_stack_of(&Identifier::minecraft("bread"), min));
    if !leftover.is_empty() {
        mob.living.entity.spawn_at_location_with_offset(leftover, 0.5);
    }
}

/// `WorkAtComposter.compostItems` — семена → компостер
/// (level 0..8, каждый предмет +0.3 chance → +1 level;
/// упрощённо детерминированно по random).
fn compost_items(
    level: &Arc<dyn LevelReader>,
    mob: &mut Mob,
    pos: BlockPos,
    state: crate::world::level::block::state::block_state::BlockState,
) {
    let seeds = [
        Identifier::minecraft("wheat_seeds"),
        Identifier::minecraft("beetroot_seeds"),
    ];
    let mut level_value: i32 = match state.values.get_by_name("level") {
        Some(crate::world::level::block::state::property::StateValue::Int(v)) => v,
        _ => 0,
    };
    let mut i = 20;
    // Инвентарь с конца.
    let size = mob.get_inventory_ref().map(|inv| inv.get_container_size()).unwrap_or(0);
    'outer: for i2 in (0..size).rev() {
        if i <= 0 {
            break;
        }
        let item = mob.get_inventory().get_item(i2);
        if item.is_empty() {
            continue;
        }
        let Some(id) = item.item_identifier() else { continue };
        let Some(index) = seeds.iter().position(|s| *s == id) else {
            continue;
        };
        let _ = index;
        let count = item.get_count();
        // ComposterBlock.insertItem — каждый предмет шанс 0.3
        // (random); при level==7 → extractProduce (bone meal).
        let mut to_insert = count.min(i);
        let mut remaining = to_insert;
        while remaining > 0 {
            let chance = {
                let mut r = mob.random.borrow_mut();
                r.next_float()
            };
            if chance < 0.3 {
                level_value += 1;
                remaining -= 1;
                i -= 1;
            } else {
                // Неудача — предмет остаётся (Java: break по
                // количеству попыток). Упрощённо: выходим.
                break;
            }
            if level_value >= 8 {
                // extractProduce → bone meal dроп + level=0.
                mob.get_inventory()
                    .add_item(item_stack_of(&Identifier::minecraft("bone_meal"), 1));
                level_value = 0;
                break 'outer;
            }
        }
        to_insert = remaining;
        let _ = to_insert;
        // Уменьшаем стек на реально внесённое (count - remaining
        // не считаем точно — упрощённо уменьшаем на i-разницу).
        let inserted = count.min(i) - remaining.max(0);
        let _ = inserted;
        if !item.is_empty() {
            let mut stack = mob.get_inventory().get_item(i2);
            // shrink на число внесённых (min(count,i) - remaining)
            let used = (count.min(20) - remaining).max(0);
            if used > 0 {
                stack.shrink(used);
                mob.get_inventory().set_item(i2, stack);
            }
        }
    }
    let old_level: i32 = match state.values.get_by_name("level") {
        Some(crate::world::level::block::state::property::StateValue::Int(v)) => v,
        _ => 0,
    };
    let changed = level_value != old_level;
    if changed {
        let mut values = state.values.clone();
        values.set_erased(
            "level",
            crate::world::level::block::state::property::StateValue::Int(level_value),
        );
        let new_state = crate::world::level::block::state::block_state::BlockState::new(
            state.block.clone(),
            values,
        );
        level.set_block_state(pos, &new_state);
        level.level_event(4200, pos, if changed { 1 } else { 0 }); // LevelEvent.COMPOSTER_FILL
    }
}

// ---------- UseBonemeal ----------

/// `UseBonemeal` — 80-тиковая работа: bonemeal по грядкам.
pub fn use_bonemeal() -> impl BehaviorControl {
    struct UseBonemeal {
        base: BehaviorBase,
        next_work_cycle_time: i64,
        last_bonemealing_session: i64,
        time_worked_so_far: i32,
        crop_pos: Option<BlockPos>,
    }
    impl BehaviorControl for UseBonemeal {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "UseBonemeal".to_string() }

        fn check_extra_start_conditions(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            if mob.living.entity.tick_count % 10 != 0 {
                return false;
            }
            if self.last_bonemealing_session != 0
                && self.last_bonemealing_session + 160 > mob.living.entity.tick_count as i64
            {
                return false;
            }
            let bone_meal = Identifier::minecraft("bone_meal");
            let has = mob.get_inventory_ref().map(|i| i.count_item(&bone_meal) > 0).unwrap_or(false);
            if !has {
                return false;
            }
            self.crop_pos = self.pick_next_target(level, mob);
            self.crop_pos.is_some()
        }

        fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) -> bool {
            self.time_worked_so_far < 80 && self.crop_pos.is_some()
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            self.set_current_crop_as_target(mob);
            mob.living.set_item_slot(
                crate::entity::equipment_slot::EquipmentSlot::Mainhand,
                item_stack_of(&Identifier::minecraft("bone_meal"), 1),
            );
            self.next_work_cycle_time = game_time;
            self.time_worked_so_far = 0;
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            mob.living.set_item_slot(
                crate::entity::equipment_slot::EquipmentSlot::Mainhand,
                ItemStack::empty(),
            );
            self.last_bonemealing_session = mob.living.entity.tick_count as i64;
        }

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            let Some(block_pos) = self.crop_pos else { return };
            if game_time < self.next_work_cycle_time {
                return;
            }
            if !pos_closer_to_center(block_pos, mob.living.entity.position(), 1.0) {
                return;
            }
            // Найти bone meal в инвентаре.
            let bone_meal = Identifier::minecraft("bone_meal");
            let has = mob.get_inventory_ref().map(|i| i.count_item(&bone_meal) > 0).unwrap_or(false);
            if !has {
                return;
            }
            // BoneMealItem.growCrop — crop grow; упрощённо:
            // если у блока есть "age" свойство — +1 (max 7),
            // иначе bonemeal не действует.
            if grow_crop_at(level, mob, block_pos) {
                level.level_event(1505, block_pos, 15); // LevelEvent.PARTICLES_AND_SOUND_PLANT_GROWTH
                self.crop_pos = self.pick_next_target(level, mob);
                self.set_current_crop_as_target(mob);
                self.next_work_cycle_time = game_time + 40;
            }
            self.time_worked_so_far += 1;
        }
    }
    impl UseBonemeal {
        /// `pickNextTarget` — случайная валидная культура в
        /// радиусе 1 (резервирующий выбор).
        fn pick_next_target(&self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> Option<BlockPos> {
            let mut chosen: Option<BlockPos> = None;
            let mut count = 0i32;
            let base = mob.block_position();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let p = base.offset(dx, dy, dz);
                        if self.valid_pos(level, p) {
                            count += 1;
                            let mut r = mob.random.borrow_mut();
                            if r.next_int_bound(count) == 0 {
                                chosen = Some(p);
                            }
                        }
                    }
                }
            }
            chosen
        }

        /// `validPos` — CropBlock && !isMaxAge. Упрощённо:
        /// блок имеет int-свойство "age" со значением <7
        /// (большинство ванильных культур maxAge=7).
        fn valid_pos(&self, level: &Arc<dyn LevelReader>, pos: BlockPos) -> bool {
            let Some(state) = level.get_block_state(pos) else { return false };
            crop_age(&state).map(|a| a < 7).unwrap_or(false)
        }

        fn set_current_crop_as_target(&self, mob: &mut Mob) {
            if let Some(pos) = self.crop_pos {
                let tracker = Rc::new(memory::BlockPosTracker::new(pos))
                    as Rc<dyn memory::PositionTracker>;
                mob.living.brain_set_value(&keys::look_target(), tracker.clone());
                mob.living.brain_set_value(
                    &keys::walk_target(),
                    WalkTarget::of_tracker(tracker, 0.5, 1),
                );
            }
        }
    }
    UseBonemeal {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[
                (keys::look_target(), MemoryStatus::ValueAbsent),
                (keys::walk_target(), MemoryStatus::ValueAbsent),
            ]),
            80,
        ),
        next_work_cycle_time: 0,
        last_bonemealing_session: 0,
        time_worked_so_far: 0,
        crop_pos: None,
    }
}

/// `crop_age` — значение свойства "age" (int) у BlockState.
fn crop_age(state: &crate::world::level::block::state::block_state::BlockState) -> Option<i32> {
    match state.values.get_by_name("age") {
        Some(crate::world::level::block::state::property::StateValue::Int(v)) => Some(v),
        _ => None,
    }
}

/// `BoneMealItem.growCrop` — bonemeal на культуру: растёт на
/// 1-3 стадии (vanilla: random 1..3 пока не max). Возвращает
/// true если что-то выросло. Также расходует bone meal из
/// инвентаря жителя.
fn grow_crop_at(level: &Arc<dyn LevelReader>, mob: &mut Mob, pos: BlockPos) -> bool {
    let Some(state) = level.get_block_state(pos) else { return false };
    let Some(age) = crop_age(&state) else { return false };
    if age >= 7 {
        return false;
    }
    let mut r = mob.random.borrow_mut();
    let grow_by = 1 + r.next_int_bound(3);
    let new_age = (age + grow_by).min(7);
    drop(r);
    let mut values = state.values.clone();
    values.set_erased(
        "age",
        crate::world::level::block::state::property::StateValue::Int(new_age),
    );
    let new_state = crate::world::level::block::state::block_state::BlockState::new(
        state.block.clone(),
        values,
    );
    level.set_block_state(pos, &new_state);
    // Расход bone meal.
    let bone_meal = Identifier::minecraft("bone_meal");
    mob.get_inventory().remove_item_type(&bone_meal, 1);
    true
}

// ---------- HarvestFarmland ----------

/// `HarvestFarmland` — farmer: harvest mature crops на
/// farmland вокруг (200 тиков, speed 0.5).
pub fn harvest_farmland() -> impl BehaviorControl {
    struct HarvestFarmland {
        base: BehaviorBase,
        next_ok_start_time: i64,
        time_worked_so_far: i32,
        above_farmland_pos: Option<BlockPos>,
        valid_farmland: Vec<BlockPos>,
    }
    impl BehaviorControl for HarvestFarmland {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "HarvestFarmland".to_string() }

        fn check_extra_start_conditions(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            // GameRules.MOB_GRIEFING — level-хук; default true.
            if !level.mob_griefing() {
                return false;
            }
            let data = mob.get_villager_data().clone();
            let is_farmer = villager_data::profession_is(
                &data.profession,
                &villager_profession::farmer(),
            );
            if !is_farmer {
                return false;
            }
            self.valid_farmland.clear();
            let base = mob.block_position();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let p = base.offset(dx, dy, dz);
                        if self.valid_pos(level, p) {
                            self.valid_farmland.push(p);
                        }
                    }
                }
            }
            self.above_farmland_pos = self.get_valid_farmland(level, mob);
            self.above_farmland_pos.is_some()
        }

        fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) -> bool {
            self.time_worked_so_far < 200 && self.above_farmland_pos.is_some()
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            self.next_ok_start_time = game_time;
            self.time_worked_so_far = 0;
            if let Some(pos) = self.above_farmland_pos {
                mob.living.brain_set_value(
                    &keys::look_target(),
                    Rc::new(memory::BlockPosTracker::new(pos))
                        as Rc<dyn memory::PositionTracker>,
                );
                mob.living.brain_set_value(
                    &keys::walk_target(),
                    WalkTarget::new_block(pos, 0.5, 1),
                );
            }
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) {
            self.above_farmland_pos = None;
        }

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            let Some(pos) = self.above_farmland_pos else { return };
            if game_time < self.next_ok_start_time {
                return;
            }
            if !pos_closer_to_center(pos, mob.living.entity.position(), 1.0) {
                return;
            }
            // Harvest: crop age>=7 → сломать, дроп через loot.
            if harvest_crop_at(level, mob, pos) {
                level.game_event(&crate::world::level::gameevent::game_event::block_change(), mob.living.entity.block_position());
                self.above_farmland_pos = self.get_valid_farmland(level, mob);
                self.next_ok_start_time = game_time + 20;
            }
            self.time_worked_so_far += 1;
        }
    }
    impl HarvestFarmland {
        fn get_valid_farmland(&self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> Option<BlockPos> {
            if self.valid_farmland.is_empty() {
                return None;
            }
            let idx = {
                let mut r = mob.random.borrow_mut();
                r.next_int_bound(self.valid_farmland.len() as i32) as usize
            };
            self.valid_farmland.get(idx).copied()
        }

        /// `validPos`: над блоком — farmland, на farmland —
        /// mature crop (age==7) ИЛИ grass-like. Упрощённо:
        /// pos.below() = farmland, pos = crop с age>=7.
        fn valid_pos(&self, level: &Arc<dyn LevelReader>, pos: BlockPos) -> bool {
            let below = level.get_block_state(pos.below());
            let is_farmland = below
                .map(|s| block_id_path(&s).ends_with("farmland"))
                .unwrap_or(false);
            if !is_farmland {
                return false;
            }
            level
                .get_block_state(pos)
                .and_then(|s| crop_age(&s))
                .map(|a| a >= 7)
                .unwrap_or(false)
        }
    }
    HarvestFarmland {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[
                (keys::look_target(), MemoryStatus::ValueAbsent),
                (keys::walk_target(), MemoryStatus::ValueAbsent),
                (keys::secondary_job_site(), MemoryStatus::ValuePresent),
            ]),
            200,
        ),
        next_ok_start_time: 0,
        time_worked_so_far: 0,
        above_farmland_pos: None,
        valid_farmland: Vec::new(),
    }
}

/// `HarvestFarmland.harvest` — сломать зрелую культуру:
/// block→air + дроп (семена/продукт через loot — упрощённо:
/// дропаем тот же предмет если известен, иначе пропускаем).
fn harvest_crop_at(level: &Arc<dyn LevelReader>, mob: &mut Mob, pos: BlockPos) -> bool {
    let Some(state) = level.get_block_state(pos) else { return false };
    if crop_age(&state).map(|a| a < 7).unwrap_or(true) {
        return false;
    }
    let air = crate::world::level::block::state::block_state::BlockState::air();
    level.set_block_state(pos, &air);
    // Дроп: wheat/beetroot/carrot/potato по id блока —
    // базовый маппинг ванильных культур.
    let id = format!("minecraft:{}", block_id_path(&state));
    let drop_id = match id.rsplit(':').next().unwrap_or("") {
        "wheat" => Some("wheat"),
        "beetroots" => Some("beetroot"),
        "carrots" => Some("carrot"),
        "potatoes" => Some("potato"),
        _ => None,
    };
    let drop_stack = if let Some(d) = drop_id {
        let mut r = mob.random.borrow_mut();
        let count = 1 + r.next_int_bound(2);
        drop(r);
        item_stack_of(&Identifier::minecraft(d), count)
    } else {
        ItemStack::empty()
    };
    if !drop_stack.is_empty() {
        mob.living.entity.spawn_at_location(drop_stack);
    }
    // Семена дополнительно (wheat → wheat_seeds 0-3).
    if id.ends_with("wheat") {
        let mut r = mob.random.borrow_mut();
        let seeds = r.next_int_bound(4);
        drop(r);
        if seeds > 0 {
            mob.living.entity.spawn_at_location(item_stack_of(
                &Identifier::minecraft("wheat_seeds"),
                seeds,
            ));
        }
    }
    true
}

// ---------- SleepInBed ----------

/// `SleepInBed` — villager/животное идёт спать в HOME-кровать.
pub fn sleep_in_bed() -> impl BehaviorControl {
    struct SleepInBed {
        base: BehaviorBase,
        next_ok_start_time: i64,
    }
    impl BehaviorControl for SleepInBed {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "SleepInBed".to_string() }

        fn check_extra_start_conditions(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            if mob.living.entity.is_passenger() {
                return false;
            }
            let Some(gp): Option<GlobalPos> = mob.living.brain_get(&keys::home()) else {
                return false;
            };
            if !same_dimension(level, &gp) {
                return false;
            }
            let last_woken: Option<i64> = mob.living.brain_get(&keys::last_woken());
            if let Some(last_woken) = last_woken {
                let l = level.game_time() - last_woken;
                if l > 0 && l < 100 {
                    return false;
                }
            }
            let bp = gp.pos;
            pos_closer_to_center(bp, mob.living.entity.position(), 2.0)
                && block_is_tag(level, bp, &BlockTags::beds())
                && !bed_occupied(level, bp)
        }

        fn can_still_use(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
            let Some(gp): Option<GlobalPos> = mob.living.brain_get(&keys::home()) else {
                return false;
            };
            let rest = crate::entity::schedule::activity::activity_by_name("rest");
            let is_rest = rest
                .as_ref()
                .map(|a| {
                    mob.living
                        .brain_get_active_activity()
                        .map(|act| act.id == a.id)
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            let bp = gp.pos;
            is_rest
                && mob.living.entity.get_y() > bp.get_y() as f64 + 0.4
                && pos_closer_to_center(bp, mob.living.entity.position(), 1.14)
        }

        fn start(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            if game_time <= self.next_ok_start_time {
                return;
            }
            // DOORS_TO_CLOSE → closeDoorsThatIHaveOpenedOrPassedThrough.
            let doors_mem: Option<HashSet<GlobalPos>> =
                mob.living.brain_get(&keys::doors_to_close());
            if let Some(doors) = doors_mem {
                let nle: Option<Vec<crate::world::entity::living_entity::LivingEntity>> =
                    mob.living.brain_get(&keys::nearest_living_entities());
                let remaining = close_doors_that_i_have_opened_or_passed_through(
                    level,
                    &mob.living.entity,
                    None,
                    None,
                    doors,
                    nle,
                );
                mob.living.brain_set_value(&keys::doors_to_close(), remaining);
            }
            let Some(gp): Option<GlobalPos> = mob.living.brain_get(&keys::home()) else {
                return;
            };
            mob.living.start_sleeping(gp.pos);
        }

        fn timed_out(&self, _game_time: i64) -> bool {
            false
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            if mob.living.is_sleeping_entity() {
                mob.living.stop_sleeping();
                self.next_ok_start_time = game_time + 40;
            }
        }
    }
    SleepInBed {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[
                (keys::home(), MemoryStatus::ValuePresent),
                (keys::last_woken(), MemoryStatus::Registered),
            ]),
            60,
        ),
        next_ok_start_time: 0,
    }
}

/// `bedIsOccupied` — BEDS + occupied=true.
fn bed_occupied(level: &Arc<dyn LevelReader>, pos: BlockPos) -> bool {
    level
        .get_block_state(pos)
        .map(|s| {
            s.is_block_tag(&BlockTags::beds())
                && matches!(
                    s.values.get_by_name("occupied"),
                    Some(crate::world::level::block::state::property::StateValue::Bool(true))
                )
        })
        .unwrap_or(false)
}

// ---------- JumpOnBed ----------

/// `JumpOnBed(speedModifier)` — baby mobs прыгают на кровать.
pub fn jump_on_bed(speed_modifier: f32) -> impl BehaviorControl {
    struct JumpOnBed {
        base: BehaviorBase,
        speed_modifier: f32,
        target_bed: Option<BlockPos>,
        remaining_time_to_reach_bed: i32,
        remaining_jumps: i32,
        remaining_cooldown: i32,
    }
    impl BehaviorControl for JumpOnBed {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "JumpOnBed".to_string() }

        fn check_extra_start_conditions(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            mob.living.entity.is_baby() && self.near_bed(level, mob)
        }

        fn can_still_use(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
            mob.living.entity.is_baby()
                && self.target_bed.is_some()
                && self
                    .target_bed
                    .map(|p| self.is_bed(level, p))
                    .unwrap_or(false)
                && !self.tired_of_walking(level, mob)
                && !self.tired_of_jumping(level, mob)
        }

        fn timed_out(&self, _game_time: i64) -> bool {
            false
        }

        fn start(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            if let Some(bp) = self.get_nearest_bed(mob) {
                self.target_bed = Some(bp);
                self.remaining_time_to_reach_bed = 100;
                self.remaining_jumps = 3 + {
                    let mut r = mob.random.borrow_mut();
                    r.next_int_bound(4)
                };
                self.remaining_cooldown = 0;
                self.start_walking_towards_bed(mob, bp);
            }
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) {
            self.target_bed = None;
            self.remaining_time_to_reach_bed = 0;
            self.remaining_jumps = 0;
            self.remaining_cooldown = 0;
        }

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            if !self.on_or_over_bed(level, mob) {
                self.remaining_time_to_reach_bed -= 1;
            } else if self.remaining_cooldown > 0 {
                self.remaining_cooldown -= 1;
            } else if self.on_bed_surface(level, mob) {
                mob.jump_control.jump();
                self.remaining_jumps -= 1;
                self.remaining_cooldown = 5;
            }
        }
    }
    impl JumpOnBed {
        fn start_walking_towards_bed(&self, mob: &mut Mob, pos: BlockPos) {
            mob.living.brain_set_value(
                &keys::walk_target(),
                WalkTarget::new_block(pos, self.speed_modifier, 0),
            );
        }
        fn near_bed(&self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            self.on_or_over_bed(level, mob) || self.get_nearest_bed(mob).is_some()
        }
        fn on_or_over_bed(&self, level: &Arc<dyn LevelReader>, mob: &Mob) -> bool {
            let bp = mob.block_position();
            self.is_bed(level, bp) || self.is_bed(level, bp.below())
        }
        fn on_bed_surface(&self, level: &Arc<dyn LevelReader>, mob: &Mob) -> bool {
            self.is_bed(level, mob.block_position())
        }
        fn is_bed(&self, level: &Arc<dyn LevelReader>, pos: BlockPos) -> bool {
            block_is_tag(level, pos, &BlockTags::beds())
        }
        fn get_nearest_bed(&self, mob: &Mob) -> Option<BlockPos> {
            mob.living.brain_get(&keys::nearest_bed())
        }
        fn tired_of_walking(&self, level: &Arc<dyn LevelReader>, mob: &Mob) -> bool {
            !self.on_or_over_bed(level, mob) && self.remaining_time_to_reach_bed <= 0
        }
        fn tired_of_jumping(&self, level: &Arc<dyn LevelReader>, mob: &Mob) -> bool {
            self.on_or_over_bed(level, mob) && self.remaining_jumps <= 0
        }
    }
    JumpOnBed {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[
                (keys::nearest_bed(), MemoryStatus::ValuePresent),
                (keys::walk_target(), MemoryStatus::ValueAbsent),
            ]),
            60,
        ),
        speed_modifier,
        target_bed: None,
        remaining_time_to_reach_bed: 0,
        remaining_jumps: 0,
        remaining_cooldown: 0,
    }
}

// ---------- InteractWithDoor ----------

/// `InteractWithDoor.create()` — Behavior на PATH+DOORS_TO_CLOSE.
pub fn interact_with_door() -> impl BehaviorControl {
    struct InteractWithDoor {
        base: BehaviorBase,
        last_node: Option<crate::world::level::pathfinder::node::NodeRef>,
        cooldown: i32,
    }
    impl BehaviorControl for InteractWithDoor {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "InteractWithDoor".to_string() }

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let Some(path) = mob
                .living
                .brain_get::<crate::world::level::pathfinder::path::Path>(&keys::path())
            else {
                return;
            };
            if path.not_started() || path.is_done() {
                return;
            }
            let next_node = Some(path.next_node());
            let prev_node = path.previous_node();
            if self.last_node.is_some() && node_eq(&self.last_node, &next_node) {
                self.cooldown = 20;
            } else if {
                self.cooldown -= 1;
                self.cooldown > 0
            } {
                return;
            }
            self.last_node = next_node.clone();
            if let Some(prev) = &prev_node {
                let bp = node_block_pos(prev);
                if let Some(state) = level.get_block_state(bp) {
                    if is_door_state(&state) && !state.is_open_property() {
                        set_door_open(level, bp, true);
                        remember_door_to_close(mob, level, bp);
                    }
                }
            }
            if let Some(next) = &next_node {
                let bp = node_block_pos(next);
                if let Some(state) = level.get_block_state(bp) {
                    if is_door_state(&state) && !state.is_open_property() {
                        set_door_open(level, bp, true);
                        remember_door_to_close(mob, level, bp);
                    }
                }
            }
            let doors: Option<HashSet<GlobalPos>> = mob.living.brain_get(&keys::doors_to_close());
            if let Some(set) = doors {
                let nle: Option<Vec<crate::world::entity::living_entity::LivingEntity>> =
                    mob.living.brain_get(&keys::nearest_living_entities());
                let remaining = close_doors_that_i_have_opened_or_passed_through(
                    level,
                    &mob.living.entity,
                    prev_node.as_ref(),
                    next_node.as_ref(),
                    set,
                    nle,
                );
                mob.living.brain_set_value(&keys::doors_to_close(), remaining);
            }
        }
    }
    InteractWithDoor {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[
                (keys::path(), MemoryStatus::ValuePresent),
                (keys::doors_to_close(), MemoryStatus::Registered),
                (keys::nearest_living_entities(), MemoryStatus::Registered),
            ]),
            60,
        ),
        last_node: None,
        cooldown: 0,
    }
}

fn node_eq(
    a: &Option<crate::world::level::pathfinder::node::NodeRef>,
    b: &Option<crate::world::level::pathfinder::node::NodeRef>,
) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => x.borrow().as_block_pos() == y.borrow().as_block_pos(),
        (None, None) => true,
        _ => false,
    }
}

fn node_block_pos(n: &crate::world::level::pathfinder::node::NodeRef) -> BlockPos {
    n.borrow().as_block_pos()
}

/// `state.is(MOB_INTERACTABLE_DOORS, DoorBlock)` — деревянные
/// двери (тег wooden_doors/doors + свойство open).
fn is_door_state(state: &crate::world::level::block::state::block_state::BlockState) -> bool {
    state.is_open_property()
        && (state.is_block_tag(&BlockTags::wooden_doors())
            || state.is_block_tag(&BlockTags::doors()))
}

/// `DoorBlock.setOpen(entity, level, state, pos, open)` —
/// переключаем свойство "open" через state values.
fn set_door_open(level: &Arc<dyn LevelReader>, pos: BlockPos, open: bool) {
    let Some(state) = level.get_block_state(pos) else { return };
    let mut values = state.values.clone();
    values.set_erased(
        "open",
        crate::world::level::block::state::property::StateValue::Bool(open),
    );
    let new_state = crate::world::level::block::state::block_state::BlockState::new(
        state.block.clone(),
        values,
    );
    level.set_block_state(pos, &new_state);
}

fn remember_door_to_close(mob: &mut Mob, level: &Arc<dyn LevelReader>, pos: BlockPos) {
    let gp = GlobalPos::of(
        crate::resources::ResourceKey::<crate::world::level::LevelKey>::create_with_registry_name(
            crate::core::registry::registries::dimension(),
            level
                .dimension_id()
                .cloned()
                .unwrap_or_else(|| Identifier::with_default_namespace("overworld").unwrap()),
        ),
        pos,
    );
    let mut set: HashSet<GlobalPos> = mob
        .living
        .brain_get(&keys::doors_to_close())
        .unwrap_or_default();
    set.insert(gp);
    mob.living.brain_set_value(&keys::doors_to_close(), set);
}

/// `InteractWithDoor.closeDoorsThatIHaveOpenedOrPassedThrough`.
#[allow(clippy::too_many_arguments)]
pub fn close_doors_that_i_have_opened_or_passed_through(
    level: &Arc<dyn LevelReader>,
    entity: &crate::world::entity::entity::Entity,
    previous: Option<&crate::world::level::pathfinder::node::NodeRef>,
    next: Option<&crate::world::level::pathfinder::node::NodeRef>,
    door_positions: HashSet<GlobalPos>,
    nearest_living_entities: Option<Vec<crate::world::entity::living_entity::LivingEntity>>,
) -> HashSet<GlobalPos> {
    let entity_pos = entity.position();
    let mut doors = door_positions;
    doors.retain(|gp| {
        let bp = gp.pos;
        if previous.map(|p| node_block_pos(p) == bp).unwrap_or(false)
            || next.map(|p| node_block_pos(p) == bp).unwrap_or(false)
        {
            return true;
        }
        if !same_dimension(level, gp) || !pos_closer_to_center(bp, entity_pos, 3.0) {
            return false;
        }
        let Some(state) = level.get_block_state(bp) else { return false };
        if !is_door_state(&state) {
            return false;
        }
        if !state.is_open_property() {
            return false;
        }
        if are_other_mobs_coming_through_door(entity, bp, nearest_living_entities.as_ref()) {
            return false;
        }
        set_door_open(level, bp, false);
        false
    });
    doors
}

fn are_other_mobs_coming_through_door(
    entity: &crate::world::entity::entity::Entity,
    pos: BlockPos,
    nearest: Option<&Vec<crate::world::entity::living_entity::LivingEntity>>,
) -> bool {
    let Some(list) = nearest else { return false };
    list.iter()
        .filter(|e| {
            entity_type_path(&e.entity) == entity_type_path(entity)
                && pos_closer_to_center(pos, e.entity.position(), 2.0)
        })
        .any(|e| is_mob_coming_through_door(e, pos))
}

fn is_mob_coming_through_door(
    living: &crate::world::entity::living_entity::LivingEntity,
    pos: BlockPos,
) -> bool {
    let Some(brain) = living.brain.clone() else { return false };
    let Some(path) = brain.get_memory::<crate::world::level::pathfinder::path::Path>(&keys::path()) else {
        return false;
    };
    if path.is_done() {
        return false;
    }
    let prev = path.previous_node().map(|n| node_block_pos(&n));
    let next = Some(node_block_pos(&path.next_node()));
    prev == Some(pos) || next == Some(pos)
}

// ---------- LookAndFollowTradingPlayerSink ----------

/// `LookAndFollowTradingPlayerSink(speedModifier)` —
/// duration Integer.MAX_VALUE, timedOut=false.
pub fn look_and_follow_trading_player_sink(speed_modifier: f32) -> impl BehaviorControl {
    struct LookAndFollowTradingPlayerSink {
        base: BehaviorBase,
        speed_modifier: f32,
    }
    impl BehaviorControl for LookAndFollowTradingPlayerSink {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "LookAndFollowTradingPlayerSink".to_string() }

        fn check_extra_start_conditions(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            let player = mob.get_trading_player().and_then(|u| level.entity_by_uuid(u));
            player.is_some()
                && mob.living.entity.is_alive()
                && !mob.living.entity.is_in_water()
                && !mob.living.hurt_marked
                && player
                    .as_ref()
                    .map(|p| p.position().distance_to_sqr(&mob.living.entity.position()) <= 16.0)
                    .unwrap_or(false)
        }

        fn can_still_use(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
            self.check_extra_start_conditions(level, mob)
        }

        fn timed_out(&self, _game_time: i64) -> bool {
            false
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            self.follow_player(mob);
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            mob.living.brain_erase(&keys::walk_target());
            mob.living.brain_erase(&keys::look_target());
        }

        fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            self.follow_player(mob);
        }
    }
    impl LookAndFollowTradingPlayerSink {
        fn follow_player(&self, mob: &mut Mob) {
            let player_uuid = mob.get_trading_player();
            let player = player_uuid
                .and_then(|u| mob.living.entity.level.as_ref().and_then(|l| l.entity_by_uuid(u)));
            if let Some(p) = player {
                mob.living.brain_set_value(
                    &keys::walk_target(),
                    WalkTarget::of_tracker(
                        Rc::new(memory::EntityTracker::new(p.clone(), false))
                            as Rc<dyn memory::PositionTracker>,
                        self.speed_modifier,
                        2,
                    ),
                );
                mob.living.brain_set_value(
                    &keys::look_target(),
                    Rc::new(memory::EntityTracker::new(p, true))
                        as Rc<dyn memory::PositionTracker>,
                );
            }
        }
    }
    LookAndFollowTradingPlayerSink {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[
                (keys::walk_target(), MemoryStatus::Registered),
                (keys::look_target(), MemoryStatus::Registered),
            ]),
            i32::MAX,
        ),
        speed_modifier,
    }
}

// ---------- TradeWithVillager ----------

/// `TradeWithVillager` — обмен предметами между жителями.
pub fn trade_with_villager() -> impl BehaviorControl {
    struct TradeWithVillager {
        base: BehaviorBase,
        trades: HashSet<Identifier>,
    }
    impl BehaviorControl for TradeWithVillager {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "TradeWithVillager".to_string() }

        fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            mob.living
                .brain
                .as_ref()
                .map(|b| behavior_utils::target_is_valid(b, &keys::interaction_target(), "villager"))
                .unwrap_or(false)
        }

        fn can_still_use(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
            self.check_extra_start_conditions(level, mob)
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let target: Option<crate::world::entity::living_entity::LivingEntity> =
                mob.living.brain_get(&keys::interaction_target());
            if let Some(t) = target {
                behavior_utils::lock_gaze_and_walk_to_each_other_living(
                    &mob.living.clone(),
                    &t,
                    0.5,
                    2,
                );
                // figureOutWhatIAmWillingToTrade — partner
                // profession недоступна (LivingEntity-клон);
                // оставляем пустым — обмен только food.
                self.trades = HashSet::new();
            }
        }

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            let target: Option<crate::world::entity::living_entity::LivingEntity> =
                mob.living.brain_get(&keys::interaction_target());
            let Some(t) = target else { return };
            if mob.living.entity.position().distance_to_sqr(&t.entity.position()) > 5.0 {
                return;
            }
            behavior_utils::lock_gaze_and_walk_to_each_other_living(
                &mob.living.clone(),
                &t,
                0.5,
                2,
            );
            // gossip(level, target, gameTime) — target это клон
            // LivingEntity без Mob-полей; decay только self.
            if let Some(g) = &mut mob.gossips {
                g.decay();
            }
            let is_farmer = {
                let data = mob.get_villager_data().clone();
                villager_data::profession_is(&data.profession, &villager_profession::farmer())
            };
            let excess = mob.villager_food_points() >= 12;
            if excess && is_farmer {
                throw_half_stack(mob, villager_food_item_ids(), Some(t.entity.position()));
            }
            let _ = (level, game_time, t);
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            mob.living.brain_erase(&keys::interaction_target());
        }
    }
    TradeWithVillager {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[
                (keys::interaction_target(), MemoryStatus::ValuePresent),
                (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
            ]),
            60,
        ),
        trades: HashSet::new(),
    }
}

/// FOOD_POINTS предметы жителя (Villager.FOOD_POINTS).
fn villager_food_item_ids() -> HashSet<Identifier> {
    HashSet::from([
        Identifier::minecraft("bread"),
        Identifier::minecraft("potato"),
        Identifier::minecraft("carrot"),
        Identifier::minecraft("beetroot"),
    ])
}

/// `TradeWithVillager.throwHalfStack` — отдать половину стека
/// предметов из набора `stack` ближайшему entity.
/// `target_pos` — позиция получателя (None → не бросаем).
fn throw_half_stack(
    villager: &mut Mob,
    stack: HashSet<Identifier>,
    target_pos: Option<Vec3>,
) {
    let Some(pos) = target_pos else { return };
    let size = villager.get_inventory_ref().map(|i| i.get_container_size()).unwrap_or(0);
    let mut item_stack = ItemStack::empty();
    'outer: for i in 0..size {
        let item = villager.get_inventory().get_item(i);
        if item.is_empty() {
            continue;
        }
        let Some(id) = item.item_identifier() else { continue };
        if !stack.contains(&id) {
            continue;
        }
        let count = item.get_count();
        let max = item.get_max_stack_size();
        let take = if count > max / 2 {
            count / 2
        } else if count > 24 {
            count - 24
        } else {
            continue;
        };
        let mut slot_item = villager.get_inventory().get_item(i);
        slot_item.shrink(take);
        villager.get_inventory().set_item(i, slot_item);
        let mut ns = item.copy();
        ns.set_count(take);
        item_stack = ns;
        break 'outer;
    }
    if !item_stack.is_empty() {
        // throwItem — дроп в сторону цели; spawn_at_location
        // кладёт у ног (runtime бросок — при entity-tick).
        let _ = pos;
        villager.living.entity.spawn_at_location(item_stack);
    }
}

// ---------- VillagerMakeLove ----------

/// `VillagerMakeLove` — 350-350 тиков, рождение детёныша.
pub fn villager_make_love() -> impl BehaviorControl {
    struct VillagerMakeLove {
        base: BehaviorBase,
        birth_timestamp: i64,
    }
    impl BehaviorControl for VillagerMakeLove {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "VillagerMakeLove".to_string() }

        fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            self.is_breeding_possible(mob)
        }

        fn can_still_use(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) -> bool {
            game_time <= self.birth_timestamp && self.is_breeding_possible(mob)
        }

        fn start(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            let target: Option<crate::world::entity::living_entity::LivingEntity> =
                mob.living.brain_get(&keys::breed_target());
            if let Some(t) = &target {
                behavior_utils::lock_gaze_and_walk_to_each_other_living(&mob.living.clone(), t, 0.5, 2);
                level.broadcast_entity_event(t.entity.id, crate::world::entity::entity_event::IN_LOVE_HEARTS);
                level.broadcast_entity_event(mob.living.entity.id, crate::world::entity::entity_event::IN_LOVE_HEARTS);
                let i = 275 + {
                    let mut r = mob.random.borrow_mut();
                    r.next_int_bound(50)
                };
                self.birth_timestamp = game_time + i as i64;
            }
        }

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            let target: Option<crate::world::entity::living_entity::LivingEntity> =
                mob.living.brain_get(&keys::breed_target());
            let Some(villager) = target else { return };
            if mob.living.entity.position().distance_to_sqr(&villager.entity.position()) > 5.0 {
                return;
            }
            behavior_utils::lock_gaze_and_walk_to_each_other_living(&mob.living, &villager, 0.5, 2);
            if game_time >= self.birth_timestamp {
                mob.eat_and_digest_food();
                // partner-side: только entity-поля (age/love).
                self.try_to_give_birth(level, mob, &villager);
            } else {
                let mut r = mob.random.borrow_mut();
                if r.next_int_bound(35) == 0 {
                    drop(r);
                    level.broadcast_entity_event(villager.entity.id, crate::world::entity::entity_event::LOVE_HEARTS);
                    level.broadcast_entity_event(mob.living.entity.id, crate::world::entity::entity_event::LOVE_HEARTS);
                }
            }
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            mob.living.brain_erase(&keys::breed_target());
        }
    }
    impl VillagerMakeLove {
        fn is_breeding_possible(&self, mob: &Mob) -> bool {
            let target: Option<crate::world::entity::living_entity::LivingEntity> =
                mob.living.brain_get(&keys::breed_target());
            let Some(t) = target else { return false };
            if !is_villager_entity(&t) {
                return false;
            }
            let target_ok = mob
                .living
                .brain
                .as_ref()
                .map(|b| behavior_utils::target_is_valid(b, &keys::breed_target(), "villager"))
                .unwrap_or(false);
            if !target_ok {
                return false;
            }
            mob.can_breed_villager()
                && t.entity.is_adult()
                && !t.entity.is_in_love()
        }

        fn try_to_give_birth(
            &self,
            level: &Arc<dyn LevelReader>,
            parent: &mut Mob,
            partner: &crate::world::entity::living_entity::LivingEntity,
        ) {
            let vacant = self.take_vacant_bed(level, parent);
            if vacant.is_none() {
                level.broadcast_entity_event(partner.entity.id, crate::world::entity::entity_event::VILLAGER_ANGRY);
                level.broadcast_entity_event(parent.living.entity.id, crate::world::entity::entity_event::VILLAGER_ANGRY);
                return;
            }
            let bed = vacant.unwrap();
            // breed → spawn offspring (entity-hook).
            if self.breed(level, parent, partner).is_some() {
                // giveBedToChild: HOME memory на child —
                // child спавнится через level hook; память
                // HOME ставится при villager-init (runtime).
                let _ = bed;
            } else {
                if let Some(pm) = level.poi_manager() {
                    let _ = pm.release(bed);
                }
            }
        }

        fn take_vacant_bed(&self, level: &Arc<dyn LevelReader>, mob: &Mob) -> Option<BlockPos> {
            let pm = level.poi_manager()?;
            let home = poi_types::home();
            pm.take(
                &|h| crate::world::entity::ai::village::poi::poi_record::holder_is(h, &home),
                &|h, bp| {
                    let range = crate::world::entity::ai::village::poi::poi_manager::valid_range_of(h);
                    self.can_reach(mob, bp, range)
                },
                mob.block_position(),
                48,
            )
        }

        fn can_reach(&self, mob: &Mob, pos: BlockPos, _range: i32) -> bool {
            mob.navigation
                .as_ref()
                .and_then(|n| n.borrow_mut().create_path(mob, pos.get_x() as f64, pos.get_y() as f64, pos.get_z() as f64, 1))
                .map(|p| p.can_reach())
                .unwrap_or(false)
        }

        /// `breed` — spawn детёныша через `level.add_fresh_entity`
        /// (фабрика villager-child — runtime hook). Возвращает
        /// Some(pos) при успехе.
        fn breed(
            &self,
            level: &Arc<dyn LevelReader>,
            parent: &mut Mob,
            partner: &crate::world::entity::living_entity::LivingEntity,
        ) -> Option<Vec3> {
            // setAge(6000) обоим родителям, child age=-24000.
            parent.living.entity.set_age(6000);
            let _ = partner;
            let pos = parent.living.entity.position();
            let bp = BlockPos::containing(pos.x, pos.y, pos.z);
            let child = level.spawn_entity_of_type(
                &Identifier::minecraft("villager"),
                bp,
            )?;
            let mut child_entity = child.entity.clone();
            child_entity.set_age(-24000);
            child_entity.set_pos(pos.x, pos.y, pos.z);
            level.add_fresh_entity(child_entity.clone());
            level.broadcast_entity_event(child_entity.id, crate::world::entity::entity_event::LOVE_HEARTS);
            Some(pos)
        }
    }
    VillagerMakeLove {
        base: BehaviorBase::with_range(
            BehaviorBase::conditions(&[
                (keys::breed_target(), MemoryStatus::ValuePresent),
                (keys::nearest_visible_living_entities(), MemoryStatus::ValuePresent),
            ]),
            350,
            350,
        ),
        birth_timestamp: 0,
    }
}

// ---------- Croak (Frog) ----------

/// `Croak` — лягушка квакает 60 тиков (Pose.CROAKING).
pub fn croak() -> impl BehaviorControl {
    struct Croak {
        base: BehaviorBase,
        croak_counter: i32,
    }
    impl BehaviorControl for Croak {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "Croak".to_string() }

        fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            mob.living.entity.get_pose() == Pose::Standing
        }

        fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) -> bool {
            self.croak_counter < 60
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            if !(mob.living.entity.is_in_water() || mob.living.entity.is_in_lava()) {
                mob.living.set_pose(Pose::Croaking);
                self.croak_counter = 0;
            }
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            mob.living.set_pose(Pose::Standing);
        }

        fn tick(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) {
            self.croak_counter += 1;
        }
    }
    Croak {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[(keys::walk_target(), MemoryStatus::ValueAbsent)]),
            100,
        ),
        croak_counter: 0,
    }
}

// ---------- SpearApproach / SpearAttack / SpearRetreat ----------

/// `SpearAttack.SpearStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpearStatus {
    /// `APPROACH`.
    Approach,
    /// `CHARGING`.
    Charging,
    /// `RETREAT`.
    Retreat,
}

fn spear_able_to_attack(mob: &Mob) -> bool {
    mob.living
        .brain_get::<crate::world::entity::living_entity::LivingEntity>(&keys::attack_target())
        .is_some()
        && mob
            .living
            .get_main_hand_item()
            .has_component(&Identifier::minecraft("kinetic_weapon"))
}

fn spear_get_target(mob: &Mob) -> Option<crate::world::entity::living_entity::LivingEntity> {
    mob.living.brain_get(&keys::attack_target())
}

fn spear_charge_speed(mob: &Mob) -> f32 {
    let root = mob.living.entity.get_root_vehicle();
    root.entity_charge_speed_modifier()
}

/// `SpearApproach(speedModifierWhenRepositioning,
/// approachDistance)`.
pub fn spear_approach(speed_when_repositioning: f64, approach_distance: f32) -> impl BehaviorControl {
    struct SpearApproach {
        base: BehaviorBase,
        speed_modifier: f64,
        approach_distance_sq: f32,
    }
    impl BehaviorControl for SpearApproach {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "SpearApproach".to_string() }

        fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            spear_able_to_attack(mob) && !mob.living.is_using_item()
        }

        fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
            spear_able_to_attack(mob) && self.far_enough(mob)
        }

        fn timed_out(&self, _game_time: i64) -> bool { false }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            mob.set_aggressive(true);
            mob.living.brain_set_value(
                &keys::spear_status(),
                SpearStatus::Approach,
            );
        }

        fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let Some(target) = spear_get_target(mob) else { return };
            let f = spear_charge_speed(mob);
            mob.living.brain_set_value(
                &keys::look_target(),
                Rc::new(memory::EntityTracker::new(target.entity.clone(), true))
                    as Rc<dyn memory::PositionTracker>,
            );
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().move_to_entity(
                    mob,
                    &target,
                    f as f64 * self.speed_modifier,
                );
            }
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().stop(mob);
            }
            mob.living.brain_set_value(&keys::spear_status(), SpearStatus::Charging);
        }
    }
    impl SpearApproach {
        fn far_enough(&self, mob: &Mob) -> bool {
            let Some(target) = spear_get_target(mob) else { return false };
            let d = mob
                .living
                .entity
                .distance_to_sqr_pos(target.entity.get_x(), target.entity.get_y(), target.entity.get_z());
            d > self.approach_distance_sq as f64
        }
    }
    SpearApproach {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[(keys::spear_status(), MemoryStatus::ValueAbsent)]),
            60,
        ),
        speed_modifier: speed_when_repositioning,
        approach_distance_sq: approach_distance * approach_distance,
    }
}

/// `SpearAttack.chargeSpeedModifier`-duration — engage time.
pub fn spear_attack(
    speed_charging: f64,
    speed_repositioning: f64,
    approach_distance: f32,
    target_in_range_radius: f32,
) -> impl BehaviorControl {
    struct SpearAttack {
        base: BehaviorBase,
        speed_charging: f64,
        speed_repositioning: f64,
        approach_distance_sq: f32,
        target_in_range_radius_sq: f32,
    }
    impl BehaviorControl for SpearAttack {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "SpearAttack".to_string() }

        fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            let status: Option<SpearStatus> = mob.living.brain_get(&keys::spear_status());
            (status.unwrap_or(SpearStatus::Approach) == SpearStatus::Charging)
                && spear_able_to_attack(mob)
                && !mob.living.is_using_item()
        }

        fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
            let t: Option<i32> = mob.living.brain_get(&keys::spear_engage_time());
            t.unwrap_or(0) > 0 && spear_able_to_attack(mob)
        }

        fn timed_out(&self, _game_time: i64) -> bool { false }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            mob.set_aggressive(true);
            let duration = get_kinetic_weapon_use_duration(mob);
            mob.living.brain_set_value(&keys::spear_engage_time(), duration);
            mob.living.brain_erase(&keys::spear_charge_position());
            mob.living.start_using_item(HumanoidArm::Right);
        }

        fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let Some(target) = spear_get_target(mob) else { return };
            let d = mob
                .living
                .entity
                .distance_to_sqr_pos(target.entity.get_x(), target.entity.get_y(), target.entity.get_z());
            let f = spear_charge_speed(mob);
            let i = if mob.living.entity.is_passenger() { 2 } else { 0 };
            mob.living.brain_set_value(
                &keys::look_target(),
                Rc::new(memory::EntityTracker::new(target.entity.clone(), true))
                    as Rc<dyn memory::PositionTracker>,
            );
            let cur: Option<i32> = mob.living.brain_get(&keys::spear_engage_time());
            mob.living
                .brain_set_value(&keys::spear_engage_time(), cur.unwrap_or(0) - 1);
            let charge_pos: Option<Vec3> = mob.living.brain_get(&keys::spear_charge_position());
            if let Some(vec3) = charge_pos {
                if let Some(nav) = mob.get_navigation() {
                    nav.borrow_mut().move_to_pos(
                        mob,
                        vec3.x,
                        vec3.y,
                        vec3.z,
                        f as f64 * self.speed_repositioning,
                    );
                }
                let done = mob
                    .get_navigation()
                    .as_ref()
                    .map(|n| n.borrow().is_done())
                    .unwrap_or(true);
                if done {
                    mob.living.brain_erase(&keys::spear_charge_position());
                }
            } else {
                if let Some(nav) = mob.get_navigation() {
                    nav.borrow_mut().move_to_entity(
                        mob,
                        &target,
                        f as f64 * self.speed_charging,
                    );
                }
                let nav_done = mob
                    .get_navigation()
                    .as_ref()
                    .map(|n| n.borrow().is_done())
                    .unwrap_or(true);
                if d < self.target_in_range_radius_sq as f64 || nav_done {
                    let square_root = d.sqrt();
                    let pos_away = crate::world::entity::ai::util::land_random_pos::get_pos_away_min(
                        mob,
                        6.0 + i as f64 - square_root,
                        7.0 + i as f64 - square_root,
                        7,
                        target.position(),
                    );
                    mob.living.brain_set_value(&keys::spear_charge_position(), pos_away);
                }
            }
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().stop(mob);
            }
            mob.living.stop_using_item();
            mob.living.brain_erase(&keys::spear_charge_position());
            mob.living.brain_erase(&keys::spear_engage_time());
            mob.living.brain_set_value(&keys::spear_status(), SpearStatus::Retreat);
        }
    }
    SpearAttack {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[(keys::spear_status(), MemoryStatus::ValuePresent)]),
            60,
        ),
        speed_charging,
        speed_repositioning,
        approach_distance_sq: approach_distance * approach_distance,
        target_in_range_radius_sq: target_in_range_radius * target_in_range_radius,
    }
}

/// `SpearAttack.getKineticWeaponUseDuration` —
/// `delayTicks + damageConditions.maxDurationTicks`
/// (без reducedTickDelay — Brain-тик каждый тик).
fn get_kinetic_weapon_use_duration(mob: &Mob) -> i32 {
    let stack = mob.living.get_main_hand_item();
    stack
        .get_component::<crate::world::item::component::kinetic_weapon::KineticWeapon>(
            &Identifier::minecraft("kinetic_weapon"),
        )
        .map(|kw| {
            kw.delay_ticks
                + kw.damage_conditions
                    .map(|c| c.max_duration_ticks)
                    .unwrap_or(0)
        })
        .unwrap_or(0)
}

/// `SpearRetreat(speedModifierWhenRepositioning)` —
/// duration 100.
pub fn spear_retreat(speed_when_repositioning: f64) -> impl BehaviorControl {
    struct SpearRetreat {
        base: BehaviorBase,
        speed_modifier: f64,
    }
    impl BehaviorControl for SpearRetreat {
        fn status(&self) -> BehaviorStatus { self.base.status }
        fn set_status(&mut self, s: BehaviorStatus) { self.base.status = s; }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
        fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
        fn set_end_timestamp(&mut self, t: i64) { self.base.end_timestamp = t; }
        fn debug_string(&self) -> String { "SpearRetreat".to_string() }

        fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            if !(spear_able_to_attack(mob) && !mob.living.is_using_item()) {
                return false;
            }
            let status: Option<SpearStatus> = mob.living.brain_get(&keys::spear_status());
            if status.unwrap_or(SpearStatus::Approach) != SpearStatus::Retreat {
                return false;
            }
            let Some(target) = spear_get_target(mob) else { return false };
            let d = mob
                .living
                .entity
                .distance_to_sqr_pos(target.entity.get_x(), target.entity.get_y(), target.entity.get_z());
            let i = if mob.living.entity.is_passenger() { 2 } else { 0 };
            let square_root = d.sqrt();
            let pos_away = crate::world::entity::ai::util::land_random_pos::get_pos_away_min(
                mob,
                (9.0 + i as f64 - square_root).max(0.0),
                (11.0 + i as f64 - square_root).max(1.0),
                7,
                target.position(),
            );
            match pos_away {
                Some(p) => {
                    mob.living.brain_set_value(&keys::spear_fleeing_position(), p);
                    true
                }
                None => false,
            }
        }

        fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
            let t: Option<i32> = mob.living.brain_get(&keys::spear_fleeing_time());
            t.unwrap_or(100) < 100
                && mob.living.brain_has(&keys::spear_fleeing_position())
                && !mob
                    .get_navigation()
                    .as_ref()
                    .map(|n| n.borrow().is_done())
                    .unwrap_or(true)
                && spear_able_to_attack(mob)
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            mob.set_aggressive(true);
            mob.living.brain_set_value(&keys::spear_fleeing_time(), 0);
        }

        fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let Some(target) = spear_get_target(mob) else { return };
            let f = spear_charge_speed(mob);
            mob.living.brain_set_value(
                &keys::look_target(),
                Rc::new(memory::EntityTracker::new(target.entity.clone(), true))
                    as Rc<dyn memory::PositionTracker>,
            );
            let cur: Option<i32> = mob.living.brain_get(&keys::spear_fleeing_time());
            mob.living
                .brain_set_value(&keys::spear_fleeing_time(), cur.unwrap_or(0) + 1);
            if let Some(vec3) = mob.living.brain_get::<Vec3>(&keys::spear_fleeing_position()) {
                if let Some(nav) = mob.get_navigation() {
                    nav.borrow_mut().move_to_pos(
                        mob,
                        vec3.x,
                        vec3.y,
                        vec3.z,
                        f as f64 * self.speed_modifier,
                    );
                }
            }
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().stop(mob);
            }
            mob.set_aggressive(false);
            mob.living.stop_using_item();
            mob.living.brain_erase(&keys::spear_fleeing_time());
            mob.living.brain_erase(&keys::spear_fleeing_position());
            mob.living.brain_erase(&keys::spear_status());
        }
    }
    SpearRetreat {
        base: BehaviorBase::with_duration(
            BehaviorBase::conditions(&[(keys::spear_status(), MemoryStatus::ValuePresent)]),
            100,
        ),
        speed_modifier: speed_when_repositioning,
    }
}
