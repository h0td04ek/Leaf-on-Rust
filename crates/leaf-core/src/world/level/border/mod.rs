//! `net.minecraft.world.level.border` — граница мира.

pub mod border_status;
pub mod border_change_listener;
pub mod world_border;

pub use border_status::BorderStatus;
pub use border_change_listener::BorderChangeListener;
pub use world_border::{WorldBorder, Settings};
