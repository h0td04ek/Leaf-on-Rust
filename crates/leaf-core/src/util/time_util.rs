//! Перенос `net.minecraft.util.TimeUtil`.

pub const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;
pub const NANOSECONDS_PER_MILLISECOND: i64 = 1_000_000;
pub const MILLISECONDS_PER_SECOND: i64 = 1_000;
pub const SECONDS_PER_HOUR: i64 = 3_600;
pub const SECONDS_PER_MINUTE: i32 = 60;

use crate::util::valueproviders::UniformInt;

/// `rangeOfSeconds(minInclusive, maxInclusive)` — диапазон тиков.
pub fn range_of_seconds(min_inclusive: i32, max_inclusive: i32) -> UniformInt {
    UniformInt::of(min_inclusive * 20, max_inclusive * 20)
}
