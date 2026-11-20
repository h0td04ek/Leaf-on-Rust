//! Перенос `net.minecraft.world.level.lighting.LightEventListener` и
//! `LayerLightEventListener` — интерфейсы слушателей света,
//! используемые `LevelLightEngine` и StarLight.

use crate::core::block_pos::BlockPos;
use crate::core::section_pos::SectionPos;
use crate::world::level::chunk::data_layer::DataLayer;
use crate::world::level::chunk_pos::ChunkPos;

/// `LightEventListener` — базовый интерфейс слушателя света.
pub trait LightEventListener: Send + Sync {
    /// `checkBlock(pos)`.
    fn check_block(&self, pos: BlockPos);
    /// `hasLightWork()`.
    fn has_light_work(&self) -> bool;
    /// `runLightUpdates()`.
    fn run_light_updates(&self) -> i32;
    /// `updateSectionStatus(pos, isQueueEmpty)` (BlockPos-вариант).
    fn update_section_status_block(&self, pos: BlockPos, is_queue_empty: bool) {
        self.update_section_status(SectionPos::of(&pos), is_queue_empty);
    }
    /// `updateSectionStatus(SectionPos, isQueueEmpty)`.
    fn update_section_status(&self, pos: SectionPos, is_queue_empty: bool);
    /// `setLightEnabled(chunkPos, lightEnabled)`.
    fn set_light_enabled(&self, chunk_pos: ChunkPos, light_enabled: bool);
    /// `propagateLightSources(chunkPos)`.
    fn propagate_light_sources(&self, chunk_pos: ChunkPos);
}

/// `LayerLightEventListener` — слушатель конкретного слоя света.
pub trait LayerLightEventListener: LightEventListener + Send + Sync {
    /// `getDataLayerData(sectionPos)`.
    fn get_data_layer_data(&self, pos: SectionPos) -> Option<DataLayer>;
    /// `getLightValue(pos)`.
    fn get_light_value(&self, pos: BlockPos) -> i32;
}

/// `DummyLightLayerEventListener` — пустая реализация.
pub struct DummyLightLayerEventListener;

impl LightEventListener for DummyLightLayerEventListener {
    fn check_block(&self, _pos: BlockPos) {}
    fn has_light_work(&self) -> bool {
        false
    }
    fn run_light_updates(&self) -> i32 {
        0
    }
    fn update_section_status(&self, _pos: SectionPos, _is_queue_empty: bool) {}
    fn set_light_enabled(&self, _chunk_pos: ChunkPos, _light_enabled: bool) {}
    fn propagate_light_sources(&self, _chunk_pos: ChunkPos) {}
}

impl LayerLightEventListener for DummyLightLayerEventListener {
    fn get_data_layer_data(&self, _pos: SectionPos) -> Option<DataLayer> {
        None
    }
    fn get_light_value(&self, _pos: BlockPos) -> i32 {
        0
    }
}
