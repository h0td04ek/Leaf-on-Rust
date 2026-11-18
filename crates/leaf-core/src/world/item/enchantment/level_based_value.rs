//! Перенос `net.minecraft.world.item.enchantment.LevelBasedValue` —
//! значения, зависящие от уровня зачарования.

use crate::util::mth;

/// `LevelBasedValue` — tagged union (`enchantment_level_based_value_type`).
#[derive(Clone, Debug, PartialEq)]
pub enum LevelBasedValue {
    /// `constant`.
    Constant(ConstantValue),
    /// `linear`.
    Linear(LinearValue),
    /// `clamped`.
    Clamped(ClampedValue),
    /// `fraction`.
    Fraction(FractionValue),
    /// `levels_squared`.
    LevelsSquared(LevelsSquaredValue),
    /// `exponent`.
    Exponent(ExponentValue),
    /// `lookup`.
    Lookup(LookupValue),
}

/// `LevelBasedValue.Constant`.
#[derive(Clone, Debug, PartialEq)]
pub struct ConstantValue(pub f32);
impl ConstantValue {
    pub fn exactly(value: f32) -> Self { Self(value) }
}

/// `LevelBasedValue.Linear`.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearValue {
    pub base: f32,
    pub per_level_above_first: f32,
}
impl LinearValue {
    pub fn new(base: f32, per_level_above_first: f32) -> Self {
        Self { base, per_level_above_first }
    }
    pub fn per_level(per_level: f32) -> Self {
        Self { base: per_level, per_level_above_first: per_level }
    }
}

/// `LevelBasedValue.Clamped`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClampedValue {
    pub value: Box<LevelBasedValue>,
    pub min: f32,
    pub max: f32,
}

/// `LevelBasedValue.Fraction`.
#[derive(Clone, Debug, PartialEq)]
pub struct FractionValue {
    pub numerator: Box<LevelBasedValue>,
    pub denominator: Box<LevelBasedValue>,
}

/// `LevelBasedValue.LevelsSquared`.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelsSquaredValue {
    pub added: f32,
}

/// `LevelBasedValue.Exponent`.
#[derive(Clone, Debug, PartialEq)]
pub struct ExponentValue {
    pub base: Box<LevelBasedValue>,
    pub power: Box<LevelBasedValue>,
}

/// `LevelBasedValue.Lookup`.
#[derive(Clone, Debug, PartialEq)]
pub struct LookupValue {
    pub values: Vec<f32>,
    pub fallback: Box<LevelBasedValue>,
}

impl LevelBasedValue {
    /// `LevelBasedValue.constant`.
    pub fn constant(value: f32) -> Self {
        Self::Constant(ConstantValue(value))
    }
    /// `LevelBasedValue.perLevel(base, perLevelAfterFirst)`.
    pub fn per_level(base: f32, per_level_above_first: f32) -> Self {
        Self::Linear(LinearValue::new(base, per_level_above_first))
    }
    /// `LevelBasedValue.perLevel(perLevel)`.
    pub fn per_level_only(per_level: f32) -> Self {
        Self::Linear(LinearValue::per_level(per_level))
    }
    /// `LevelBasedValue.lookup`.
    pub fn lookup(values: Vec<f32>, fallback: LevelBasedValue) -> Self {
        Self::Lookup(LookupValue { values, fallback: Box::new(fallback) })
    }

    /// `calculate(level)`.
    pub fn calculate(&self, level: i32) -> f32 {
        match self {
            LevelBasedValue::Constant(v) => v.0,
            LevelBasedValue::Linear(v) => v.base + v.per_level_above_first * (level - 1) as f32,
            LevelBasedValue::Clamped(v) => mth::clamp_f32(v.value.calculate(level), v.min, v.max),
            LevelBasedValue::Fraction(v) => {
                let d = v.denominator.calculate(level);
                if d == 0.0 { 0.0 } else { v.numerator.calculate(level) / d }
            }
            LevelBasedValue::LevelsSquared(v) => (level * level) as f32 + v.added,
            LevelBasedValue::Exponent(v) => {
                v.base.calculate(level).powf(v.power.calculate(level))
            }
            LevelBasedValue::Lookup(v) => {
                if level <= v.values.len() as i32 {
                    v.values[(level - 1).max(0) as usize]
                } else {
                    v.fallback.calculate(level)
                }
            }
        }
    }
}
