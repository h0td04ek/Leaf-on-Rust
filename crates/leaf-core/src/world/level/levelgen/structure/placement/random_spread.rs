//! Перенос `RandomSpreadStructurePlacement` + `RandomSpreadType`.

use super::structure_placement::{
    ExclusionZone, FrequencyReductionMethod, StructurePlacement, StructureStateAccess,
};
use crate::core::vec3i::Vec3i;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::levelgen::structure::structure_set::StructureSet;
use crate::world::level::levelgen::worldgen_random::WorldgenRandom;
use crate::random::legacy::LegacyRandomSource;
use crate::random::RandomSource;
use std::sync::Arc;

/// `RandomSpreadType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RandomSpreadType {
    /// `LINEAR`.
    Linear,
    /// `TRIANGULAR`.
    Triangular,
}

impl RandomSpreadType {
    /// `evaluate`.
    pub fn evaluate(self, random: &mut dyn RandomSource, bound: i32) -> i32 {
        match self {
            RandomSpreadType::Linear => random.next_int_bound(bound),
            RandomSpreadType::Triangular => {
                (random.next_int_bound(bound) + random.next_int_bound(bound)) / 2
            }
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            RandomSpreadType::Linear => "linear",
            RandomSpreadType::Triangular => "triangular",
        }
    }

    /// `byName`.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "linear" => Some(Self::Linear),
            "triangular" => Some(Self::Triangular),
            _ => None,
        }
    }
}

/// `RandomSpreadStructurePlacement`.
#[derive(Clone)]
pub struct RandomSpreadStructurePlacement {
    /// Базовый placement.
    pub base: StructurePlacement,
    /// `spacing` — расстояние между регионами.
    pub spacing: i32,
    /// `separation` — минимальная дистанция между структурами.
    pub separation: i32,
    /// `spreadType`.
    pub spread_type: RandomSpreadType,
}

impl RandomSpreadStructurePlacement {
    /// `new` (полный).
    pub fn new(
        locate_offset: Vec3i,
        frequency_reduction_method: FrequencyReductionMethod,
        frequency: f32,
        salt: i32,
        exclusion_zone: Option<ExclusionZone>,
        spacing: i32,
        separation: i32,
        spread_type: RandomSpreadType,
    ) -> Self {
        Self {
            base: StructurePlacement::new(
                locate_offset,
                frequency_reduction_method,
                frequency,
                salt,
                exclusion_zone,
            ),
            spacing,
            separation,
            spread_type,
        }
    }

    /// `new(spacing, separation, spreadType, salt)` — дефолтный.
    pub fn simple(spacing: i32, separation: i32, spread_type: RandomSpreadType, salt: i32) -> Self {
        Self::new(
            Vec3i::ZERO,
            FrequencyReductionMethod::Default,
            1.0,
            salt,
            None,
            spacing,
            separation,
            spread_type,
        )
    }

    /// `getPotentialStructureChunk(seed, regionX, regionZ)`.
    pub fn get_potential_structure_chunk(
        &self,
        seed: i64,
        region_x: i32,
        region_z: i32,
    ) -> ChunkPos {
        let i = region_x.div_euclid(self.spacing);
        let i1 = region_z.div_euclid(self.spacing);
        let mut random = WorldgenRandom::new(Box::new(LegacyRandomSource::new(0)));
        random.set_large_feature_with_salt(seed, i, i1, self.base.salt);
        let i2 = self.spacing - self.separation;
        let i3 = self.spread_type.evaluate(&mut random, i2);
        let i4 = self.spread_type.evaluate(&mut random, i2);
        ChunkPos::new(i * self.spacing + i3, i1 * self.spacing + i4)
    }

    /// `isPlacementChunk`.
    pub fn is_placement_chunk(
        &self,
        structure_state: &dyn StructureStateAccess,
        x: i32,
        z: i32,
    ) -> bool {
        let potential =
            self.get_potential_structure_chunk(structure_state.level_seed(), x, z);
        potential.x == x && potential.z == z
    }

    /// `isStructureChunk` — с delegation на base.
    pub fn is_structure_chunk(
        &self,
        structure_state: &dyn StructureStateAccess,
        x: i32,
        z: i32,
        structure_set_key: Option<&crate::resources::resource_key::ResourceKey<StructureSet>>,
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
