//! Перенос `net.minecraft.util.valueproviders.ConstantFloat`.

use crate::random::RandomSource;
use crate::util::valueproviders::float_provider::{FloatProvider, SampledFloat};

/// `ConstantFloat(value)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConstantFloat {
    value: f32,
}

pub const ZERO: ConstantFloat = ConstantFloat { value: 0.0 };

impl ConstantFloat {
    pub fn of(value: f32) -> Self {
        if value == 0.0 { ZERO } else { Self { value } }
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }
}

impl SampledFloat for ConstantFloat {
    fn sample(&self, _random: &mut dyn RandomSource) -> f32 {
        self.value
    }
}

impl FloatProvider for ConstantFloat {
    fn min_value(&self) -> f32 {
        self.value
    }
    fn max_value(&self) -> f32 {
        self.value
    }
}

impl std::fmt::Display for ConstantFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
