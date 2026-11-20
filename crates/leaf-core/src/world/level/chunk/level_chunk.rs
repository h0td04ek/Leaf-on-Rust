//! Перенос `net.minecraft.world.level.chunk.LevelChunk` — полный чанк с секциями.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::section_pos::SectionPos;
use crate::nbt::compound::CompoundTag;
use crate::world::level::block::entity::block_entity::BlockEntityAccess;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::chunk::chunk_access::{ChunkAccess, UpgradeData, BlendingData};
use crate::world::level::chunk::level_chunk_section::LevelChunkSection;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_height_accessor::LevelHeightAccessor;
use crate::world::level::levelgen::heightmap::{Heightmap, HeightmapType};

/// `LevelChunk` — полный чанк с загруженными секциями.
pub struct LevelChunk {
    /// `chunkAccess` — базовый ChunkAccess.
    pub chunk_access: ChunkAccess,
    /// `level` — ссылка на уровень (упрощённо).
    pub level: Option<Arc<dyn crate::world::level::level_reader::LevelReader>>,
    /// `lightEngine` — light engine.
    pub light_engine: Option<Arc<dyn crate::world::level::lighting::light_engine::LightEngine>>,
    /// `chunkStatus` — статус чанка.
    pub chunk_status: ChunkStatus,
    /// `isUnsaved` — флаг несохранённых изменений.
    pub is_unsaved: bool,
    /// `postProcessing` — постобработка.
    pub post_processing: Vec<BlockPos>,
    /// `blockTicks` — тики блоков.
    pub block_ticks: HashMap<BlockPos, i64>,
    /// `fluidTicks` — тики жидкостей.
    pub fluid_ticks: HashMap<BlockPos, i64>,
}

/// `ChunkStatus` — статус генерации чанка.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkStatus {
    /// `EMPTY`.
    Empty,
    /// `STRUCTURE_STARTS`.
    StructureStarts,
    /// `STRUCTURE_REFERENCES`.
    StructureReferences,
    /// `BIOMES`.
    Biomes,
    /// `NOISE`.
    Noise,
    /// `SURFACE`.
    Surface,
    /// `CARVERS`.
    Carvers,
    /// `FEATURES`.
    Features,
    /// `INITIALIZE_LIGHT`.
    InitializeLight,
    /// `LIGHT`.
    Light,
    /// `SPAWN`.
    Spawn,
    /// `FULL`.
    Full,
}

impl ChunkStatus {
    /// `values()`.
    pub const VALUES: [ChunkStatus; 12] = [
        ChunkStatus::Empty,
        ChunkStatus::StructureStarts,
        ChunkStatus::StructureReferences,
        ChunkStatus::Biomes,
        ChunkStatus::Noise,
        ChunkStatus::Surface,
        ChunkStatus::Carvers,
        ChunkStatus::Features,
        ChunkStatus::InitializeLight,
        ChunkStatus::Light,
        ChunkStatus::Spawn,
        ChunkStatus::Full,
    ];

    /// `isOrAfter`.
    pub fn is_or_after(&self, other: ChunkStatus) -> bool {
        self.ordinal() >= other.ordinal()
    }

    /// `ordinal`.
    pub fn ordinal(&self) -> usize {
        Self::VALUES.iter().position(|s| s == self).unwrap_or(0)
    }

}

/// Глобальные write-radius'ы статусов (в Java — поле на enum-константе;
/// у нас enum без состояния, поэтому таблица + Mutex).
static WRITE_RADII: std::sync::Mutex<Option<[i32; 12]>> = std::sync::Mutex::new(None);

impl ChunkStatus {
    /// `moonrise$getWriteRadius()` статическая таблица.
    /// Индекс = ordinal. По умолчанию `-1` (как в Java static init).
    pub fn write_radius(self) -> i32 {
        let mut guard = WRITE_RADII.lock().unwrap();
        if guard.is_none() {
            *guard = Some([-1; 12]);
        }
        guard.unwrap()[self.ordinal()]
    }

    /// `moonrise$setWriteRadius(value)` статическая таблица.
    pub fn set_write_radius(self, value: i32) {
        let mut guard = WRITE_RADII.lock().unwrap();
        if guard.is_none() {
            *guard = Some([-1; 12]);
        }
        guard.as_mut().unwrap()[self.ordinal()] = value;
    }
}

impl LevelChunk {
    /// `new(...)` — создаёт новый LevelChunk.
    pub fn new(
        chunk_pos: ChunkPos,
        level_height_accessor: Box<dyn LevelHeightAccessor>,
        sections: Vec<LevelChunkSection>,
        level: Option<Arc<dyn crate::world::level::level_reader::LevelReader>>,
    ) -> Self {
        let chunk_access = ChunkAccess::new(chunk_pos, level_height_accessor, sections);
        Self {
            chunk_access,
            level,
            light_engine: None,
            chunk_status: ChunkStatus::Full,
            is_unsaved: false,
            post_processing: Vec::new(),
            block_ticks: HashMap::new(),
            fluid_ticks: HashMap::new(),
        }
    }

    /// `getPos` — позиция чанка.
    pub fn get_pos(&self) -> ChunkPos {
        self.chunk_access.get_pos()
    }

    /// `getBlockState` — состояние блока.
    pub fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        self.chunk_access.get_block_state(pos)
    }

    /// `setBlockState` — установить состояние блока.
    pub fn set_block_state(&mut self, pos: &BlockPos, state: BlockState) -> bool {
        self.chunk_access.set_block_state(pos, state)
    }

    /// `getFluidState` — состояние жидкости.
    pub fn get_fluid_state(&self, pos: &BlockPos) -> FluidState {
        self.chunk_access.get_fluid_state(pos)
    }

    /// `getSection` — секция.
    pub fn get_section(&self, section: i32) -> Option<&LevelChunkSection> {
        self.chunk_access.get_section(section)
    }

    /// `getSection_mut` — изменяемая секция.
    pub fn get_section_mut(&mut self, section: i32) -> Option<&mut LevelChunkSection> {
        self.chunk_access.get_section_mut(section)
    }

    /// `getBlockEntities` — block entities.
    pub fn get_block_entities(&self) -> &HashMap<BlockPos, Arc<dyn BlockEntityAccess>> {
        &self.chunk_access.block_entities
    }

    /// `setBlockEntity` — установить block entity.
    pub fn set_block_entity(&mut self, pos: BlockPos, entity: Arc<dyn BlockEntityAccess>) {
        self.chunk_access.set_block_entity(pos, entity);
    }

    /// `removeBlockEntity` — удалить block entity.
    pub fn remove_block_entity(&mut self, pos: &BlockPos) -> Option<Arc<dyn BlockEntityAccess>> {
        self.chunk_access.remove_block_entity(pos)
    }

    /// `getHeightmaps` — heightmaps.
    pub fn get_heightmaps(&self) -> &HashMap<HeightmapType, Heightmap> {
        &self.chunk_access.heightmaps
    }

    /// `getHeight` — высота.
    pub fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32 {
        self.chunk_access.get_height(heightmap, x, z)
    }

    /// `setHeight` — установить высоту.
    pub fn set_height(&mut self, heightmap: HeightmapType, x: i32, z: i32, height: i32) {
        self.chunk_access.set_height(heightmap, x, z, height);
    }

    /// `isEmpty` — пустой ли чанк.
    pub fn is_empty(&self) -> bool {
        self.chunk_access.is_empty()
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

    /// `isUnsaved` — несохранённый ли чанк.
    pub fn is_unsaved(&self) -> bool {
        self.is_unsaved
    }

    /// `setUnsaved` — установить флаг несохранённости.
    pub fn set_unsaved(&mut self, unsaved: bool) {
        self.is_unsaved = unsaved;
        self.chunk_access.set_unsaved(unsaved);
    }

    /// `getPostProcessing` — постобработка.
    pub fn get_post_processing(&self) -> &[BlockPos] {
        &self.post_processing
    }

    /// `addPostProcessing` — добавить постобработку.
    pub fn add_post_processing(&mut self, pos: BlockPos) {
        self.post_processing.push(pos);
    }

    /// `clearPostProcessing` — очистить постобработку.
    pub fn clear_post_processing(&mut self) {
        self.post_processing.clear();
    }

    /// `getBlockTicks` — тики блоков.
    pub fn get_block_ticks(&self) -> &HashMap<BlockPos, i64> {
        &self.block_ticks
    }

    /// `setBlockTick` — установить тик блока.
    pub fn set_block_tick(&mut self, pos: BlockPos, time: i64) {
        self.block_ticks.insert(pos, time);
    }

    /// `getFluidTicks` — тики жидкостей.
    pub fn get_fluid_ticks(&self) -> &HashMap<BlockPos, i64> {
        &self.fluid_ticks
    }

    /// `setFluidTick` — установить тик жидкости.
    pub fn set_fluid_tick(&mut self, pos: BlockPos, time: i64) {
        self.fluid_ticks.insert(pos, time);
    }

    /// `getLightEngine` — light engine.
    pub fn get_light_engine(&self) -> Option<&Arc<dyn crate::world::level::lighting::light_engine::LightEngine>> {
        self.light_engine.as_ref()
    }

    /// `setLightEngine` — установить light engine.
    pub fn set_light_engine(&mut self, engine: Arc<dyn crate::world::level::lighting::light_engine::LightEngine>) {
        self.light_engine = Some(engine);
    }
}
