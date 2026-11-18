//! `net.minecraft.world.entity.ai.navigation` — навигация мобов.

pub mod mob_navigation;
pub mod path_navigation;
pub mod ground_path_navigation;
pub mod flying_path_navigation;
pub mod amphibious_path_navigation;
pub mod water_bound_path_navigation;
pub mod wall_climber_navigation;

pub use mob_navigation::MobNavigation;
pub use path_navigation::{NavKind, PathNavigation};
