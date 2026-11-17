//! `net.minecraft.world.level.levelgen.feature` — фичи генерации.
pub mod configured_feature;
pub mod feature;
pub mod placed_feature;

pub use configured_feature::ConfiguredFeature;
pub use feature::{Feature, FeaturePlaceContext};
pub use placed_feature::PlacedFeature;
