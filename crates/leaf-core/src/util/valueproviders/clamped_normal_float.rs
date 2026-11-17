//! Перенос `net.minecraft.util.valueproviders.ClampedNormalFloat`.

use crate::random::RandomSource;
use crate::util::mth;
use crate::util::valueproviders::float_provider::{FloatProvider, SampledFloat};

/// `ClampedNormalFloat(mean, deviation, min, max)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClampedNormalFloat {
    mean: f32,
    deviation: f32,
    min: f32,
    max: f32,
}

impl ClampedNormalFloat {
    pub fn of(mean: f32, deviation: f32, min: f32, max: f32) -> Self {
        Self { mean, deviation, min, max }
    }

    /// `sample(random, mean, deviation, min, max)`.
    pub fn sample_with(random: &mut dyn RandomSource, mean: f32, deviation: f32, min: f32, max: f32) -> f32 {
        mth::clamp_f32(mth::normal_f32(random, mean, deviation), min, max)
    }
}

impl SampledFloat for ClampedNormalFloat {
    fn sample(&self, random: &mut dyn RandomSource) -> f32 {
        Self::sample_with(random, self.mean, self.deviation, self.min, self.max)
    }
}

impl FloatProvider for ClampedNormalFloat {
    fn min_value(&self) -> f32 {
        self.min
    }
    fn max_value(&self) -> f32 {
        self.max
    }
}

impl std::fmt::Display for ClampedNormalFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "normal({}, {}) in [{}-{}]", self.mean, self.deviation, self.min, self.max)
    }
}
