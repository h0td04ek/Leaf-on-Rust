//! Перенос `net.minecraft.util.valueproviders.ClampedInt`.

use crate::random::RandomSource;
use crate::util::mth;
use crate::util::valueproviders::int_provider::IntProvider;
use std::sync::Arc;

/// `ClampedInt(source, minInclusive, maxInclusive)` — кламп результата source.
#[derive(Clone)]
pub struct ClampedInt {
    source: Arc<dyn IntProvider>,
    min_inclusive: i32,
    max_inclusive: i32,
}

impl ClampedInt {
    pub fn of(source: Arc<dyn IntProvider>, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self { source, min_inclusive, max_inclusive }
    }
}

impl IntProvider for ClampedInt {
    fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        mth::clamp_i32(self.source.sample(random), self.min_inclusive, self.max_inclusive)
    }
    fn min_value(&self) -> i32 {
        self.min_inclusive.max(self.source.min_value())
    }
    fn max_value(&self) -> i32 {
        self.max_inclusive.min(self.source.max_value())
    }
    fn describe(&self) -> String {
        format!("clamped({}, {}, {})", self.min_inclusive, self.max_inclusive, self.source.describe())
    }
}
