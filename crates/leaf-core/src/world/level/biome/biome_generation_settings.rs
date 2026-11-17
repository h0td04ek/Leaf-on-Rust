//! `net.minecraft.world.level.biome.BiomeGenerationSettings` —
//! карверы и фичи биома.

use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::world::level::levelgen::carver::configured_world_carver::ConfiguredWorldCarver;
use crate::world::level::levelgen::feature::placed_feature::PlacedFeature;
use crate::world::level::levelgen::generation_step::GenerationStep;
use std::sync::{Arc, OnceLock};

/// `BiomeGenerationSettings.EMPTY`.
pub fn empty() -> BiomeGenerationSettings {
    BiomeGenerationSettings::empty_static().clone()
}

/// `BiomeGenerationSettings`.
#[derive(Clone)]
pub struct BiomeGenerationSettings {
    /// `carvers`.
    pub carvers: HolderSet<ConfiguredWorldCarver>,
    /// `features` — по шагам `GenerationStep.Decoration`.
    pub features: Vec<HolderSet<PlacedFeature>>,
}

impl Default for BiomeGenerationSettings {
    fn default() -> Self {
        empty()
    }
}

impl BiomeGenerationSettings {
    fn empty_static() -> &'static BiomeGenerationSettings {
        static EMPTY: OnceLock<BiomeGenerationSettings> = OnceLock::new();
        EMPTY.get_or_init(|| BiomeGenerationSettings {
            carvers: HolderSet::Direct(Vec::new()),
            features: Vec::new(),
        })
    }

    /// `new BiomeGenerationSettings(carvers, features)`.
    pub fn new(carvers: HolderSet<ConfiguredWorldCarver>, features: Vec<HolderSet<PlacedFeature>>) -> Self {
        Self { carvers, features }
    }

    /// `getCarvers`.
    pub fn get_carvers(&self) -> &HolderSet<ConfiguredWorldCarver> {
        &self.carvers
    }

    /// `features()`.
    pub fn features(&self) -> &[HolderSet<PlacedFeature>] {
        &self.features
    }

    /// `getFlowerFeatures` — memoized список flower-фич.
    /// `Feature.FLOWER` ещё не портирован — возвращает пустой список
    /// (заполняется при порте `Feature`-реестра).
    pub fn get_flower_features(&self) -> Vec<Arc<dyn std::any::Any + Send + Sync>> {
        Vec::new()
    }

    /// `hasFeature(placedFeature)` — identity-сравнение по
    /// `Arc<dyn Feature>` сконфигурированной фичи (PlacedFeature в Java —
    /// identity equals).
    pub fn has_feature(&self, feature: &PlacedFeature) -> bool {
        let target = feature.feature.value().map(|cf| Arc::as_ptr(&cf.feature) as *const () as usize);
        let Some(target) = target else { return false };
        self.features.iter().flat_map(|hs| hs.iter()).any(|h| {
            if let Holder::Direct(pf) = h {
                pf.feature
                    .value()
                    .map(|cf| Arc::as_ptr(&cf.feature) as *const () as usize == target)
                    .unwrap_or(false)
            } else {
                false
            }
        })
    }
}

/// `BiomeGenerationSettings.Builder`.
#[derive(Default)]
pub struct BiomeGenerationSettingsBuilder {
    carvers: Vec<Holder<ConfiguredWorldCarver>>,
    features: Vec<Vec<Holder<PlacedFeature>>>,
}

impl BiomeGenerationSettingsBuilder {
    /// `new Builder()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `addFeature(decoration, feature)` — Holder-версия.
    pub fn add_feature_holder(mut self, decoration: GenerationStep, feature: Holder<PlacedFeature>) -> Self {
        self.add_feature_step(decoration.ordinal() as usize, feature)
    }

    /// `addFeature(step, feature)`.
    pub fn add_feature_step(mut self, step: usize, feature: Holder<PlacedFeature>) -> Self {
        self.add_feature_steps_up_to(step);
        self.features[step].push(feature);
        self
    }

    /// `addCarver(carver)`.
    pub fn add_carver(mut self, carver: Holder<ConfiguredWorldCarver>) -> Self {
        self.carvers.push(carver);
        self
    }

    fn add_feature_steps_up_to(&mut self, step: usize) {
        while self.features.len() <= step {
            self.features.push(Vec::new());
        }
    }

    /// `build`.
    pub fn build(self) -> BiomeGenerationSettings {
        BiomeGenerationSettings::new(
            HolderSet::Direct(self.carvers),
            self.features.into_iter().map(HolderSet::Direct).collect(),
        )
    }
}
