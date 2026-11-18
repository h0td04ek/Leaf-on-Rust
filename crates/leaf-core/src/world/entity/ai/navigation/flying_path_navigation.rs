//! `FlyingPathNavigation` — летающая навигация (FlyNodeEvaluator).

use super::path_navigation::{NavKind, PathNavigation};

/// `FlyingPathNavigation(mob, level)`-эквивалент.
pub fn new(max_visited_nodes: i32) -> PathNavigation {
    PathNavigation::new(NavKind::Flying, max_visited_nodes)
}
