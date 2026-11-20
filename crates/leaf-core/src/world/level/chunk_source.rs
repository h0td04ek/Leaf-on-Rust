//! `net.minecraft.world.level.chunk.ChunkSource` — источник чанков.

use std::sync::Arc;

use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::chunk_pos::ChunkPos;

/// `ChunkSource` — источник чанков.
pub trait ChunkSource: Send + Sync {
    /// `getChunk(x, z)` — получить чанк.
    fn get_chunk(&self, x: i32, z: i32) -> Option<Arc<ChunkAccess>>;

    /// `getChunkNow` — получить чанк синхронно.
    fn get_chunk_now(&self, x: i32, z: i32) -> Option<Arc<ChunkAccess>> {
        self.get_chunk(x, z)
    }

    /// `getMinSectionY` — минимальная Y секции.
    fn get_min_section_y(&self) -> i32;

    /// `getSectionsCount` — количество секций.
    fn get_sections_count(&self) -> i32;

    /// `hasChunk(x, z)`.
    fn has_chunk(&self, x: i32, z: i32) -> bool {
        self.get_chunk_now(x, z).is_some()
    }

    /// `getLightEngine` — light engine.
    fn get_light_engine(&self) -> Option<Arc<dyn crate::world::level::lighting::light_engine::LightEngine>> {
        None
    }
}
