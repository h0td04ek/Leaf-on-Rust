//! `net.minecraft.world.level.levelgen.feature.Feature` —
//! интерфейс фичи генерации.
//!
//! `FeaturePlaceContext` — шов под `WorldGenLevel`/`ChunkGenerator`:
//! конкретная реализация появится с портом генератора чанков;
//! методы добавляются по мере порта контекста.

use crate::core::block_pos::BlockPos;
use std::sync::Arc;

/// `Feature.FeaturePlaceContext` — контекст размещения фичи.
pub trait FeaturePlaceContext: Send {
    /// `origin()` — origin-позиция.
    fn origin(&self) -> BlockPos;
    /// `level()` — уровень генерации (шов до `WorldGenLevel`).
    fn level(&self) -> &dyn FeatureLevelAccess;
    /// `chunkGenerator()` — генератор (шов).
    fn chunk_generator(&self) -> &dyn FeatureGeneratorAccess;
    /// `random()` — источник случайности.
    fn random(&self) -> &mut dyn crate::random::RandomSource;
}

/// Доступ к уровню из FeaturePlaceContext (шов).
pub trait FeatureLevelAccess: Send + Sync {}

/// Доступ к генератору из FeaturePlaceContext (шов).
pub trait FeatureGeneratorAccess: Send + Sync {}

/// `Feature` — фича генерации мира.
pub trait Feature: Send + Sync {
    /// `place(context)` — разместить фичу; true при успехе.
    fn place(&self, context: &mut dyn FeaturePlaceContext) -> bool;
}

/// `ConfiguredFeature<F, C>` — фича + конфиг (C стёрт, как дженерик в Java).
#[derive(Clone)]
pub struct ConfiguredFeature {
    /// `feature`.
    pub feature: Arc<dyn Feature>,
    /// `config` — типизированный конфиг фичи.
    pub config: Arc<dyn std::any::Any + Send + Sync>,
}

impl ConfiguredFeature {
    /// `new ConfiguredFeature(feature, config)`.
    pub fn new(feature: Arc<dyn Feature>, config: Arc<dyn std::any::Any + Send + Sync>) -> Self {
        Self { feature, config }
    }
}
