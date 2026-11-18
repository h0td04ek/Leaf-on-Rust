//! Перенос `net.minecraft.world.damagesource.FallLocation` — место падения.

/// `FallLocation` — место, откуда упала сущность.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FallLocation {
    /// `GENERIC`.
    Generic,
    /// `LADDER`.
    Ladder,
    /// `VINES`.
    Vines,
    /// `TWISTING_VINES`.
    TwistingVines,
    /// `WEEPING_VINES`.
    WeepingVines,
    /// `SCAFFOLDING`.
    Scaffolding,
    /// `OTHER_CLIMBABLE`.
    OtherClimbable,
    /// `HONEY`.
    Honey,
    /// `WATER`.
    Water,
    /// `CANDLE`.
    Candle,
}

impl FallLocation {
    /// `values()`.
    pub const VALUES: [FallLocation; 10] = [
        FallLocation::Generic,
        FallLocation::Ladder,
        FallLocation::Vines,
        FallLocation::TwistingVines,
        FallLocation::WeepingVines,
        FallLocation::Scaffolding,
        FallLocation::OtherClimbable,
        FallLocation::Honey,
        FallLocation::Water,
        FallLocation::Candle,
        // Candle обрабатывается отдельно в Java через block check
    ];

    /// `languageKey()` — ключ перевода.
    pub fn language_key(&self) -> &'static str {
        match self {
            FallLocation::Generic => "death.fall",
            FallLocation::Ladder => "death.fall.ladder",
            FallLocation::Vines => "death.fall.vines",
            FallLocation::TwistingVines => "death.fall.vines",
            FallLocation::WeepingVines => "death.fall.vines",
            FallLocation::Scaffolding => "death.fall.scaffolding",
            FallLocation::OtherClimbable => "death.fall.climbable",
            FallLocation::Honey => "death.fall.honey",
            FallLocation::Water => "death.fall.water",
            FallLocation::Candle => "death.fall.candle",
        }
    }
}
