//! Перенос `net.minecraft.world.level.levelgen.structure.StructureSet`.

use crate::core::holder::Holder;
use crate::world::level::levelgen::structure::placement::{
    ConcentricRingsStructurePlacement, RandomSpreadStructurePlacement,
    RandomSpreadType, StructurePlacement, StructureStateAccess,
};
use crate::world::level::levelgen::structure::structure::Structure;

/// `Placement` — типизированный placement (`StructurePlacement.CODEC`
/// dispatch по типу).
#[derive(Clone)]
pub enum Placement {
    /// `random_spread`.
    RandomSpread(RandomSpreadStructurePlacement),
    /// `concentric_rings`.
    ConcentricRings(ConcentricRingsStructurePlacement),
}

impl Placement {
    /// `isStructureChunk` — dispatch.
    pub fn is_structure_chunk(
        &self,
        structure_state: &dyn StructureStateAccess,
        x: i32,
        z: i32,
        structure_set_key: Option<&crate::resources::resource_key::ResourceKey<StructureSet>>,
    ) -> bool {
        match self {
            Placement::RandomSpread(p) => {
                p.is_structure_chunk(structure_state, x, z, structure_set_key)
            }
            Placement::ConcentricRings(p) => {
                p.is_structure_chunk(structure_state, x, z, structure_set_key)
            }
        }
    }

    /// `getLocatePos` — dispatch.
    pub fn get_locate_pos(&self, chunk_pos: crate::world::level::chunk_pos::ChunkPos) -> crate::core::block_pos::BlockPos {
        match self {
            Placement::RandomSpread(p) => p.get_locate_pos(chunk_pos),
            Placement::ConcentricRings(p) => p.get_locate_pos(chunk_pos),
        }
    }

    /// `type()` — сериализованное имя типа.
    pub fn type_name(&self) -> &'static str {
        match self {
            Placement::RandomSpread(_) => "random_spread",
            Placement::ConcentricRings(_) => "concentric_rings",
        }
    }

    /// `frequency()` — частота.
    pub fn frequency(&self) -> f32 {
        match self {
            Placement::RandomSpread(p) => p.base.frequency,
            Placement::ConcentricRings(p) => p.base.frequency,
        }
    }

    /// `locateOffset()`.
    pub fn locate_offset(&self) -> crate::core::vec3i::Vec3i {
        match self {
            Placement::RandomSpread(p) => p.base.locate_offset,
            Placement::ConcentricRings(p) => p.base.locate_offset,
        }
    }
}

/// `StructureSelectionEntry` — структура + вес.
#[derive(Clone)]
pub struct StructureSelectionEntry {
    /// `structure` — holder на `Structure`.
    pub structure: Holder<Structure>,
    /// `weight`.
    pub weight: i32,
}

/// `StructureSet` — набор структур с общим placement.
#[derive(Clone)]
pub struct StructureSet {
    /// `structures`.
    pub structures: Vec<StructureSelectionEntry>,
    /// `placement`.
    pub placement: Placement,
}

impl StructureSet {
    /// `new(structures, placement)`.
    pub fn new(structures: Vec<StructureSelectionEntry>, placement: Placement) -> Self {
        Self {
            structures,
            placement,
        }
    }

    /// `new(structure, placement)` — одиночная структура с weight 1.
    pub fn single(
        structure: Holder<Structure>,
        placement: Placement,
    ) -> Self {
        Self::new(
            vec![StructureSelectionEntry {
                structure,
                weight: 1,
            }],
            placement,
        )
    }

    /// `entry(structure, weight)`.
    pub fn entry(structure: Holder<Structure>, weight: i32) -> StructureSelectionEntry {
        StructureSelectionEntry { structure, weight }
    }

    /// `entry(structure)` — weight 1.
    pub fn entry_single(structure: Holder<Structure>) -> StructureSelectionEntry {
        Self::entry(structure, 1)
    }
}
