//! Перенос `net.minecraft.util.valueproviders.ClampedNormalInt`.

use crate::random::RandomSource;
use crate::util::mth;
use crate::util::valueproviders::int_provider::IntProvider;

/// `ClampedNormalInt(mean, deviation, minInclusive, maxInclusive)` — нормальное
/// распределение с клампом.
#[derive(Clone, Debug, PartialEq)]
pub struct ClampedNormalInt {
    mean: f32,
    deviation: f32,
    min_inclusive: i32,
    max_inclusive: i32,
}

impl ClampedNormalInt {
    pub fn of(mean: f32, deviation: f32, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self { mean, deviation, min_inclusive, max_inclusive }
    }

    /// `sample(random, mean, deviation, minInclusive, maxInclusive)`.
    pub fn sample_with(
        random: &mut dyn RandomSource,
        mean: f32,
        deviation: f32,
        min_inclusive: f32,
        max_inclusive: f32,
    ) -> i32 {
        mth::clamp_f32(mth::normal_f32(random, mean, deviation), min_inclusive, max_inclusive) as i32
    }
}

impl IntProvider for ClampedNormalInt {
    fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        Self::sample_with(
            random,
            self.mean,
            self.deviation,
            self.min_inclusive as f32,
            self.max_inclusive as f32,
        )
    }
    fn min_value(&self) -> i32 {
        self.min_inclusive
    }
    fn max_value(&self) -> i32 {
        self.max_inclusive
    }
    fn describe(&self) -> String {
        format!("normal({}, {}) in [{}-{}]", self.mean, self.deviation, self.min_inclusive, self.max_inclusive)
    }
}
