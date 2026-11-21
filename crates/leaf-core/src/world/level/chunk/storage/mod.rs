//! Перенос `net.minecraft.world.level.chunk.storage` —
//! регионные файлы и legacy-исправления тегов чанков.

pub mod legacy_tag_fixer;
pub mod region_file_storage;
pub mod serializable_chunk_data;

pub use legacy_tag_fixer::{EmptyLegacyTagFixer, LegacyTagFixer};
pub mod registry_chunk_palettes;
