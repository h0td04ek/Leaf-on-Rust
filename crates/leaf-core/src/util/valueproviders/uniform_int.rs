//! Перенос `net.minecraft.util.valueproviders.UniformInt`.

use crate::random::RandomSource;
use crate::util::mth;
use crate::util::valueproviders::int_provider::IntProvider;

/// `UniformInt(minInclusive, maxInclusive)` — равномерное распределение.
#[derive(Clone, Debug, PartialEq)]
pub struct UniformInt {
    min_inclusive: i32,
    max_inclusive: i32,
}

impl UniformInt {
    /// `of(minInclusive, maxInclusive)`.
    pub fn of(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self { min_inclusive, max_inclusive }
    }
}

impl IntProvider for UniformInt {
    fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        mth::random_between_inclusive(random, self.min_inclusive, self.max_inclusive)
    }
    fn min_value(&self) -> i32 {
        self.min_inclusive
    }
    fn max_value(&self) -> i32 {
        self.max_inclusive
    }
    fn describe(&self) -> String {
        format!("[{}-{}]", self.min_inclusive, self.max_inclusive)
    }
}

impl std::fmt::Display for UniformInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}-{}]", self.min_inclusive, self.max_inclusive)
    }
}
