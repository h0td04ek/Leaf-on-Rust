//! Перенос `net.minecraft.world.level.lighting.LevelLightEngine` — движок освещения уровня.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::world::level::light_layer::LightLayer;
use crate::world::level::lighting::light_engine::LightEngine;

/// `LevelLightEngine` — движок освещения уровня.
pub struct LevelLightEngine {
    /// `chunkSource` — источник чанков.
    chunk_source: Arc<dyn crate::world::level::chunk_source::ChunkSource>,
    /// `skyEngine` — sky light engine.
    pub sky_engine: Option<Arc<dyn LightEngine>>,
    /// `blockEngine` — block light engine.
    pub block_engine: Option<Arc<dyn LightEngine>>,
}

impl LevelLightEngine {
    /// `new(chunkSource)`.
    pub fn new(chunk_source: Arc<dyn crate::world::level::chunk_source::ChunkSource>) -> Self {
        Self {
            chunk_source,
            sky_engine: None,
            block_engine: None,
        }
    }

    /// `getChunkSource`.
    pub fn get_chunk_source(&self) -> &Arc<dyn crate::world::level::chunk_source::ChunkSource> {
        &self.chunk_source
    }

    /// `getLightValue(pos, layer)`.
    pub fn get_light_value(&self, pos: &BlockPos, layer: LightLayer) -> u8 {
        match layer {
            LightLayer::Sky => self.sky_engine.as_ref().map(|e| e.get_light_value(pos, layer)).unwrap_or(0),
            LightLayer::Block => self.block_engine.as_ref().map(|e| e.get_light_value(pos, layer)).unwrap_or(0),
        }
    }

    /// `setLightValue(pos, layer, value)`.
    pub fn set_light_value(&self, pos: &BlockPos, layer: LightLayer, value: u8) {
        match layer {
            LightLayer::Sky => {
                if let Some(engine) = &self.sky_engine {
                    engine.set_light_value(pos, layer, value);
                }
            }
            LightLayer::Block => {
                if let Some(engine) = &self.block_engine {
                    engine.set_light_value(pos, layer, value);
                }
            }
        }
    }

    /// `propagateLight(pos, layer)`.
    pub fn propagate_light(&self, pos: &BlockPos, layer: LightLayer) {
        match layer {
            LightLayer::Sky => {
                if let Some(engine) = &self.sky_engine {
                    engine.propagate_light(pos, layer);
                }
            }
            LightLayer::Block => {
                if let Some(engine) = &self.block_engine {
                    engine.propagate_light(pos, layer);
                }
            }
        }
    }

    /// `updateLight(pos, layer)`.
    pub fn update_light(&self, pos: &BlockPos, layer: LightLayer) {
        match layer {
            LightLayer::Sky => {
                if let Some(engine) = &self.sky_engine {
                    engine.update_light(pos, layer);
                }
            }
            LightLayer::Block => {
                if let Some(engine) = &self.block_engine {
                    engine.update_light(pos, layer);
                }
            }
        }
    }

    /// `checkLight(pos, layer)`.
    pub fn check_light(&self, pos: &BlockPos, layer: LightLayer) {
        match layer {
            LightLayer::Sky => {
                if let Some(engine) = &self.sky_engine {
                    engine.check_light(pos, layer);
                }
            }
            LightLayer::Block => {
                if let Some(engine) = &self.block_engine {
                    engine.check_light(pos, layer);
                }
            }
        }
    }

    /// `getMinLightSection` — минимальная light-секция.
    pub fn get_min_light_section(&self) -> i32 {
        self.chunk_source.get_min_section_y()
    }

    /// `getLightSectionCount` — количество light-секций.
    pub fn get_light_section_count(&self) -> i32 {
        self.chunk_source.get_sections_count()
    }

    /// `getLayerListener(layer)` — listener для слоя.
    pub fn get_layer_listener(&self, layer: LightLayer) -> Option<&Arc<dyn LightEngine>> {
        match layer {
            LightLayer::Sky => self.sky_engine.as_ref(),
            LightLayer::Block => self.block_engine.as_ref(),
        }
    }
}
