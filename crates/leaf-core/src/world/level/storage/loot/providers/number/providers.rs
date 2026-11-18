//! Конкретные `NumberProvider` (`NumberProviders`).

use crate::resources::identifier::Identifier;
use crate::world::item::enchantment::level_based_value::LevelBasedValue;
use crate::world::level::storage::loot::providers::score::scoreboard_name_provider::{
    ContextScoreboardNameProvider, FixedScoreboardNameProvider, ScoreboardNameProvider,
};
use crate::world::phys::vec3::Vec3;

/// `ConstantValue`.
#[derive(Clone, Debug, PartialEq)]
pub struct ConstantValue(pub f32);
impl ConstantValue {
    pub fn exactly(value: f32) -> Self { Self(value) }
}

/// `UniformGenerator`.
pub struct UniformGenerator {
    pub min: Box<dyn NumberProviderTrait>,
    pub max: Box<dyn NumberProviderTrait>,
}

// NOTE: trait objects — используем dyn NumberProvider через alias.
pub use super::number_provider::NumberProvider as NumberProviderTrait;

impl UniformGenerator {
    pub fn new(min: Box<dyn NumberProviderTrait>, max: Box<dyn NumberProviderTrait>) -> Self {
        Self { min, max }
    }
    pub fn between(min: f32, max: f32) -> Self {
        Self::new(
            Box::new(ConstantValue::exactly(min)),
            Box::new(ConstantValue::exactly(max)),
        )
    }
}

/// `BinomialDistributionGenerator`.
pub struct BinomialDistributionGenerator {
    pub n: Box<dyn NumberProviderTrait>,
    pub p: Box<dyn NumberProviderTrait>,
}
impl BinomialDistributionGenerator {
    pub fn new(n: Box<dyn NumberProviderTrait>, p: Box<dyn NumberProviderTrait>) -> Self {
        Self { n, p }
    }
    pub fn binomial(n: i32, p: f32) -> Self {
        Self::new(
            Box::new(ConstantValue::exactly(n as f32)),
            Box::new(ConstantValue::exactly(p)),
        )
    }
}

/// `EnchantmentLevelProvider`.
pub struct EnchantmentLevelProvider {
    pub amount: LevelBasedValue,
}
impl EnchantmentLevelProvider {
    pub fn for_enchantment_level(amount: LevelBasedValue) -> Self {
        Self { amount }
    }
}

/// `ScoreboardValue`.
pub struct ScoreboardValue {
    pub target: ScoreboardNameProvider,
    pub score: String,
    pub scale: f32,
}
impl ScoreboardValue {
    pub fn new(target: ScoreboardNameProvider, score: String, scale: f32) -> Self {
        Self { target, score, scale }
    }
    pub fn from_scoreboard(target: crate::world::level::storage::loot::loot_context::EntityTarget, score: &str) -> Self {
        Self::new(
            ScoreboardNameProvider::Context(ContextScoreboardNameProvider { target }),
            score.to_string(),
            1.0,
        )
    }
    pub fn from_scoreboard_scaled(target: crate::world::level::storage::loot::loot_context::EntityTarget, score: &str, scale: f32) -> Self {
        Self::new(
            ScoreboardNameProvider::Context(ContextScoreboardNameProvider { target }),
            score.to_string(),
            scale,
        )
    }
    pub fn fixed(name: &str, score: &str) -> Self {
        Self::new(
            ScoreboardNameProvider::Fixed(FixedScoreboardNameProvider { name: name.to_string() }),
            score.to_string(),
            1.0,
        )
    }
}

/// `StorageValue`.
pub struct StorageValue {
    pub storage: Identifier,
    pub path: crate::nbt::path::NbtPath,
}
impl StorageValue {
    pub fn new(storage: Identifier, path: crate::nbt::path::NbtPath) -> Self {
        Self { storage, path }
    }
}

/// `NumberProviders` — константы типов (реестр `LOOT_NUMBER_PROVIDER_TYPE`).
pub mod types {
    /// `constant`.
    pub const CONSTANT: &str = "constant";
    /// `uniform`.
    pub const UNIFORM: &str = "uniform";
    /// `binomial`.
    pub const BINOMIAL: &str = "binomial";
    /// `score`.
    pub const SCORE: &str = "score";
    /// `storage`.
    pub const STORAGE: &str = "storage";
    /// `enchantment_level`.
    pub const ENCHANTMENT_LEVEL: &str = "enchantment_level";
}

#[allow(dead_code)]
fn _unused(_: Vec3) {}
