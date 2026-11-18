//! Перенос `net.minecraft.world.entity.ai.util.DefaultRandomPos`.

use crate::core::block_pos::BlockPos;
use crate::util::mth;
use crate::world::entity::ai::util::goal_utils;
use crate::world::entity::mob::Mob;
use crate::world::level::pathfinder::path_computation_type::PathComputationType;
use crate::world::phys::vec3::Vec3;
use std::cell::RefCell;
use std::rc::Rc;

use super::random_pos;

/// `getPos(mob, radius, verticalDistance)`.
pub fn get_pos(mob: &Mob, radius: i32, vertical_distance: i32) -> Option<Vec3> {
    let is_restricted = goal_utils::mob_restricted(mob, radius as f64);
    let random = mob.random.clone();
    random_pos::generate_random_pos(mob, || {
        let offset = {
            let mut r = random.borrow_mut();
            random_pos::generate_random_direction(&mut **r, radius, vertical_distance)
        };
        generate_random_pos_toward_direction_impl(mob, radius as f64, is_restricted, offset)
    })
}

/// `getPosTowards(mob, radius, yRange, targetPos, amplifier)`.
pub fn get_pos_towards(
    mob: &Mob,
    radius: i32,
    y_range: i32,
    target_pos: Vec3,
    amplifier: f64,
) -> Option<Vec3> {
    let diff = target_pos.subtract_vec(&mob.living.entity.position());
    let is_restricted = goal_utils::mob_restricted(mob, radius as f64);
    let random = mob.random.clone();
    random_pos::generate_random_pos(mob, || {
        let offset = {
            let mut r = random.borrow_mut();
            random_pos::generate_random_direction_within_radians(
                &mut **r,
                0.0,
                radius as f64,
                y_range,
                0,
                diff.x,
                diff.z,
                amplifier,
            )?
        };
        generate_random_pos_toward_direction_impl(mob, radius as f64, is_restricted, offset)
    })
}

/// `getPosAway(mob, radius, yRange, avoidPos)`.
pub fn get_pos_away(
    mob: &Mob,
    radius: i32,
    y_range: i32,
    avoid_pos: Vec3,
) -> Option<Vec3> {
    let diff = mob.living.entity.position().subtract_vec(&avoid_pos);
    let is_restricted = goal_utils::mob_restricted(mob, radius as f64);
    let random = mob.random.clone();
    random_pos::generate_random_pos(mob, || {
        let offset = {
            let mut r = random.borrow_mut();
            random_pos::generate_random_direction_within_radians(
                &mut **r,
                0.0,
                radius as f64,
                y_range,
                0,
                diff.x,
                diff.z,
                std::f64::consts::FRAC_PI_2,
            )?
        };
        generate_random_pos_toward_direction_impl(mob, radius as f64, is_restricted, offset)
    })
}

/// `generateRandomPosTowardDirection(mob, radius, isRestricted, pos)` —
/// приватный в Java; здесь pub(crate) для DefaultRandomPos.
pub(crate) fn generate_random_pos_toward_direction_impl(
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
        || goal_utils::has_malus(mob, block_pos)
    {
        None
    } else {
        Some(block_pos)
    }
}
