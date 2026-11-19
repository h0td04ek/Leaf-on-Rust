//! Перенос `net.minecraft.world.entity.ai.util.LandRandomPos`.

use crate::core::block_pos::BlockPos;
use crate::world::entity::ai::util::{goal_utils, random_pos};
use crate::world::entity::mob::Mob;
use crate::world::phys::vec3::Vec3;

/// `getPos(mob, radius, verticalRange)`.
pub fn get_pos(mob: &Mob, radius: i32, vertical_range: i32) -> Option<Vec3> {
    let is_restricted = goal_utils::mob_restricted(mob, radius as f64);
    let random = mob.random.clone();
    random_pos::generate_random_pos(mob, || {
        let offset = {
            let mut r = random.borrow_mut();
            random_pos::generate_random_direction(&mut **r, radius, vertical_range)
        };
        let bp = generate_random_pos_toward_direction(mob, radius as f64, is_restricted, offset)?;
        move_pos_up_out_of_solid(mob, bp)
    })
}

/// `getPosTowards(mob, radius, yRange, vectorPosition)`.
pub fn get_pos_towards(mob: &Mob, radius: i32, y_range: i32, vector_position: Vec3) -> Option<Vec3> {
    get_pos_in_direction(mob, 0.0, radius as f64, y_range, vector_position)
}

/// `getPosAway(mob, radius, yRange, vectorPosition)`.
pub fn get_pos_away(mob: &Mob, radius: i32, y_range: i32, vector_position: Vec3) -> Option<Vec3> {
    get_pos_away_min(mob, 0.0, radius as f64, y_range, vector_position)
}

/// `getPosAway(mob, minDistance, maxDistance, yRange, vectorPosition)`.
pub fn get_pos_away_min(
    mob: &Mob,
    min_distance: f64,
    max_distance: f64,
    y_range: i32,
    vector_position: Vec3,
) -> Option<Vec3> {
    let mut diff = mob.living.entity.position().subtract_vec(&vector_position);
    if diff.length_sqr() == 0.0 {
        let mut r = mob.random.borrow_mut();
        diff = Vec3::new(r.next_double() - 0.5, 0.0, r.next_double() - 0.5);
    }
    let is_restricted = goal_utils::mob_restricted(mob, max_distance);
    get_pos_in_direction_impl(mob, min_distance, max_distance, y_range, diff, is_restricted)
}

fn get_pos_in_direction(
    mob: &Mob,
    min_distance: f64,
    max_distance: f64,
    y_range: i32,
    vector_position: Vec3,
) -> Option<Vec3> {
    let diff = vector_position.subtract_vec(&mob.living.entity.position());
    let is_restricted = goal_utils::mob_restricted(mob, max_distance);
    get_pos_in_direction_impl(mob, min_distance, max_distance, y_range, diff, is_restricted)
}

fn get_pos_in_direction_impl(
    mob: &Mob,
    min_distance: f64,
    max_distance: f64,
    y_range: i32,
    vector_position: Vec3,
    is_restricted: bool,
) -> Option<Vec3> {
    let random = mob.random.clone();
    random_pos::generate_random_pos(mob, || {
        let offset = {
            let mut r = random.borrow_mut();
            random_pos::generate_random_direction_within_radians(
                &mut **r,
                min_distance,
                max_distance,
                y_range,
                0,
                vector_position.x,
                vector_position.z,
                std::f64::consts::FRAC_PI_2,
            )?
        };
        let bp = generate_random_pos_toward_direction(mob, max_distance, is_restricted, offset)?;
        move_pos_up_out_of_solid(mob, bp)
    })
}

/// `movePosUpOutOfSolid(mob, pos)`.
pub fn move_pos_up_out_of_solid(mob: &Mob, pos: BlockPos) -> Option<BlockPos> {
    let Some(level) = mob.level() else { return None };
    let max_y = level.min_build_height() + level.height();
    let bp = random_pos::move_up_out_of_solid(pos, max_y, |p| goal_utils::is_solid(mob, p));
    if !goal_utils::is_water(mob, bp) && !goal_utils::has_malus(mob, bp) {
        Some(bp)
    } else {
        None
    }
}

/// `generateRandomPosTowardDirection(mob, radius, isRestricted, pos)`.
pub fn generate_random_pos_toward_direction(
    mob: &Mob,
    radius: f64,
    is_restricted: bool,
    pos: BlockPos,
) -> Option<BlockPos> {
    let block_pos = {
        let mut r = mob.random.borrow_mut();
        random_pos::generate_random_pos_toward_direction(mob, radius, &mut **r, pos)
    };
    if goal_utils::is_outside_limits(mob, block_pos)
        || goal_utils::is_restricted_block(mob, is_restricted, block_pos)
        || goal_utils::is_not_stable(mob, block_pos)
    {
        None
    } else {
        Some(block_pos)
    }
}

/// `getPos(mob, radius, yRange, scorer)` — scorer вместо
/// walkTargetValue (MoveThroughVillageGoal, StrollThroughVillageGoal).
pub fn get_pos_scored(
    mob: &Mob,
    radius: i32,
    vertical_range: i32,
    mut scorer: impl FnMut(&Mob, BlockPos) -> f64,
) -> Option<Vec3> {
    let is_restricted = goal_utils::mob_restricted(mob, radius as f64);
    let random = mob.random.clone();
    random_pos::generate_random_pos_scored(
        mob,
        || {
            let offset = {
                let mut r = random.borrow_mut();
                random_pos::generate_random_direction(&mut **r, radius, vertical_range)
            };
            let bp = generate_random_pos_toward_direction(mob, radius as f64, is_restricted, offset)?;
            move_pos_up_out_of_solid(mob, bp)
        },
        |m, bp| scorer(m, bp),
    )
}
