//! `ca.spottedleaf.moonrise.patches.starlight.light.StarLightLightingProvider` —
//! интерфейс, через который `Level`/`ChunkMap` получают доступ к
//! `StarLightInterface` и клиентским/серверным операциям со светом.

use std::collections::HashSet;
use std::sync::Arc;

use crate::core::section_pos::SectionPos;
use crate::world::level::chunk::data_layer::DataLayer;
use crate::world::level::chunk::level_chunk::LevelChunk;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::light_layer::LightLayer;

use super::star_light_interface::StarLightInterface;

/// `StarLightLightingProvider`.
pub trait StarLightLightingProvider {
    /// `starlight$getLightEngine()`.
    fn starlight_get_light_engine(&self) -> Arc<StarLightInterface>;

    /// `starlight$clientUpdateLight(lightType, pos, nibble, trustEdges)`.
    fn starlight_client_update_light(
        &self,
        light_type: LightLayer,
        pos: SectionPos,
        nibble: &DataLayer,
        trust_edges: bool,
    );

    /// `starlight$clientRemoveLightData(chunkPos)`.
    fn starlight_client_remove_light_data(&self, chunk_pos: ChunkPos);

    /// `starlight$clientChunkLoad(pos, chunk)`.
    fn starlight_client_chunk_load(&self, pos: ChunkPos, chunk: &LevelChunk);

    /// `starlight$serverRelightChunks(chunks, chunkLightCallback, onComplete)`.
    /// По умолчанию бросает `UnsupportedOperationException` — переопределяется
    /// серверными реализациями.
    fn starlight_server_relight_chunks(
        &self,
        _chunks: &HashSet<ChunkPos>,
        _chunk_light_callback: Option<&mut dyn FnMut(ChunkPos)>,
        _on_complete: Option<&mut dyn FnMut(i32)>,
    ) -> Result<(), String> {
        Err("UnsupportedOperationException".to_string())
    }
}
