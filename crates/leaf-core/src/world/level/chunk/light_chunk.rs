//! `net.minecraft.world.level.chunk.LightChunk` — чанк, доступный движку
//! освещения: источники block light и sky light sources.

use std::collections::HashMap;

use crate::core::block_pos::BlockPos;
use crate::world::level::lighting::chunk_sky_light_sources::ChunkSkyLightSources;
use crate::world::level::block::state::block_state::BlockState;

/// `LightChunk`.
pub trait LightChunk {
    /// `findBlockLightSources(BiConsumer<BlockPos, BlockState>)`.
    fn find_block_light_sources(&self, output: &mut dyn FnMut(BlockPos, &BlockState));
    /// `getSkyLightSources` — `None` (в Paper/Leaf возвращает null,
    /// StarLight вычисляет источники самостоятельно).
    fn get_sky_light_sources(&self) -> Option<&ChunkSkyLightSources>;
    /// `getBlockState(pos)` (из `BlockGetter`).
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}

/// Вспомогательный тип для `find_block_light_sources`.
pub type BlockLightSources = HashMap<BlockPos, BlockState>;
