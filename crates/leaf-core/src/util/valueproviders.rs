//! Перенос `net.minecraft.util.valueproviders`.
//!
//! Типы `IntProviderType`/`FloatProviderType` и DFU-кодеки отложены —
//! они целиком завязаны на реестры `BuiltInRegistries` и DataFixerUpper.
//! Когда DFU будет портирован, сюда добавятся `*_type.rs` и codec-методы.

pub mod biased_to_bottom_int;
pub mod clamped_int;
pub mod clamped_normal_float;
pub mod clamped_normal_int;
pub mod constant_float;
pub mod constant_int;
pub mod float_provider;
pub mod int_provider;
pub mod multiplied_floats;
pub mod trapezoid_float;
pub mod uniform_float;
pub mod uniform_int;
pub mod weighted_list_int;

pub use biased_to_bottom_int::BiasedToBottomInt;
pub use clamped_int::ClampedInt;
pub use clamped_normal_float::ClampedNormalFloat;
pub use clamped_normal_int::ClampedNormalInt;
pub use constant_float::{ConstantFloat, ZERO as CONSTANT_FLOAT_ZERO};
pub use constant_int::{ConstantInt, ZERO as CONSTANT_INT_ZERO};
pub use float_provider::{FloatProvider, SampledFloat};
pub use int_provider::IntProvider;
pub use multiplied_floats::MultipliedFloats;
pub use trapezoid_float::TrapezoidFloat;
pub use uniform_float::UniformFloat;
pub use uniform_int::UniformInt;
pub use weighted_list_int::WeightedListInt;
