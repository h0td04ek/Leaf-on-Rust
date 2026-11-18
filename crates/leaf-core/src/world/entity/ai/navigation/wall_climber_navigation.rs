//! `WallClimberNavigation` — лазание по стенам
//! (extends GroundPathNavigation).

use super::path_navigation::{NavKind, PathNavigation};

/// `WallClimberNavigation(mob, level)`-эквивалент.
pub fn new(max_visited_nodes: i32) -> PathNavigation {
    PathNavigation::new(NavKind::WallClimber, max_visited_nodes)
}
