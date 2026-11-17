//! Перенос `net.minecraft.util.valueproviders.BiasedToBottomInt`.

use crate::random::RandomSource;
use crate::util::valueproviders::int_provider::IntProvider;

/// `BiasedToBottomInt(minInclusive, maxInclusive)` — смещение к минимуму.
#[derive(Clone, Debug, PartialEq)]
pub struct BiasedToBottomInt {
    min_inclusive: i32,
    max_inclusive: i32,
}

impl BiasedToBottomInt {
    pub fn of(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self { min_inclusive, max_inclusive }
    }
}

impl IntProvider for BiasedToBottomInt {
    fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        let inner = random.next_int_bound(self.max_inclusive - self.min_inclusive + 1);
        self.min_inclusive + random.next_int_bound(inner + 1)
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

impl std::fmt::Display for BiasedToBottomInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}-{}]", self.min_inclusive, self.max_inclusive)
    }
}
