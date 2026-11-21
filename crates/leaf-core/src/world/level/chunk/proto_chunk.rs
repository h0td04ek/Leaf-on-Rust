//! Перенос `net.minecraft.world.level.chunk.ProtoChunk` — чанк в процессе генерации.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::nbt::compound::CompoundTag;
use crate::world::level::chunk::chunk_access::{BlendingData, ChunkAccess, UpgradeData};
use crate::world::level::chunk::level_chunk::ChunkStatus;
use crate::world::level::chunk::level_chunk_section::LevelChunkSection;
use crate::world::level::chunk::packed_ticks::PackedTicks;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_height_accessor::LevelHeightAccessor;
use crate::world::level::scheduled_tick_access::{BlockTickType, FluidTickType};
use crate::world::ticks::ProtoChunkTicks;

/// `ProtoChunk` — чанк в процессе генерации.
pub struct ProtoChunk {
    /// `chunkAccess` — базовый ChunkAccess.
    pub chunk_access: ChunkAccess,
    /// `chunkStatus` — статус генерации.
    pub chunk_status: ChunkStatus,
    /// `entities` — сущности для сохранения.
    pub entities: Vec<CompoundTag>,
    pub post_processing: Vec<BlockPos>,
    /// `carvingMask` — маска карвинга.
    pub carving_mask: Option<Vec<bool>>,
    /// `belowZeroRetrogen` — retrogen для below-zero.
    pub below_zero_retrogen: Option<BelowZeroRetrogen>,
    /// `blockTicks` — тики блоков.
    pub block_ticks: ProtoChunkTicks<BlockTickType>,
    /// `fluidTicks` — тики жидкостей.
    pub fluid_ticks: ProtoChunkTicks<FluidTickType>,
    /// `lightEngine` — light engine.
    pub light_engine: Option<Arc<dyn crate::world::level::lighting::light_engine::LightEngine>>,
}

/// `BelowZeroRetrogen` — retrogen для below-zero генерации.
#[derive(Clone, Debug)]
pub struct BelowZeroRetrogen {
    /// `belowZero` — флаг below-zero.
    pub below_zero: bool,
}

impl ProtoChunk {
    /// `new(...)` — создаёт новый ProtoChunk.
    pub fn new(
        chunk_pos: ChunkPos,
        upgrade_data: UpgradeData,
        level_height_accessor: Box<dyn LevelHeightAccessor>,
        sections: Vec<LevelChunkSection>,
        blending_data: Option<BlendingData>,
    ) -> Self {
        let mut chunk_access = ChunkAccess::new(chunk_pos, level_height_accessor, sections);
        chunk_access.persisted_status = ChunkStatus::Empty;
        chunk_access.upgrade_data = Some(upgrade_data);
        chunk_access.blending_data = blending_data;
        Self::from_access(chunk_access, PackedTicks::default(), Vec::new(), Vec::new())
    }

    pub(crate) fn from_access(
        chunk_access: ChunkAccess,
        ticks: PackedTicks,
        post_processing: Vec<BlockPos>,
        entities: Vec<CompoundTag>,
    ) -> Self {
        Self {
            chunk_status: chunk_access.persisted_status,
            chunk_access,
            entities,
            post_processing,
            carving_mask: None,
            below_zero_retrogen: None,
            block_ticks: ProtoChunkTicks::load(ticks.blocks),
            fluid_ticks: ProtoChunkTicks::load(ticks.fluids),
            light_engine: None,
        }
    }

    /// `getPos` — позиция чанка.
    pub fn get_pos(&self) -> ChunkPos {
        self.chunk_access.get_pos()
    }

    /// `getChunkStatus` — статус чанка.
    pub fn get_chunk_status(&self) -> ChunkStatus {
        self.chunk_status
    }

    /// `setChunkStatus` — установить статус чанка.
    pub fn set_chunk_status(&mut self, status: ChunkStatus) {
        self.chunk_status = status;
        self.chunk_access.persisted_status = status;
    }

    /// `getSection` — секция.
    pub fn get_section(&self, section: i32) -> Option<&LevelChunkSection> {
        self.chunk_access.get_section(section)
    }

    /// `getSection_mut` — изменяемая секция.
    pub fn get_section_mut(&mut self, section: i32) -> Option<&mut LevelChunkSection> {
        self.chunk_access.get_section_mut(section)
    }

    /// `getBlockState` — состояние блока.
    pub fn get_block_state(
        &self,
        pos: &BlockPos,
    ) -> crate::world::level::block::state::block_state::BlockState {
        self.chunk_access.get_block_state(pos)
    }

    /// `setBlockState` — установить состояние блока.
    pub fn set_block_state(
        &mut self,
        pos: &BlockPos,
        state: crate::world::level::block::state::block_state::BlockState,
    ) -> bool {
        self.chunk_access.set_block_state(pos, state)
    }

    /// `getEntities` — сущности.
    pub fn get_entities(&self) -> &[CompoundTag] {
        &self.entities
    }

    /// `addEntity` — добавить сущность.
    pub fn add_entity(&mut self, tag: CompoundTag) {
        self.entities.push(tag);
    }

    /// `getCarvingMask` — маска карвинга.
    pub fn get_carving_mask(&self) -> Option<&Vec<bool>> {
        self.carving_mask.as_ref()
    }

    /// `setCarvingMask` — установить маску карвинга.
    pub fn set_carving_mask(&mut self, mask: Vec<bool>) {
        self.carving_mask = Some(mask);
    }

    /// `getBelowZeroRetrogen` — below-zero retrogen.
    pub fn get_below_zero_retrogen(&self) -> Option<&BelowZeroRetrogen> {
        self.below_zero_retrogen.as_ref()
    }

    /// `setBelowZeroRetrogen` — установить below-zero retrogen.
    pub fn set_below_zero_retrogen(&mut self, retrogen: BelowZeroRetrogen) {
        self.below_zero_retrogen = Some(retrogen);
    }

    pub fn get_block_ticks(&self) -> &ProtoChunkTicks<BlockTickType> {
        &self.block_ticks
    }

    pub fn get_fluid_ticks(&self) -> &ProtoChunkTicks<FluidTickType> {
        &self.fluid_ticks
    }

    pub fn pack_ticks(&self) -> PackedTicks {
        PackedTicks {
            blocks: self.block_ticks.scheduled_ticks(),
            fluids: self.fluid_ticks.scheduled_ticks(),
        }
    }

    /// `getLightEngine` — light engine.
    pub fn get_light_engine(
        &self,
    ) -> Option<&Arc<dyn crate::world::level::lighting::light_engine::LightEngine>> {
        self.light_engine.as_ref()
    }

    /// `setLightEngine` — установить light engine.
    pub fn set_light_engine(
        &mut self,
        engine: Arc<dyn crate::world::level::lighting::light_engine::LightEngine>,
    ) {
        self.light_engine = Some(engine);
    }

    /// `isEmpty` — пустой ли чанк.
    pub fn is_empty(&self) -> bool {
        self.chunk_access.is_empty()
    }

    /// `getHeight` — высота.
    pub fn get_height(
        &self,
        heightmap: crate::world::level::levelgen::heightmap::HeightmapType,
        x: i32,
        z: i32,
    ) -> i32 {
        self.chunk_access.get_height(heightmap, x, z)
    }

    /// `setHeight` — установить высоту.
    pub fn set_height(
        &mut self,
        heightmap: crate::world::level::levelgen::heightmap::HeightmapType,
        x: i32,
        z: i32,
        height: i32,
    ) {
        self.chunk_access.set_height(heightmap, x, z, height);
    }
}
