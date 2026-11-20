//! Перенос `net.minecraft.world.level.chunk.storage` —
//! legacy-исправления тегов чанков.

pub mod legacy_tag_fixer;

pub use legacy_tag_fixer::{EmptyLegacyTagFixer, LegacyTagFixer};
