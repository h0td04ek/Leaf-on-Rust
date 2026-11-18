//! Перенос `net.minecraft.world.damagesource.DamageEffects`.

/// `DamageEffects` — эффекты урона (звук).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DamageEffects {
    /// `HURT` — `player_hurt`.
    Hurt,
    /// `THORNS` — `player_hurt`.
    Thorns,
    /// `DROWNING` — `player_hurt_drown`.
    Drowning,
    /// `BURNING` — `player_hurt_on_fire`.
    Burning,
    /// `POKING` — `player_hurt_sweet_berry_bush`.
    Poking,
    /// `FREEZING` — `player_hurt_freeze`.
    Freezing,
}

impl DamageEffects {
    /// `values()`.
    pub const VALUES: [DamageEffects; 6] = [
        DamageEffects::Hurt,
        DamageEffects::Thorns,
        DamageEffects::Drowning,
        DamageEffects::Burning,
        DamageEffects::Poking,
        DamageEffects::Freezing,
    ];

    /// `getSerializedName()`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            DamageEffects::Hurt => "hurt",
            DamageEffects::Thorns => "thorns",
            DamageEffects::Drowning => "drowning",
            DamageEffects::Burning => "burning",
            DamageEffects::Poking => "poking",
            DamageEffects::Freezing => "freezing",
        }
    }

    /// `sound()` — идентификатор звука эффекта.
    pub fn sound(&self) -> &'static str {
        match self {
            DamageEffects::Hurt | DamageEffects::Thorns => "minecraft:entity.player.hurt",
            DamageEffects::Drowning => "minecraft:entity.player.hurt_drown",
            DamageEffects::Burning => "minecraft:entity.player.hurt_on_fire",
            DamageEffects::Poking => "minecraft:entity.player.hurt_sweet_berry_bush",
            DamageEffects::Freezing => "minecraft:entity.player.hurt_freeze",
        }
    }
}

impl std::str::FromStr for DamageEffects {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hurt" => Ok(DamageEffects::Hurt),
            "thorns" => Ok(DamageEffects::Thorns),
            "drowning" => Ok(DamageEffects::Drowning),
            "burning" => Ok(DamageEffects::Burning),
            "poking" => Ok(DamageEffects::Poking),
            "freezing" => Ok(DamageEffects::Freezing),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for DamageEffects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}
