//! Перенос `net.minecraft.world.entity.ai.util.GoalUtils`.

use crate::core::block_pos::BlockPos;
use crate::tags::fluid_tags::FluidTags;
use crate::world::entity::ai::navigation::mob_navigation::MobNavigation;
use crate::world::entity::mob::{Mob, MobPathing};
use crate::world::level::pathfinder::node_evaluator::NodeEvaluator;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::level::pathfinder::walk_node_evaluator::WalkNodeEvaluator;
use crate::world::level::pathfinder::pathfinding_context::PathfindingContext;
use crate::world::phys::vec3::Vec3;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// `hasGroundPathNavigation(mob)`.
pub fn has_ground_path_navigation(mob: &Mob) -> bool {
    mob.navigation
        .as_ref()
        .map(|n| n.borrow().can_navigate_ground())
        .unwrap_or(false)
}

/// `mobRestricted(mob, radius)`.
pub fn mob_restricted(mob: &Mob, radius: f64) -> bool {
    mob.has_home()
        && mob.home_position
            .closer_to_center_than(mob.living.entity.position(), mob.home_radius as f64 + radius + 1.0)
}

/// `isOutsideLimits(pos, mob)`.
pub fn is_outside_limits(mob: &Mob, pos: BlockPos) -> bool {
    mob.level()
        .map(|l| !l.is_inside_build_height(pos.y))
        .unwrap_or(true)
}

/// `isRestricted(isRestricted, mob, pos)` — BlockPos-вариант.
pub fn is_restricted_block(mob: &Mob, is_restricted: bool, pos: BlockPos) -> bool {
    is_restricted && !mob.is_within_home(pos)
}

/// `isRestricted(isRestricted, mob, pos)` — Vec3-вариант.
pub fn is_restricted_vec(mob: &Mob, is_restricted: bool, pos: Vec3) -> bool {
    is_restricted && !mob.is_within_home_vec(pos)
}

/// `isNotStable(navigation, pos)`.
pub fn is_not_stable(mob: &Mob, pos: BlockPos) -> bool {
    mob.navigation
        .as_ref()
        .map(|n| !n.borrow().is_stable_destination(mob, pos))
        .unwrap_or(false)
}

/// `isWater(mob, pos)`.
pub fn is_water(mob: &Mob, pos: BlockPos) -> bool {
    mob.level()
        .and_then(|l| l.get_fluid_state(pos))
        .map(|f| f.is_in_named_tag(&FluidTags::water()) || f.is_water())
        .unwrap_or(false)
}

/// `hasMalus(mob, pos)` — malus != 0 для path type в точке.
pub fn has_malus(mob: &Mob, pos: BlockPos) -> bool {
    let path_type = walk_path_type_static(mob, pos);
    mob.get_pathfinding_malus(path_type) != 0.0
}

/// `WalkNodeEvaluator.getPathTypeStatic(mob, pos)` — статический
/// path type без mob-оценщика (BlockState-only классификация).
pub fn walk_path_type_static(mob: &Mob, pos: BlockPos) -> PathType {
    let Some(level) = mob.level() else { return PathType::Blocked };
    let level_ref: &dyn crate::world::level::level_reader::LevelReader = level.as_ref();
    let mut ctx = PathfindingContext::new(level_ref, level.path_type_cache(), pos);
    WalkNodeEvaluator::get_path_type_static(&mut ctx, pos.x, pos.y, pos.z)
}

/// `isSolid(mob, pos)`.
pub fn is_solid(mob: &Mob, pos: BlockPos) -> bool {
    let Some(level) = mob.level() else { return false };
    let level_ref: &dyn crate::world::level::level_reader::LevelReader = level.as_ref();
    level
        .get_block_state(pos)
        .map(|s| s.is_solid(level_ref, pos))
        .unwrap_or(false)
}
