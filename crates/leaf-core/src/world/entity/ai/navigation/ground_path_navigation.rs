//! `GroundPathNavigation` — Ground-навигация (WalkNodeEvaluator).
//! Реализация — `PathNavigation` с `NavKind::Ground`.

use super::path_navigation::{NavKind, PathNavigation};

/// `GroundPathNavigation(mob, level)`-эквивалент.
pub fn new(max_visited_nodes: i32) -> PathNavigation {
    PathNavigation::new(NavKind::Ground, max_visited_nodes)
}

/// `setAvoidSun`.
pub fn set_avoid_sun(nav: &mut PathNavigation, v: bool) {
    nav.set_avoid_sun(v);
}

/// `setCanWalkOverFences`.
pub fn set_can_walk_over_fences(nav: &mut PathNavigation, v: bool) {
    nav.set_can_walk_over_fences(v);
}

/// `setCanPathToTargetsBelowSurface`.
pub fn set_can_path_to_targets_below_surface(nav: &mut PathNavigation, v: bool) {
    nav.set_can_path_to_targets_below_surface(v);
}

/// `hasValidPathType`.
pub fn has_valid_path_type(nav: &PathNavigation, path_type: crate::world::level::pathfinder::path_type::PathType) -> bool {
    nav.has_valid_path_type(path_type)
}
