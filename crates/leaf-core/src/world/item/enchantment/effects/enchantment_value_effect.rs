//! Перенос `net.minecraft.world.item.enchantment.effects.EnchantmentValueEffect` —
//! эффекты, преобразующие числовое значение по уровню зачарования.

use crate::random::RandomSource;
use crate::world::item::enchantment::level_based_value::LevelBasedValue;

/// `EnchantmentValueEffect` — tagged union (`enchantment_value_effect_type`).
#[derive(Clone, Debug, PartialEq)]
pub enum EnchantmentValueEffect {
    /// `add`.
    Add(Box<LevelBasedValue>),
    /// `multiply`.
    Multiply(Box<LevelBasedValue>),
    /// `set`.
    Set(Box<LevelBasedValue>),
    /// `all_of`.
    AllOf(Vec<EnchantmentValueEffect>),
    /// `remove_binomial`.
    RemoveBinomial(Box<LevelBasedValue>),
    /// `exponential`.
    ScaleExponentially { base: Box<LevelBasedValue>, exponent: Box<LevelBasedValue> },
}

impl EnchantmentValueEffect {
    /// `AddValue`.
    pub fn add(value: LevelBasedValue) -> Self {
        Self::Add(Box::new(value))
    }
    /// `MultiplyValue`.
    pub fn multiply(factor: LevelBasedValue) -> Self {
        Self::Multiply(Box::new(factor))
    }
    /// `SetValue`.
    pub fn set(value: LevelBasedValue) -> Self {
        Self::Set(Box::new(value))
    }
    /// `AllOf.ValueEffects`.
    pub fn all_of(effects: Vec<EnchantmentValueEffect>) -> Self {
        Self::AllOf(effects)
    }
    /// `RemoveBinomial`.
    pub fn remove_binomial(chance: LevelBasedValue) -> Self {
        Self::RemoveBinomial(Box::new(chance))
    }
    /// `ScaleExponentially`.
    pub fn scale_exponentially(base: LevelBasedValue, exponent: LevelBasedValue) -> Self {
        Self::ScaleExponentially { base: Box::new(base), exponent: Box::new(exponent) }
    }

    /// `process(enchantmentLevel, random, value)`.
    pub fn process(
        &self,
        enchantment_level: i32,
        random: &mut dyn RandomSource,
        value: f32,
    ) -> f32 {
        match self {
            EnchantmentValueEffect::Add(v) => value + v.calculate(enchantment_level),
            EnchantmentValueEffect::Multiply(f) => value * f.calculate(enchantment_level),
            EnchantmentValueEffect::Set(v) => v.calculate(enchantment_level),
            EnchantmentValueEffect::AllOf(effects) => {
                let mut v = value;
                for e in effects {
                    v = e.process(enchantment_level, random, v);
                }
                v
            }
            EnchantmentValueEffect::RemoveBinomial(chance) => {
                let f = chance.calculate(enchantment_level);
                let mut i = 0i32;
                if !(value <= 128.0) && !(value * f < 20.0) && !(value * (1.0 - f) < 20.0) {
                    let floor = (value * f).floor() as f64;
                    let sq = ((value * f * (1.0 - f)) as f64).sqrt();
                    i = (random.next_gaussian() * sq + floor).round() as i32;
                    i = i.clamp(0, value as i32);
                } else {
                    for _ in 0..(value as i32) {
                        if random.next_float() < f {
                            i += 1;
                        }
                    }
                }
                value - i as f32
            }
            EnchantmentValueEffect::ScaleExponentially { base, exponent } => {
                value * base.calculate(enchantment_level).powf(exponent.calculate(enchantment_level))
            }
        }
    }
}
