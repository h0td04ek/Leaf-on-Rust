//! Перенос `net.minecraft.world.level.levelgen.structure.placement.StructurePlacement`
//! — базовый класс размещения структур + FrequencyReductionMethod
//! + ExclusionZone.

use crate::core::vec3i::Vec3i;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::levelgen::structure::structure_set::StructureSet;
use crate::world::level::levelgen::worldgen_random::WorldgenRandom;
use crate::random::legacy::LegacyRandomSource;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use std::sync::Arc;

/// `HIGHLY_ARBITRARY_RANDOM_SALT`.
pub const HIGHLY_ARBITRARY_RANDOM_SALT: i32 = 10387320;

/// `ChunkGeneratorStructureState`-доступ для placement
/// (`getLevelSeed`, `hasStructureChunkInRange`,
/// `getRingPositionsFor`). Трейт — реализация подключится с
/// chunk-generator слоем.
pub trait StructureStateAccess {
    /// `getLevelSeed`.
    fn level_seed(&self) -> i64;
    /// `hasStructureChunkInRange(set, x, z, range)`.
    fn has_structure_chunk_in_range(
        &self,
        set: &Arc<StructureSet>,
        x: i32,
        z: i32,
        range: i32,
    ) -> bool;
    /// `getRingPositionsFor(placement)` — позиции колец
    /// (для ConcentricRings).
    fn get_ring_positions_for(
        &self,
        placement: &ConcentricRingPositions,
    ) -> Option<Vec<ChunkPos>>;
}

/// Дескриптор concentric rings для `getRingPositionsFor`
/// (id placement'а — в Java это ссылка на placement).
pub type ConcentricRingPositions = (i32, i32, i32);

/// `StructurePlacement` — базовое размещение.
#[derive(Clone)]
pub struct StructurePlacement {
    /// `locateOffset`.
    pub locate_offset: Vec3i,
    /// `frequencyReductionMethod`.
    pub frequency_reduction_method: FrequencyReductionMethod,
    /// `frequency`.
    pub frequency: f32,
    /// `salt`.
    pub salt: i32,
    /// `exclusionZone`.
    pub exclusion_zone: Option<ExclusionZone>,
}

impl StructurePlacement {
    /// `new`.
    pub fn new(
        locate_offset: Vec3i,
        frequency_reduction_method: FrequencyReductionMethod,
        frequency: f32,
        salt: i32,
        exclusion_zone: Option<ExclusionZone>,
    ) -> Self {
        Self {
            locate_offset,
            frequency_reduction_method,
            frequency,
            salt,
            exclusion_zone,
        }
    }

    /// `isStructureChunk` — placement chunk + additional
    /// restrictions + interactions.
    pub fn is_structure_chunk(
        &self,
        structure_state: &dyn StructureStateAccess,
        x: i32,
        z: i32,
        structure_set_key: Option<&ResourceKey<StructureSet>>,
    ) -> bool {
        let salt_override = match structure_set_key {
            Some(k) if k.id().path.contains("mineshaft") => Some(0), // config-зависимый override
            _ => None,
        };
        self.is_placement_chunk(structure_state, x, z)
            && self.apply_additional_chunk_restrictions(
                x,
                z,
                structure_state.level_seed(),
                salt_override,
            )
            && self.apply_interactions_with_other_structures(structure_state, x, z)
    }

    /// `applyAdditionalChunkRestrictions`.
    pub fn apply_additional_chunk_restrictions(
        &self,
        region_x: i32,
        region_z: i32,
        level_seed: i64,
        salt_override: Option<i32>,
    ) -> bool {
        self.frequency >= 1.0
            || self.frequency_reduction_method.should_generate(
                level_seed,
                self.salt,
                region_x,
                region_z,
                self.frequency,
                salt_override,
            )
    }

    /// `applyInteractionsWithOtherStructures`.
    pub fn apply_interactions_with_other_structures(
        &self,
        structure_state: &dyn StructureStateAccess,
        x: i32,
        z: i32,
    ) -> bool {
        self.exclusion_zone
            .as_ref()
            .map(|z_| !z_.is_placement_forbidden(structure_state, x, z))
            .unwrap_or(true)
    }

    /// `isPlacementChunk` — проверка чанка (наследник).
    pub fn is_placement_chunk(
        &self,
        structure_state: &dyn StructureStateAccess,
        x: i32,
        z: i32,
    ) -> bool {
        let _ = (structure_state, x, z);
        false
    }

    /// `getLocatePos(chunkPos)`.
    pub fn get_locate_pos(&self, chunk_pos: ChunkPos) -> crate::core::block_pos::BlockPos {
        crate::core::block_pos::BlockPos::new(
            chunk_pos.get_min_block_x(),
            0,
            chunk_pos.get_min_block_z(),
        )
        .offset(self.locate_offset.x, self.locate_offset.y, self.locate_offset.z)
    }

    /// `probabilityReducer` — default frequency check.
    fn probability_reducer(
        level_seed: i64,
        _salt: i32,
        region_x: i32,
        region_z: i32,
        frequency: f32,
        salt_override: Option<i32>,
    ) -> bool {
        let mut random = WorldgenRandom::new(Box::new(LegacyRandomSource::new(0)));
        random.set_large_feature_with_salt(
            level_seed,
            region_x,
            region_z,
            salt_override.unwrap_or(HIGHLY_ARBITRARY_RANDOM_SALT),
        );
        random.next_float() < frequency
    }

    /// `legacyArbitrarySaltProbabilityReducer`.
    fn legacy_arbitrary_salt_probability_reducer(
        level_seed: i64,
        salt: i32,
        region_x: i32,
        region_z: i32,
        frequency: f32,
        salt_override: Option<i32>,
    ) -> bool {
        let mut random = WorldgenRandom::new(Box::new(LegacyRandomSource::new(0)));
        random.set_large_feature_with_salt(
            level_seed,
            region_x,
            region_z,
            salt_override.unwrap_or(salt),
        );
        random.next_float() < frequency
    }

    /// `legacyProbabilityReducerWithDouble`.
    fn legacy_probability_reducer_with_double(
        base_seed: i64,
        salt: i32,
        chunk_x: i32,
        chunk_z: i32,
        frequency: f32,
        salt_override: Option<i32>,
    ) -> bool {
        let mut random = WorldgenRandom::new(Box::new(LegacyRandomSource::new(0)));
        if let Some(override_salt) = salt_override {
            random.set_large_feature_with_salt(base_seed, chunk_x, chunk_z, override_salt);
        } else {
            random.set_large_feature_seed(base_seed, chunk_x, chunk_z);
            let _ = salt;
        }
        random.next_double() < frequency as f64
    }

    /// `legacyPillagerOutpostReducer`.
    fn legacy_pillager_outpost_reducer(
        level_seed: i64,
        _salt: i32,
        region_x: i32,
        region_z: i32,
        frequency: f32,
        _salt_override: Option<i32>,
    ) -> bool {
        let i = region_x >> 4;
        let i1 = region_z >> 4;
        let mut random = WorldgenRandom::new(Box::new(LegacyRandomSource::new(0)));
        random.set_seed(((i as i64) ^ ((i1 as i64) << 4) ^ level_seed) as i64);
        random.next_int();
        random.next_int_bound((1.0 / frequency) as i32) == 0
    }
}

/// `ExclusionZone` — запрет размещения рядом с другим set'ом.
#[derive(Clone)]
pub struct ExclusionZone {
    /// `otherSet`.
    pub other_set: Arc<StructureSet>,
    /// `chunkCount`.
    pub chunk_count: i32,
}

impl ExclusionZone {
    /// `isPlacementForbidden`.
    pub fn is_placement_forbidden(
        &self,
        structure_state: &dyn StructureStateAccess,
        x: i32,
        z: i32,
    ) -> bool {
        structure_state.has_structure_chunk_in_range(&self.other_set, x, z, self.chunk_count)
    }
}

/// `FrequencyReductionMethod`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrequencyReductionMethod {
    /// `DEFAULT`.
    Default,
    /// `LEGACY_TYPE_1` — pillager outpost.
    LegacyType1,
    /// `LEGACY_TYPE_2` — arbitrary salt.
    LegacyType2,
    /// `LEGACY_TYPE_3` — double-based legacy.
    LegacyType3,
}

impl FrequencyReductionMethod {
    /// `shouldGenerate`.
    pub fn should_generate(
        self,
        level_seed: i64,
        salt: i32,
        region_x: i32,
        region_z: i32,
        frequency: f32,
        salt_override: Option<i32>,
    ) -> bool {
        match self {
            FrequencyReductionMethod::Default => StructurePlacement::probability_reducer(
                level_seed, salt, region_x, region_z, frequency, salt_override,
            ),
            FrequencyReductionMethod::LegacyType1 => StructurePlacement::legacy_pillager_outpost_reducer(
                level_seed, salt, region_x, region_z, frequency, salt_override,
            ),
            FrequencyReductionMethod::LegacyType2 => StructurePlacement::legacy_arbitrary_salt_probability_reducer(
                level_seed, salt, region_x, region_z, frequency, salt_override,
            ),
            FrequencyReductionMethod::LegacyType3 => StructurePlacement::legacy_probability_reducer_with_double(
                level_seed, salt, region_x, region_z, frequency, salt_override,
            ),
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            FrequencyReductionMethod::Default => "default",
            FrequencyReductionMethod::LegacyType1 => "legacy_type_1",
            FrequencyReductionMethod::LegacyType2 => "legacy_type_2",
            FrequencyReductionMethod::LegacyType3 => "legacy_type_3",
        }
    }

    /// `byName`.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "default" => Some(Self::Default),
            "legacy_type_1" => Some(Self::LegacyType1),
            "legacy_type_2" => Some(Self::LegacyType2),
            "legacy_type_3" => Some(Self::LegacyType3),
            _ => None,
        }
    }
}
