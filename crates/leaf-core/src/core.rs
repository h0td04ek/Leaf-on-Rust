//! Перенос `net.minecraft.core` — базовые core-типы (Vec3i, Direction, ...).

pub mod direction;
pub mod vec3i;

pub use direction::{Axis, AxisDirection, Direction, Plane};
pub use vec3i::Vec3i;
