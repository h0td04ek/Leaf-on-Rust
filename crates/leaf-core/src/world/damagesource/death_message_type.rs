//! Перенос `net.minecraft.world.damagesource.DeathMessageType`.

/// `DeathMessageType` — тип сообщения о смерти.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeathMessageType {
    /// `DEFAULT`.
    Default,
    /// `FALL_VARIANTS`.
    FallVariants,
    /// `INTENTIONAL_GAME_DESIGN`.
    IntentionalGameDesign,
}

impl DeathMessageType {
    /// `values()`.
    pub const VALUES: [DeathMessageType; 3] = [
        DeathMessageType::Default,
        DeathMessageType::FallVariants,
        DeathMessageType::IntentionalGameDesign,
    ];

    /// `getSerializedName()`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            DeathMessageType::Default => "default",
            DeathMessageType::FallVariants => "fall_variants",
            DeathMessageType::IntentionalGameDesign => "intentional_game_design",
        }
    }
}

impl std::str::FromStr for DeathMessageType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(DeathMessageType::Default),
            "fall_variants" => Ok(DeathMessageType::FallVariants),
            "intentional_game_design" => Ok(DeathMessageType::IntentionalGameDesign),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for DeathMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}
