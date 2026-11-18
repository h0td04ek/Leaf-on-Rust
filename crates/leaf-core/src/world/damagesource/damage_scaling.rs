//! Перенос `net.minecraft.world.damagesource.DamageScaling`.

/// `DamageScaling` — масштабирование урона.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DamageScaling {
    /// `NEVER`.
    Never,
    /// `WHEN_CAUSED_BY_LIVING_NON_PLAYER`.
    WhenCausedByLivingNonPlayer,
    /// `ALWAYS`.
    Always,
}

impl DamageScaling {
    /// `values()`.
    pub const VALUES: [DamageScaling; 3] = [
        DamageScaling::Never,
        DamageScaling::WhenCausedByLivingNonPlayer,
        DamageScaling::Always,
    ];

    /// `getSerializedName()`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            DamageScaling::Never => "never",
            DamageScaling::WhenCausedByLivingNonPlayer => "when_caused_by_living_non_player",
            DamageScaling::Always => "always",
        }
    }
}

impl std::str::FromStr for DamageScaling {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(DamageScaling::Never),
            "when_caused_by_living_non_player" => Ok(DamageScaling::WhenCausedByLivingNonPlayer),
            "always" => Ok(DamageScaling::Always),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for DamageScaling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}
