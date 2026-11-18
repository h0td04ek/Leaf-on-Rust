//! `AmphibiousPathNavigation` — амфибийная навигация
//! (AmphibiousNodeEvaluator).

use super::path_navigation::{NavKind, PathNavigation};

/// `AmphibiousPathNavigation(mob, level)`-эквивалент.
pub fn new(max_visited_nodes: i32) -> PathNavigation {
    PathNavigation::new(NavKind::Amphibious, max_visited_nodes)
}
