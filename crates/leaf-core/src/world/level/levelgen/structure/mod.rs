//! Перенос `net.minecraft.world.level.levelgen.structure` — структуры.

pub mod bounding_box;
pub mod pieces_container;
pub mod scattered_feature_piece;
pub mod single_piece_structure;
pub mod structure;
pub mod structure_check;
pub mod structure_check_result;
pub mod structure_manager;
pub mod structure_piece;
pub mod structure_spawn_override;
pub mod structure_start;
pub mod structure_type;
pub mod terrain_adjustment;

pub use bounding_box::BoundingBox;
pub use pieces_container::PiecesContainer;
pub use scattered_feature_piece::ScatteredFeaturePieceData;
pub use single_piece_structure::single_piece_structure;
pub use structure::{
    all_structures, get_lowest_y, get_lowest_y_explicit, get_lowest_y_in_5by5_box_offset_7_blocks,
    get_mean_first_occupied_height, on_top_of_chunk_center, register_structure,
    structure_by_identifier, structure_registry, GenerationContext, GenerationStub, Structure,
    StructureSettings, StructureSettingsBuilder,
};
pub use structure_check::StructureCheck;
pub use structure_check_result::StructureCheckResult;
pub use structure_manager::{StructureManager, StructureManagerAccess};
pub use structure_piece::StructurePiece;
pub use structure_spawn_override::StructureSpawnOverride;
pub use structure_start::StructureStart;
pub use structure_type::{register_structure_type, structure_type_by_id, StructureType};
pub use terrain_adjustment::TerrainAdjustment;
