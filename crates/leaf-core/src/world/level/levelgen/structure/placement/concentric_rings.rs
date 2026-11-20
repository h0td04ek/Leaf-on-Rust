//! Перенос `ConcentricRingsStructurePlacement` — кольцевое размещение
//! (stronghold).

use super::structure_placement::{
    ExclusionZone, FrequencyReductionMethod, StructurePlacement, StructureStateAccess,
};
use crate::core::vec3i::Vec3i;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::levelgen::structure::structure_set::StructureSet;
use crate::world::level::biome::biome::Biome;
use crate::core::holder_set::HolderSet;
use std::sync::Arc;

/// `ConcentricRingsStructurePlacement`.
#[derive(Clone)]
pub struct ConcentricRingsStructurePlacement {
    /// Базовый placement.
    pub base: StructurePlacement,
    /// `distance` — расстояние от центра.
    pub distance: i32,
    /// `spread` — разброс по кольцу.
    pub spread: i32,
    /// `count` — количество колец/структур.
    pub count: i32,
    /// `preferredBiomes`.
    pub preferred_biomes: Arc<HolderSet<Biome>>,
}

impl ConcentricRingsStructurePlacement {
    /// `new` (полный).
    pub fn new(
        locate_offset: Vec3i,
        frequency_reduction_method: FrequencyReductionMethod,
        frequency: f32,
        salt: i32,
        exclusion_zone: Option<ExclusionZone>,
        distance: i32,
        spread: i32,
        count: i32,
        preferred_biomes: Arc<HolderSet<Biome>>,
    ) -> Self {
        Self {
            base: StructurePlacement::new(
                locate_offset,
                frequency_reduction_method,
                frequency,
                salt,
                exclusion_zone,
            ),
            distance,
            spread,
            count,
            preferred_biomes,
        }
    }

    /// `new(distance, spread, count, preferredBiomes)` — дефолтный.
    pub fn simple(
        distance: i32,
        spread: i32,
        count: i32,
        preferred_biomes: Arc<HolderSet<Biome>>,
    ) -> Self {
        Self::new(
            Vec3i::ZERO,
            FrequencyReductionMethod::Default,
            1.0,
            0,
            None,
            distance,
            spread,
            count,
            preferred_biomes,
        )
    }

    /// `isPlacementChunk` — проверка через ring positions.
    pub fn is_placement_chunk(
        &self,
        structure_state: &dyn StructureStateAccess,
        x: i32,
        z: i32,
    ) -> bool {
        let rings = structure_state.get_ring_positions_for(&(
            self.distance,
            self.spread,
            self.count,
        ));
        rings
            .map(|positions| positions.iter().any(|p| p.x == x && p.z == z))
            .unwrap_or(false)
    }

    /// `isStructureChunk`.
    pub fn is_structure_chunk(
        &self,
        structure_state: &dyn StructureStateAccess,
        x: i32,
        z: i32,
        _structure_set_key: Option<&crate::resources::resource_key::ResourceKey<StructureSet>>,
    ) -> bool {
        self.is_placement_chunk(structure_state, x, z)
            && self
                .base
                .apply_additional_chunk_restrictions(x, z, structure_state.level_seed(), None)
            && self
                .base
                .apply_interactions_with_other_structures(structure_state, x, z)
    }

    /// `getLocatePos`.
    pub fn get_locate_pos(&self, chunk_pos: ChunkPos) -> crate::core::block_pos::BlockPos {
        self.base.get_locate_pos(chunk_pos)
    }
}
