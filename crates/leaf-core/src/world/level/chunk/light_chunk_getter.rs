//! `net.minecraft.world.level.chunk.LightChunkGetter` — провайдер чанков
//! для движка освещения.

use crate::core::section_pos::SectionPos;
use crate::world::level::chunk::light_chunk::LightChunk;
use crate::world::level::light_layer::LightLayer;

/// `LightChunkGetter`.
pub trait LightChunkGetter {
    /// `getChunkForLighting(chunkX, chunkZ)`.
    fn get_chunk_for_lighting(&self, chunk_x: i32, chunk_z: i32) -> Option<&dyn LightChunk>;
    /// `onLightUpdate(lightLayer, SectionPos)`.
    fn on_light_update(&self, _light_layer: LightLayer, _pos: SectionPos) {}
}
