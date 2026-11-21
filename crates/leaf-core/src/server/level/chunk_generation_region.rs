//! Ограниченный доступ к соседям и изменениям одной стадии генерации чанка.

use std::io;

use indexmap::IndexMap;

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::nbt::CompoundTag;
use crate::world::level::biome::biome::Biome;
use crate::world::level::block::state::BlockState;
use crate::world::level::chunk::chunk_pyramid::ChunkStep;
use crate::world::level::chunk::storage::serializable_chunk_data::LoadedChunk;
use crate::world::level::chunk::{level_chunk::ChunkStatus, RuntimeChunk};
use crate::world::level::chunk_pos::ChunkPos;

use super::server_chunk_cache::ServerChunkCache;

pub trait ChunkStageExecutor {
    /// Выполняет одну стадию генератора. Изменения региона применяются при успехе.
    fn execute(&mut self, region: &mut ChunkGenerationRegion<'_>) -> io::Result<()>;
}

pub struct ChunkGenerationRegion<'a> {
    cache: &'a ServerChunkCache,
    pub step: &'a ChunkStep,
    pub center: ChunkPos,
    edits: GenerationEdits,
}

#[derive(Default)]
pub(super) struct GenerationEdits {
    pub blocks: IndexMap<BlockPos, BlockState>,
    pub biomes: IndexMap<BlockPos, Holder<Biome>>,
    pub entities: Vec<CompoundTag>,
}

impl<'a> ChunkGenerationRegion<'a> {
    pub(super) fn new(cache: &'a ServerChunkCache, step: &'a ChunkStep, center: ChunkPos) -> Self {
        Self {
            cache,
            step,
            center,
            edits: GenerationEdits::default(),
        }
    }

    pub(super) fn into_edits(self) -> GenerationEdits {
        self.edits
    }

    pub fn chunk(&self, pos: ChunkPos) -> Option<&'a LoadedChunk> {
        let minimum = self
            .step
            .direct_dependencies
            .at_radius(self.distance(pos) as usize)?;
        let ready = self.cache.completed_status(pos)?;
        if ready.is_or_after(minimum) {
            self.cache.get_loaded(pos)
        } else {
            None
        }
    }

    pub fn block_state(&self, pos: BlockPos) -> Option<BlockState> {
        let chunk = self.chunk(chunk_at(pos))?.chunk.access();
        Some(
            self.edits
                .blocks
                .get(&pos)
                .cloned()
                .unwrap_or_else(|| chunk.get_block_state(&pos)),
        )
    }

    pub fn set_block_state(&mut self, pos: BlockPos, state: BlockState) -> io::Result<()> {
        let chunk_pos = chunk_at(pos);
        let write_radius = self.step.block_state_write_radius;
        if write_radius < 0 || self.distance(chunk_pos) > write_radius as u32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "block is outside the stage write radius",
            ));
        }
        let chunk = self.chunk(chunk_pos).ok_or_else(missing_dependency)?;
        if chunk.chunk.access().get_section(pos.get_y() >> 4).is_none() {
            return Err(outside_height());
        }
        self.edits.blocks.insert(pos, state);
        Ok(())
    }

    pub fn set_biome(&mut self, pos: BlockPos, biome: Holder<Biome>) -> io::Result<()> {
        if chunk_at(pos) != self.center || self.step.target_status != ChunkStatus::Biomes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "biomes may be written only in the center during the biome stage",
            ));
        }
        let chunk = self.chunk(self.center).ok_or_else(missing_dependency)?;
        if chunk.chunk.access().get_section(pos.get_y() >> 4).is_none() {
            return Err(outside_height());
        }
        let quart_origin = BlockPos::new(pos.get_x() & !3, pos.get_y() & !3, pos.get_z() & !3);
        self.edits.biomes.insert(quart_origin, biome);
        Ok(())
    }

    /// Сохраняет моба генерации в ProtoChunk для последующей загрузки.
    pub fn add_entity(&mut self, tag: CompoundTag) -> io::Result<()> {
        let chunk = self.chunk(self.center).ok_or_else(missing_dependency)?;
        if !matches!(chunk.chunk, RuntimeChunk::Proto(_)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected a proto chunk for generation entities",
            ));
        }
        self.edits.entities.push(tag);
        Ok(())
    }

    fn distance(&self, pos: ChunkPos) -> u32 {
        self.center
            .x
            .abs_diff(pos.x)
            .max(self.center.z.abs_diff(pos.z))
    }
}

fn chunk_at(pos: BlockPos) -> ChunkPos {
    ChunkPos::new(pos.get_x() >> 4, pos.get_z() >> 4)
}

fn missing_dependency() -> io::Error {
    io::Error::other("chunk dependency has not reached the required stage")
}

fn outside_height() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "position is outside build height",
    )
}
