//! `net.minecraft.world.level.levelgen.feature.PlacedFeature` —
//! размещённая фича (фича + placement-модификаторы).

use super::configured_feature::ConfiguredFeature;
use crate::core::holder::Holder;
use crate::world::level::levelgen::placement::placement_modifier::PlacementModifier;
use std::sync::Arc;

/// `PlacedFeature`.
#[derive(Clone)]
pub struct PlacedFeature {
    /// `feature`.
    pub feature: Holder<ConfiguredFeature>,
    /// `placement` — модификаторы размещения.
    pub placement: Vec<Arc<dyn PlacementModifier>>,
}

impl PlacedFeature {
    /// `new PlacedFeature(feature, placement)`.
    pub fn new(feature: Holder<ConfiguredFeature>, placement: Vec<Arc<dyn PlacementModifier>>) -> Self {
        Self { feature, placement }
    }

    /// `getFeatures()` — итератор по configured-фичам (для flowerFeatures).
    pub fn get_features(&self) -> impl Iterator<Item = &ConfiguredFeature> {
        self.feature.value().into_iter()
    }
}
