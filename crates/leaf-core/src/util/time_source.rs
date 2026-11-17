//! Перенос `net.minecraft.util.TimeSource`.

/// `TimeUnit`-аналог для get(unit) — наносекунды/миллисекунды/секунды.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    NANOSECONDS,
    MICROSECONDS,
    MILLISECONDS,
    SECONDS,
}

impl TimeUnit {
    /// `unit.convert(nanos, NANOSECONDS)`.
    pub fn convert_from_nanos(self, nanos: i64) -> i64 {
        match self {
            Self::NANOSECONDS => nanos,
            Self::MICROSECONDS => nanos / 1_000,
            Self::MILLISECONDS => nanos / 1_000_000,
            Self::SECONDS => nanos / 1_000_000_000,
        }
    }
}

/// `TimeSource` — источник времени.
pub trait TimeSource {
    /// `get(TimeUnit)`.
    fn get(&self, unit: TimeUnit) -> i64;
}

/// `TimeSource.NanoTimeSource`.
pub trait NanoTimeSource: TimeSource {
    /// `getAsLong()` — наносекунды.
    fn get_nanos(&self) -> i64;

    fn get(&self, unit: TimeUnit) -> i64 {
        unit.convert_from_nanos(self.get_nanos())
    }
}
