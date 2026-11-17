//! `net.minecraft.world.level.biome.BiomeResolver`.

use super::biome::Biome;
use super::climate::Sampler;
use crate::core::holder::Holder;

/// `BiomeResolver` — резолвер биома по шуму.
pub trait BiomeResolver: Send + Sync {
    /// `getNoiseBiome(x, y, z, sampler)` — x/y/z в quart-координатах.
    fn get_noise_biome(&self, x: i32, y: i32, z: i32, sampler: &Sampler) -> Holder<Biome>;
}
