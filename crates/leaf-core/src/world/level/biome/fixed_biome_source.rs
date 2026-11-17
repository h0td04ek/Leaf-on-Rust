//! `net.minecraft.world.level.biome.FixedBiomeSource`.

use super::biome::Biome;
use super::biome_source::BiomeSource;
use super::climate::Sampler;
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::random::RandomSource;
use crate::util::mth;
use crate::world::level::level_reader::LevelReader;
use std::sync::Arc;

/// `FixedBiomeSource` — источник с одним биомом.
pub struct FixedBiomeSource {
    /// `biome`.
    pub biome: Holder<Biome>,
}

impl FixedBiomeSource {
    /// `new FixedBiomeSource(biome)`.
    pub fn new(biome: Holder<Biome>) -> Self {
        Self { biome }
    }
}

/// Результат `findBiomeHorizontal`/`findClosestBiome3d` — `Pair<BlockPos, Holder>`.
pub type BiomeFindResult = (BlockPos, Holder<Biome>);

impl BiomeSource for FixedBiomeSource {
    fn get_noise_biome(&self, _x: i32, _y: i32, _z: i32, _sampler: &Sampler) -> Holder<Biome> {
        self.biome.clone()
    }

    fn possible_biomes(&self) -> &[Holder<Biome>] {
        std::slice::from_ref(&self.biome)
    }
}

impl FixedBiomeSource {
    /// `getNoiseBiome(x, y, z)` — без sampler.
    pub fn get_noise_biome_no_sampler(&self, _x: i32, _y: i32, _z: i32) -> Holder<Biome> {
        self.biome.clone()
    }

    /// `findBiomeHorizontal` — поиск биома по XZ.
    #[allow(clippy::too_many_arguments)]
    pub fn find_biome_horizontal(
        &self,
        x: i32,
        y: i32,
        z: i32,
        radius: i32,
        increment: i32,
        biome_predicate: &dyn Fn(&Holder<Biome>) -> bool,
        random: &mut dyn RandomSource,
        find_closest: bool,
        _sampler: &Sampler,
    ) -> Option<BiomeFindResult> {
        let _ = increment;
        if biome_predicate(&self.biome) {
            if find_closest {
                Some((BlockPos::new(x, y, z), self.biome.clone()))
            } else {
                Some((
                    BlockPos::new(
                        x - radius + random.next_int_bound(radius * 2 + 1),
                        y,
                        z - radius + random.next_int_bound(radius * 2 + 1),
                    ),
                    self.biome.clone(),
                ))
            }
        } else {
            None
        }
    }

    /// `findClosestBiome3d`.
    #[allow(clippy::too_many_arguments)]
    pub fn find_closest_biome3d(
        &self,
        pos: BlockPos,
        radius: i32,
        horizontal_step: i32,
        vertical_step: i32,
        biome_predicate: &dyn Fn(&Holder<Biome>) -> bool,
        _sampler: &Sampler,
        level: &dyn LevelReader,
    ) -> Option<BiomeFindResult> {
        let _ = (radius, horizontal_step, vertical_step);
        if biome_predicate(&self.biome) {
            let y = mth::clamp_i32(pos.y, level.min_build_height() + 1, level.min_build_height() + level.height() + 1);
            Some((pos.at_y(y), self.biome.clone()))
        } else {
            None
        }
    }

    /// `getBiomesWithin` — множество биомов в радиусе.
    pub fn get_biomes_within(&self, _x: i32, _y: i32, _z: i32, _radius: i32, _sampler: &Sampler) -> Vec<Holder<Biome>> {
        vec![self.biome.clone()]
    }
}
