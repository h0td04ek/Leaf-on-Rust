//! Перенос `net.minecraft.util.valueproviders.UniformFloat`.

use crate::random::RandomSource;
use crate::util::mth;
use crate::util::valueproviders::float_provider::{FloatProvider, SampledFloat};

/// `UniformFloat(minInclusive, maxExclusive)` — равномерное [min, max).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UniformFloat {
    min_inclusive: f32,
    max_exclusive: f32,
}

impl UniformFloat {
    /// `of(minInclusive, maxExclusive)` — max <= min → IllegalArgumentException.
    pub fn of(min_inclusive: f32, max_exclusive: f32) -> Self {
        assert!(max_exclusive > min_inclusive, "Max must exceed min");
        Self { min_inclusive, max_exclusive }
    }
}

impl SampledFloat for UniformFloat {
    fn sample(&self, random: &mut dyn RandomSource) -> f32 {
        mth::random_between_f32(random, self.min_inclusive, self.max_exclusive)
    }
}

impl FloatProvider for UniformFloat {
    fn min_value(&self) -> f32 {
        self.min_inclusive
    }
    fn max_value(&self) -> f32 {
        self.max_exclusive
    }
}

impl std::fmt::Display for UniformFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}-{}]", self.min_inclusive, self.max_exclusive)
    }
}
