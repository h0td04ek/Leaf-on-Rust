//! Перенос walk/stroll-поведений: `GoToTargetLocation`,
//! `GoToClosestVillage`, `GoToWantedItem`, `RandomStroll`
//! (+fly/swim), `VillageBoundRandomStroll`, `StrollAroundPoi`,
//! `StrollToPoiList`, `MoveToSkySeeingSpot`, `GoToPotentialJobSite`,
//! `SetClosestHomeAsWalkTarget`, `AcquirePoi`, `PoiCompetitorScan`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::global_pos::GlobalPos;
use crate::core::section_pos::SectionPos;
use crate::world::entity::ai::behavior::behavior_utils;
use crate::world::entity::ai::behavior::simple::{conds, PointBehavior};
use crate::world::entity::ai::memory::memory::{
    keys, MemoryModuleType, MemoryStatus, WalkTarget,
};
use crate::world::entity::ai::util::air_and_water_random_pos;
use crate::world::entity::ai::util::default_random_pos;
use crate::world::entity::ai::util::goal_utils;
use crate::world::entity::ai::util::land_random_pos;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;

// ===================== GoToTargetLocation =====================

/// `GoToTargetLocation.create(locationMemory, closeEnoughDist, speedModifier)`.
/// locationMemory present + ATTACK/WALK absent + LOOK registered;
/// !closerThan(pos, closeEnough) → walk+look на pos+randomOffset.
pub fn go_to_target_location(
    location_memory: MemoryModuleType,
    close_enough_dist: i32,
    speed_modifier: f32,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (location_memory.clone(), MemoryStatus::ValuePresent),
            (keys::attack_target(), MemoryStatus::ValueAbsent),
            (keys::walk_target(), MemoryStatus::ValueAbsent),
            (keys::look_target(), MemoryStatus::Registered),
        ]),
        move |_level, mob, _game_time| {
            let block_pos: Option<BlockPos> = mob.living.brain_get(&location_memory);
            let Some(block_pos) = block_pos else { return true };
            let flag = block_pos.dist_sqr(mob.block_position())
                < (close_enough_dist * close_enough_dist) as f64;
            if !flag {
                // getNearbyPos — pos + offset(-1..1, 0, -1..1).
                let mut r = mob.random.borrow_mut();
                let dx = r.next_int_bound(3) - 1;
                let dz = r.next_int_bound(3) - 1;
                drop(r);
                let nearby = BlockPos::new(block_pos.x + dx, block_pos.y, block_pos.z + dz);
                behavior_utils::set_walk_and_look_target_memories_pos(
                    mob,
                    nearby,
                    speed_modifier,
                    close_enough_dist,
                );
            }
            true
        },
    )
}

// ===================== GoToClosestVillage =====================

/// `GoToClosestVillage.create(speedModifier, closeEnoughDist)` —
/// Villager-only; isVillage → false; иначе 5 попыток LandRandomPos
/// к секции с меньшим sectionsToVillage → WALK_TARGET.
pub fn go_to_closest_village(speed_modifier: f32, close_enough_dist: i32) -> PointBehavior {
    PointBehavior::new(
        conds(&[(keys::walk_target(), MemoryStatus::ValueAbsent)]),
        move |level, mob, _game_time| {
            if level.is_village(mob.block_position()) {
                return false;
            }
            let i = level.sections_to_village(SectionPos::of(&mob.block_position()));
            let mut best: Option<Vec3> = None;
            for _ in 0..5 {
                // getPos к "не-деревне": -sectionsToVillage (отрицательно).
                let pos = land_random_pos::get_pos_scored(mob, 15, 7, |_m, bp| {
                    -(level.sections_to_village(SectionPos::of(&bp))) as f64
                });
                if let Some(pos) = pos {
                    let bp = BlockPos::containing_vec3(&pos);
                    let i2 = level.sections_to_village(SectionPos::of(&bp));
                    if i2 < i {
                        best = Some(pos);
                        break;
                    }
                    if i2 == i {
                        best = Some(pos);
                    }
                }
            }
            if let Some(vec3) = best {
                let wt = WalkTarget::new_vec3(vec3, speed_modifier, close_enough_dist);
                mob.living.brain_set_value(&keys::walk_target(), wt);
            }
            true
        },
    )
}

// ===================== GoToWantedItem =====================

/// `GoToWantedItem.create(canWalkToItem, speedModifier, hasTarget,
/// maxDistToWalk)`.
pub fn go_to_wanted_item(
    can_walk_to_item: impl Fn(&Mob) -> bool + 'static,
    speed_modifier: f32,
    has_target: bool,
    max_dist_to_walk: i32,
) -> PointBehavior {
    let pred = Rc::new(can_walk_to_item);
    let walk_status = if has_target {
        MemoryStatus::Registered
    } else {
        MemoryStatus::ValueAbsent
    };
    PointBehavior::new(
        conds(&[
            (keys::look_target(), MemoryStatus::Registered),
            (keys::walk_target(), walk_status),
            (keys::nearest_visible_wanted_item(), MemoryStatus::ValuePresent),
            (keys::item_pickup_cooldown_ticks(), MemoryStatus::Registered),
        ]),
        move |level, mob, _game_time| {
            let item: Option<crate::world::entity::entity::Entity> =
                mob.living.brain_get(&keys::nearest_visible_wanted_item());
            let Some(item) = item else { return false };
            let cooldown_empty = !mob
                .living
                .brain_has(&keys::item_pickup_cooldown_ticks());
            if cooldown_empty
                && pred(mob)
                && item.distance_to_sqr(&mob.living.entity)
                    <= (max_dist_to_walk * max_dist_to_walk) as f64
                && level.world_border_is_within_bounds(item.block_position())
                && mob.living.can_pick_up_loot()
            {
                let wt = WalkTarget::new_entity(item.clone(), speed_modifier, 0);
                let look = crate::world::entity::ai::memory::memory::EntityTracker::new(
                    item.clone(),
                    true,
                );
                mob.living.brain_set_value(
                    &keys::look_target(),
                    Rc::new(look)
                        as Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>,
                );
                mob.living.brain_set_value(&keys::walk_target(), wt);
                true
            } else {
                false
            }
        },
    )
}

// ===================== RandomStroll =====================

/// `SWIM_XY_DISTANCE_TIERS`.
const SWIM_XY_DISTANCE_TIERS: &[(i32, i32)] = &[
    (1, 1), (3, 3), (5, 5), (6, 5), (7, 7), (10, 7),
];

/// `RandomStroll.stroll(speedModifier)`.
pub fn random_stroll(speed_modifier: f32) -> PointBehavior {
    random_stroll_with_may_stroll_from_water(speed_modifier, true)
}

/// `stroll(speedModifier, mayStrollFromWater)`.
pub fn random_stroll_with_may_stroll_from_water(
    speed_modifier: f32,
    may_stroll_from_water: bool,
) -> PointBehavior {
    let sm = Rc::new(move |mob: &Mob| {
        if !(may_stroll_from_water || !mob.living.entity.is_in_water()) {
            return None;
        }
        land_random_pos::get_pos(mob, 10, 7)
    });
    stroll_fly_or_swim(speed_modifier, sm, Rc::new(move |mob| {
        may_stroll_from_water || !mob.living.entity.is_in_water()
    }))
}

/// `stroll(speedModifier, maxHorizontalDistance, maxVerticalDistance)`.
pub fn random_stroll_dist(
    speed_modifier: f32,
    max_horizontal_distance: i32,
    max_vertical_distance: i32,
) -> PointBehavior {
    let sm = Rc::new(move |mob: &Mob| {
        land_random_pos::get_pos(mob, max_horizontal_distance, max_vertical_distance)
    });
    stroll_fly_or_swim(speed_modifier, sm, Rc::new(|_| true))
}

/// `fly(speedModifier)` — AirAndWaterRandomPos по viewVector.
pub fn random_stroll_fly(speed_modifier: f32) -> PointBehavior {
    let sm = Rc::new(|mob: &Mob| {
        let view = mob.living.get_view_vector(0.0);
        air_and_water_random_pos::get_pos(
            mob,
            10,
            7,
            -2,
            view.x,
            view.z,
            std::f64::consts::FRAC_PI_2,
        )
    });
    stroll_fly_or_swim(speed_modifier, sm, Rc::new(|_| true))
}

/// `swim(speedModifier)` — getTargetSwimPos + isInWater.
pub fn random_stroll_swim(speed_modifier: f32) -> PointBehavior {
    let sm = Rc::new(get_target_swim_pos as fn(&Mob) -> Option<Vec3>);
    stroll_fly_or_swim(
        speed_modifier,
        sm,
        Rc::new(|mob| mob.living.entity.is_in_water()),
    )
}

/// `strollFlyOrSwim(speedModifier, target, canStroll)`.
fn stroll_fly_or_swim(
    speed_modifier: f32,
    target: Rc<dyn Fn(&Mob) -> Option<Vec3>>,
    can_stroll: Rc<dyn Fn(&Mob) -> bool>,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[(keys::walk_target(), MemoryStatus::ValueAbsent)]),
        move |_level, mob, _game_time| {
            if !can_stroll(mob) {
                return false;
            }
            let optional = target(mob);
            match optional {
                Some(target_pos) => {
                    let wt = WalkTarget::new_vec3(target_pos, speed_modifier, 0);
                    mob.living.brain_set_value(&keys::walk_target(), wt);
                }
                None => {
                    mob.living.brain_erase(&keys::walk_target());
                }
            }
            true
        },
    )
}

/// `getTargetSwimPos(mob)` — нарастающие (xz,y) ярусы; первая
/// позиция с fluidState !empty и !restricted → возвращает
/// предыдущую (или null если ни одна не вода).
fn get_target_swim_pos(mob: &Mob) -> Option<Vec3> {
    let mut prev: Option<Vec3> = None;
    let mut last: Option<Vec3> = None;
    let level = mob.level();
    let Some(level) = level else { return None };
    for &(xz, y) in SWIM_XY_DISTANCE_TIERS {
        let candidate = if prev.is_none() {
            crate::world::entity::ai::util::behavior_utils::get_random_swimmable_pos(mob, xz, y)
        } else {
            let p = prev.unwrap();
            let dir = p.subtract_vec(&mob.living.entity.position()).normalize();
            Some(
                mob.living
                    .entity
                    .position()
                    .add_vec(&dir.multiply(xz as f64, y as f64, xz as f64)),
            )
        };
        let is_restricted = goal_utils::mob_restricted(mob, xz as f64);
        let fluid_empty = candidate
            .map(|c| {
                level
                    .get_fluid_state(BlockPos::containing_vec3(&c))
                    .map(|f| f.is_empty())
                    .unwrap_or(true)
            })
            .unwrap_or(true);
        let restricted = candidate
            .map(|c| goal_utils::is_restricted_vec(mob, is_restricted, c))
            .unwrap_or(true);
        if candidate.is_none() || fluid_empty || restricted {
            return prev;
        }
        prev = candidate;
        last = candidate;
    }
    last.or(prev)
}

// ===================== VillageBoundRandomStroll =====================

/// `VillageBoundRandomStroll.create(speedModifier)`.
pub fn village_bound_random_stroll(speed_modifier: f32) -> PointBehavior {
    village_bound_random_stroll_dist(speed_modifier, 10, 7)
}

/// `create(speedModifier, maxHorizontalDist, maxVerticalDist)`.
pub fn village_bound_random_stroll_dist(
    speed_modifier: f32,
    max_horizontal_dist: i32,
    max_vertical_dist: i32,
) -> PointBehavior {
    PointBehavior::new(
        conds(&[(keys::walk_target(), MemoryStatus::ValueAbsent)]),
        move |level, mob, _game_time| {
            let block_pos = mob.block_position();
            let pos: Option<Vec3> = if level.is_village(block_pos) {
                land_random_pos::get_pos(mob, max_horizontal_dist, max_vertical_dist)
            } else {
                let section_pos = SectionPos::of(&block_pos);
                let closest =
                    behavior_utils::find_section_closest_to_village(level, section_pos, 2);
                if closest != section_pos {
                    default_random_pos::get_pos_towards(
                        mob,
                        max_horizontal_dist,
                        max_vertical_dist,
                        crate::world::phys::vec3::at_bottom_center_of(&crate::core::vec3i::Vec3i::new(
                            closest.center().x, closest.center().y, closest.center().z,
                        )),
                        std::f64::consts::FRAC_PI_2,
                    )
                } else {
                    land_random_pos::get_pos(mob, max_horizontal_dist, max_vertical_dist)
                }
            };
            match pos {
                Some(target) => {
                    let wt = WalkTarget::new_vec3(target, speed_modifier, 0);
                    mob.living.brain_set_value(&keys::walk_target(), wt);
                }
                None => {
                    mob.living.brain_erase(&keys::walk_target());
                }
            }
            true
        },
    )
}

// ===================== StrollAroundPoi =====================

/// `StrollAroundPoi.create(poiPosMemory, speedModifier, maxDistFromPoi)`.
pub fn stroll_around_poi(
    poi_pos_memory: MemoryModuleType,
    speed_modifier: f32,
    max_dist_from_poi: i32,
) -> PointBehavior {
    // mutableLong — throttle 180 тиков.
    let last_update = Rc::new(RefCell::new(0i64));
    let lu = last_update.clone();
    PointBehavior::new(
        conds(&[
            (keys::walk_target(), MemoryStatus::Registered),
            (poi_pos_memory.clone(), MemoryStatus::ValuePresent),
        ]),
        move |level, mob, game_time| {
            let gpos: Option<GlobalPos> = mob.living.brain_get(&poi_pos_memory);
            let Some(gpos) = gpos else { return false };
            let dim_ok = level
                .dimension_id()
                .map(|d| d == &gpos.dimension.identifier)
                .unwrap_or(false);
            let near = gpos.pos.closer_to_center_than(
                mob.living.entity.position(),
                max_dist_from_poi as f64,
            );
            if !dim_ok || !near {
                return false;
            }
            let mut lu = lu.borrow_mut();
            if game_time <= *lu {
                return true;
            }
            *lu = game_time + 180;
            let optional = land_random_pos::get_pos(mob, 8, 6);
            match optional {
                Some(target_pos) => {
                    let wt = WalkTarget::new_vec3(target_pos, speed_modifier, 1);
                    mob.living.brain_set_value(&keys::walk_target(), wt);
                }
                None => {
                    mob.living.brain_erase(&keys::walk_target());
                }
            }
            true
        },
    )
}

// ===================== MoveToSkySeeingSpot =====================

/// `MoveToSkySeeingSpot.create(speedModifier)` — моб под небом
/// не может видеть небо → ищет точку с canSeeSky.
pub fn move_to_sky_seeing_spot(speed_modifier: f32) -> PointBehavior {
    PointBehavior::new(
        conds(&[(keys::walk_target(), MemoryStatus::ValueAbsent)]),
        move |level, mob, _game_time| {
            if level.can_see_sky(mob.block_position()) {
                return false;
            }
            // 5 попыток LandRandomPos покрупнее.
            for _ in 0..5 {
                if let Some(pos) = land_random_pos::get_pos(mob, 10, 7) {
                    let bp = BlockPos::containing_vec3(&pos);
                    if level.can_see_sky(bp) {
                        let wt = WalkTarget::new_vec3(pos, speed_modifier, 1);
                        mob.living.brain_set_value(&keys::walk_target(), wt);
                        return true;
                    }
                }
            }
            true
        },
    )
}
