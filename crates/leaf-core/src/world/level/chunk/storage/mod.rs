//! Перенос `net.minecraft.world.level.chunk.storage` —
//! регионные файлы и legacy-исправления тегов чанков.

pub mod legacy_tag_fixer;
pub mod region_file_storage;

pub use legacy_tag_fixer::{EmptyLegacyTagFixer, LegacyTagFixer};
