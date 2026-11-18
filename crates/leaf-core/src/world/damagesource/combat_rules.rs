//! Перенос `net.minecraft.world.damagesource.CombatRules` — формулы брони.

use crate::util::mth;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::entity::entity::Entity;

/// `CombatRules` — формулы снижения урона бронёй и чары.
pub struct CombatRules;

impl CombatRules {
    /// `MAX_ARMOR`.
    pub const MAX_ARMOR: f32 = 20.0;
    /// `ARMOR_PROTECTION_DIVIDER`.
    pub const ARMOR_PROTECTION_DIVIDER: f32 = 25.0;
    /// `BASE_ARMOR_TOUGHNESS`.
    pub const BASE_ARMOR_TOUGHNESS: f32 = 2.0;
    /// `MIN_ARMOR_RATIO`.
    pub const MIN_ARMOR_RATIO: f32 = 0.2;
    /// `NUM_ARMOR_ITEMS`.
    pub const NUM_ARMOR_ITEMS: usize = 4;

    /// `getDamageAfterAbsorb(entity, damageAmount, damageSource, armorValue, armorToughness)`.
    /// EnchantmentHelper-модификатор принимает опциональный hook — по умолчанию identity.
    pub fn get_damage_after_absorb(
        entity: &Entity,
        damage_amount: f32,
        damage_source: &DamageSource,
        armor_value: f32,
        armor_toughness: f32,
        enchant_modifier: Option<&dyn Fn(f32) -> f32>,
    ) -> f32 {
        let _ = entity;
        let _ = damage_source;
        let f = 2.0 + armor_toughness / 4.0;
        let f1 = mth::clamp_f32(
            armor_value - damage_amount / f,
            armor_value * 0.2,
            f32::MAX,
        );
        let f2 = f1 / 25.0;
        let f3 = enchant_modifier
            .map(|m| mth::clamp_f32(m(f2), 0.0, 1.0))
            .unwrap_or(f2);
        let f4 = 1.0 - f3;
        damage_amount * f4
    }

    /// `getDamageAfterMagicAbsorb(damageAmount, enchantModifiers)`.
    pub fn get_damage_after_magic_absorb(damage_amount: f32, enchant_modifiers: f32) -> f32 {
        let f = mth::clamp_f32(enchant_modifiers, 0.0, f32::MAX);
        damage_amount * (1.0 - f / 25.0)
    }
}
