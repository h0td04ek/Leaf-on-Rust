//! Перенос `net.minecraft.core` — базовые core-типы (Vec3i, Direction, ...).

pub mod block_pos;
pub mod direction;
pub mod global_pos;
pub mod holder;
pub mod quart_pos;
pub mod registry;
pub mod registry_synchronization;
pub mod section_pos;
pub mod shared_constants;
pub mod vec3i;

pub use direction::{Axis, AxisDirection, Direction, Plane};
pub use vec3i::Vec3i;
