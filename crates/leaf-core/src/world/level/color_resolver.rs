//! `net.minecraft.world.level.ColorResolver` —
//! функциональный интерфейс цвета биома.

use crate::world::level::biome::biome::Biome;

/// `ColorResolver` — `getColor(biome, x, z)`.
pub trait ColorResolver: Send + Sync {
    /// `getColor(biome, x, z)` → ARGB-цвет.
    fn get_color(&self, biome: &Biome, x: f64, z: f64) -> i32;
}
