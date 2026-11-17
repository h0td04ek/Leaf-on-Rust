//! `net.minecraft.world.level.biome.BiomeSource` — источник биомов.
//!
//! Java — abstract class с `getNoiseBiome(x,y,z,sampler)`,
//! `possibleBiomes()`, `collectPossibleBiomes`. Здесь — trait с теми же
//! методами; конкретные источники — `FixedBiomeSource`,
//! `CheckerboardColumnBiomeSource`, `MultiNoiseBiomeSource`,
//! `TheEndBiomeSource` (портируются в `multi_noise_biome_source.rs` и др.).

use super::biome::Biome;
use super::climate::{ParameterPoint, Sampler};
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use std::sync::Arc;

/// `BiomeSource`.
pub trait BiomeSource: Send + Sync {
    /// `getNoiseBiome(x, y, z, sampler)`.
    fn get_noise_biome(&self, x: i32, y: i32, z: i32, sampler: &Sampler) -> Holder<Biome>;

    /// `possibleBiomes()` — возможные биомы источника.
    fn possible_biomes(&self) -> &[Holder<Biome>] {
        &[]
    }

    /// `collectPossibleBiomes()` — HolderSet возможных биомов.
    fn collect_possible_biomes(&self) -> HolderSet<Biome> {
        HolderSet::Direct(self.possible_biomes().to_vec())
    }

    /// `collectMissingPossibleBiomes` — no-op базово (переопределяется).
    fn collect_missing_possible_biomes(&self, _collector: &mut dyn FnMut(Arc<Biome>)) {}

    /// `climateParameterPoints` — для MultiNoise-источников.
    fn climate_parameter_points(&self) -> Option<&[ParameterPoint]> {
        None
    }
}
