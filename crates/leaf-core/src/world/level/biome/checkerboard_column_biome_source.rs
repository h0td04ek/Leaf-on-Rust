//! `net.minecraft.world.level.biome.CheckerboardColumnBiomeSource`.

use super::biome::Biome;
use super::biome_source::BiomeSource;
use super::climate::Sampler;
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;

/// `CheckerboardColumnBiomeSource` — шахматный источник биомов по колоннам.
pub struct CheckerboardColumnBiomeSource {
    /// `allowedBiomes`.
    pub allowed_biomes: HolderSet<Biome>,
    /// `bitShift`.
    pub bit_shift: i32,
    /// `size`.
    pub size: i32,
}

impl CheckerboardColumnBiomeSource {
    /// `new CheckerboardColumnBiomeSource(allowedBiomes, size)`.
    pub fn new(allowed_biomes: HolderSet<Biome>, size: i32) -> Self {
        Self { allowed_biomes, bit_shift: size + 2, size }
    }
}

impl BiomeSource for CheckerboardColumnBiomeSource {
    fn get_noise_biome(&self, x: i32, _y: i32, z: i32, _sampler: &Sampler) -> Holder<Biome> {
        let size = self.allowed_biomes.size();
        assert!(size > 0, "allowedBiomes пуста");
        let idx = (x >> self.bit_shift).wrapping_add(z >> self.bit_shift).rem_euclid(size as i32) as usize;
        self.allowed_biomes
            .get(idx)
            .cloned()
            .expect("CheckerboardColumnBiomeSource: named HolderSet без разрешённого тега")
    }

    fn possible_biomes(&self) -> &[Holder<Biome>] {
        self.allowed_biomes.as_direct().unwrap_or(&[])
    }
}
