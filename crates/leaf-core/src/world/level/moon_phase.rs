//! Перенос `net.minecraft.world.level.MoonPhase`.

/// `MoonPhase` — 8 фаз луны.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MoonPhase {
    /// `FULL_MOON` (index 0).
    FullMoon,
    /// `WANING_GIBBOUS` (index 1).
    WaningGibbous,
    /// `THIRD_QUARTER` (index 2).
    ThirdQuarter,
    /// `WANING_CRESCENT` (index 3).
    WaningCrescent,
    /// `NEW_MOON` (index 4).
    NewMoon,
    /// `WAXING_CRESCENT` (index 5).
    WaxingCrescent,
    /// `FIRST_QUARTER` (index 6).
    FirstQuarter,
    /// `WAXING_GIBBOUS` (index 7).
    WaxingGibbous,
}

impl MoonPhase {
    /// Все фазы в порядке объявления.
    pub const VALUES: [MoonPhase; 8] = [
        MoonPhase::FullMoon,
        MoonPhase::WaningGibbous,
        MoonPhase::ThirdQuarter,
        MoonPhase::WaningCrescent,
        MoonPhase::NewMoon,
        MoonPhase::WaxingCrescent,
        MoonPhase::FirstQuarter,
        MoonPhase::WaxingGibbous,
    ];

    /// `COUNT`.
    pub const COUNT: usize = 8;
    /// `PHASE_LENGTH`.
    pub const PHASE_LENGTH: i32 = 24000;

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            MoonPhase::FullMoon => "full_moon",
            MoonPhase::WaningGibbous => "waning_gibbous",
            MoonPhase::ThirdQuarter => "third_quarter",
            MoonPhase::WaningCrescent => "waning_crescent",
            MoonPhase::NewMoon => "new_moon",
            MoonPhase::WaxingCrescent => "waxing_crescent",
            MoonPhase::FirstQuarter => "first_quarter",
            MoonPhase::WaxingGibbous => "waxing_gibbous",
        }
    }

    /// `byName`-эквивалент.
    pub fn by_name(name: &str) -> Option<MoonPhase> {
        Self::VALUES.iter().copied().find(|p| p.get_serialized_name() == name)
    }

    /// `index`.
    pub fn index(&self) -> i32 {
        *self as i32
    }

    /// `startTick`.
    pub fn start_tick(&self) -> i32 {
        self.index() * Self::PHASE_LENGTH
    }
}
