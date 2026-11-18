//! `WaterBoundPathNavigation` — водная навигация
//! (SwimNodeEvaluator, allowBreaching для Dolphin).

use super::path_navigation::{NavKind, PathNavigation};

/// `WaterBoundPathNavigation(mob, level)`-эквивалент.
pub fn new(max_visited_nodes: i32) -> PathNavigation {
    PathNavigation::new(NavKind::WaterBound, max_visited_nodes)
}
