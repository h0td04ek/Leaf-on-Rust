//! Перенос `net.minecraft.world.item.enchantment.effects.DamageImmunity`.

/// `DamageImmunity` — unit-запись: иммунитет к типу урона
/// (в Java — `record DamageImmunity()` с `MapCodec.unitCodec`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamageImmunity;
