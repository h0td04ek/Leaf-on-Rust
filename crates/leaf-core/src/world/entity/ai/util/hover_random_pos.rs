//! Перенос `net.minecraft.world.entity.ai.util.HoverRandomPos`.

use crate::world::entity::ai::util::{goal_utils, land_random_pos, random_pos};
use crate::world::entity::mob::Mob;
use crate::world::phys::vec3::Vec3;

/// `getPos(mob, radius, yRange, x, z, amplifier, maxSwimUp, minSwimUp)`.
#[allow(clippy::too_many_arguments)]
pub fn get_pos(
    mob: &Mob,
    radius: i32,
    y_range: i32,
    x: f64,
    z: f64,
    amplifier: f64,
    max_swim_up: i32,
    min_swim_up: i32,
) -> Option<Vec3> {
    let is_restricted = goal_utils::mob_restricted(mob, radius as f64);
    let random = mob.random.clone();
    random_pos::generate_random_pos(mob, || {
        let offset = {
            let mut r = random.borrow_mut();
            random_pos::generate_random_direction_within_radians(
                &mut **r, 0.0, radius as f64, y_range, 0, x, z, amplifier,
            )?
        };
        let mut bp = land_random_pos::generate_random_pos_toward_direction(
            mob, radius as f64, is_restricted, offset,
        )?;
        let swim_up = {
            let mut r = random.borrow_mut();
            r.next_int_bound(max_swim_up - min_swim_up + 1) + min_swim_up
        };
        let Some(level) = mob.level() else { return None };
        let max_y = level.min_build_height() + level.height();
        bp = random_pos::move_up_to_above_solid(bp, swim_up, max_y, |p| goal_utils::is_solid(mob, p));
        if !goal_utils::is_water(mob, bp) && !goal_utils::has_malus(mob, bp) {
            Some(bp)
        } else {
            None
        }
    })
}
