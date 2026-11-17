//! Перенос `net.minecraft.world.waypoints`.

pub mod tracked_waypoint;
pub mod waypoint;

pub use tracked_waypoint::{PitchDirection, TrackedWaypoint, TrackedWaypointCamera, TrackedWaypointType};
pub use waypoint::{waypoint_style_assets_default, WaypointIcon, WAYPOINT_STYLE_ASSET_ROOT};
