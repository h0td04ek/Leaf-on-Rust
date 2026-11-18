//! Перенос `net.minecraft.world.entity.ai.behavior.BehaviorUtils` —
//! часть, нужная целям (swimmable pos и др.).

use crate::world::entity::ai::util::default_random_pos;
use crate::world::entity::mob::Mob;
use crate::world::level::pathfinder::path_computation_type::PathComputationType;
use crate::world::phys::vec3::Vec3;

/// `getRandomSwimmablePos(pathfinder, radius, verticalDistance)` —
/// до 10 попыток найти точку, проходимую для воды.
pub fn get_random_swimmable_pos(
    mob: &Mob,
    radius: i32,
    vertical_distance: i32,
) -> Option<Vec3> {
    let mut pos = default_random_pos::get_pos(mob, radius, vertical_distance);
    let mut i = 0;
    while let Some(p) = pos {
        let bp = crate::core::block_pos::BlockPos::containing(p.x, p.y, p.z);
        let pathfindable = mob.level().map(|l| {
            l.get_block_state(bp)
                .map(|s| {
                    let level_ref: &dyn crate::world::level::level_reader::LevelReader = l.as_ref();
                    s.is_pathfindable(level_ref, bp, PathComputationType::Water)
                })
                .unwrap_or(false)
        }).unwrap_or(false);
        if pathfindable || i >= 10 {
            return if pathfindable { Some(p) } else { None };
        }
        i += 1;
        pos = default_random_pos::get_pos(mob, radius, vertical_distance);
    }
    pos
}
