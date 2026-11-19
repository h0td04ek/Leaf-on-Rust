//! Перенос `net.minecraft.world.item.enchantment.EnchantmentTarget`.

/// `EnchantmentTarget` — цель зачарования (`attacker`/`damaging_entity`/`victim`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnchantmentTarget {
    /// `ATTACKER`.
    Attacker,
    /// `DAMAGING_ENTITY`.
    DamagingEntity,
    /// `VICTIM`.
    Victim,
}

impl EnchantmentTarget {
    /// serialized name.
    pub fn serialized_name(&self) -> &'static str {
        match self {
            Self::Attacker => "attacker",
            Self::DamagingEntity => "damaging_entity",
            Self::Victim => "victim",
        }
    }

    /// `StringRepresentable.fromEnum`-поиск.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "attacker" => Some(Self::Attacker),
            "damaging_entity" => Some(Self::DamagingEntity),
            "victim" => Some(Self::Victim),
            _ => None,
        }
    }
}
