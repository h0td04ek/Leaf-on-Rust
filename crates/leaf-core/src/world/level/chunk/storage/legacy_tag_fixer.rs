//! Перенос `net.minecraft.world.level.chunk.storage.LegacyTagFixer`.

use crate::world::level::chunk_pos::ChunkPos;
use crate::nbt::compound::CompoundTag;

/// `LegacyTagFixer` — интерфейс исправления legacy-чанков при загрузке.
///
/// В Java методы не `synchronized` (кроме реализации); здесь —
/// `&mut self`, что даёт ту же потокобезопасность на уровне типов.
pub trait LegacyTagFixer {
    /// `applyFix(tag)`.
    fn apply_fix(&mut self, tag: CompoundTag) -> CompoundTag;
    /// `markChunkDone(chunkPos)` — чанк финально загружен.
    fn mark_chunk_done(&mut self, _chunk_pos: ChunkPos) {}
    /// `targetDataVersion()` — -1 = без привязки к версии.
    fn target_data_version(&self) -> i32 {
        -1
    }
}

/// `LegacyTagFixer.EMPTY` — `() -> tag -> tag`.
#[derive(Debug, Default, Clone, Copy)]
pub struct EmptyLegacyTagFixer;

impl LegacyTagFixer for EmptyLegacyTagFixer {
    fn apply_fix(&mut self, tag: CompoundTag) -> CompoundTag {
        tag
    }
}
