//! Перенос `net.minecraft.world.entity.ai.util.AirAndWaterRandomPos`.

use crate::core::block_pos::BlockPos;
use crate::world::entity::ai::util::{goal_utils, random_pos};
use crate::world::entity::mob::Mob;
use crate::world::phys::vec3::Vec3;

/// `getPos(mob, maxDistance, yRange, y, x, z, amplifier)`.
#[allow(clippy::too_many_arguments)]
pub fn get_pos(
    mob: &Mob,
    max_distance: i32,
    y_range: i32,
    y: i32,
    x: f64,
    z: f64,
    amplifier: f64,
) -> Option<Vec3> {
    let is_restricted = goal_utils::mob_restricted(mob, max_distance as f64);
    let random = mob.random.clone();
    random_pos::generate_random_pos(mob, || {
        generate_random_pos(mob, max_distance, y_range, y, x, z, amplifier, is_restricted, &random)
    })
}

/// `generateRandomPos(mob, maxDistance, yRange, y, x, z, amplifier,
/// isRestricted)`.
#[allow(clippy::too_many_arguments)]
pub fn generate_random_pos(
    mob: &Mob,
    max_distance: i32,
    y_range: i32,
    y: i32,
    x: f64,
    z: f64,
    amplifier: f64,
    is_restricted: bool,
    random: &std::rc::Rc<std::cell::RefCell<Box<dyn crate::random::RandomSource>>>,
) -> Option<BlockPos> {
    let offset = {
        let mut r = random.borrow_mut();
        random_pos::generate_random_direction_within_radians(
            &mut **r, 0.0, max_distance as f64, y_range, y, x, z, amplifier,
        )?
    };
    let bp = {
        let mut r = random.borrow_mut();
        random_pos::generate_random_pos_toward_direction(mob, max_distance as f64, &mut **r, offset)
    };
    if goal_utils::is_outside_limits(mob, bp) || goal_utils::is_restricted_block(mob, is_restricted, bp) {
        return None;
    }
    let Some(level) = mob.level() else { return None };
    let max_y = level.min_build_height() + level.height();
    let bp = random_pos::move_up_out_of_solid(bp, max_y, |p| goal_utils::is_solid(mob, p));
    if goal_utils::has_malus(mob, bp) {
        None
    } else {
        Some(bp)
    }
}
