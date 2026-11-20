//! `net.minecraft.world.level.chunk.LightChunkGetter` — провайдер чанков
//! для движка освещения.

use crate::core::section_pos::SectionPos;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::light_layer::LightLayer;

/// `LightChunkGetter`.
pub trait LightChunkGetter: Send + Sync {
    /// `getChunkForLighting(chunkX, chunkZ)`.
    /// В Java возвращает `LightChunk` (все реализации —
    /// `ChunkAccess`-наследники), здесь сразу `ChunkAccess`.
    fn get_chunk_for_lighting(&self, chunk_x: i32, chunk_z: i32) -> Option<&ChunkAccess>;
    /// `onLightUpdate(lightLayer, SectionPos)`.
    fn on_light_update(&self, _light_layer: LightLayer, _pos: SectionPos) {}
    /// `getLevel()` — уровень, которому принадлежит getter
    /// (в Java `BlockGetter`; здесь `LevelReader` для доступа к
    /// `LevelHeightAccessor`-данным, которые нужны Starlight).
    fn get_level(&self) -> Option<&dyn LevelReader> {
        None
    }
}
