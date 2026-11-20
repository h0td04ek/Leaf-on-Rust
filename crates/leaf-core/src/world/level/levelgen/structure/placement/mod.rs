//! `net.minecraft.world.level.levelgen.structure.placement` —
//! размещение структур (StructurePlacement/RandomSpread/ConcentricRings).

pub mod concentric_rings;
pub mod random_spread;
pub mod structure_placement;

pub use concentric_rings::ConcentricRingsStructurePlacement;
pub use random_spread::{RandomSpreadStructurePlacement, RandomSpreadType};
pub use structure_placement::{
    ExclusionZone, FrequencyReductionMethod, StructurePlacement, StructureStateAccess,
    HIGHLY_ARBITRARY_RANDOM_SALT,
};
