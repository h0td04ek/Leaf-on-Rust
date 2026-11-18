//! Перенос `net.minecraft.world.level.lighting.LightEngine` — базовый движок освещения.

use crate::core::block_pos::BlockPos;
use crate::core::section_pos::SectionPos;
use crate::world::level::block::state::block_state::BlockState;
use std::sync::Arc;

use crate::world::level::chunk::data_layer::DataLayer;
use crate::world::level::light_layer::LightLayer;

/// `LightEngine` — базовый движок освещения (упрощённая версия).
pub trait LightEngine: Send + Sync {
    /// `getLightValue(pos, layer)` — значение света в позиции.
    fn get_light_value(&self, pos: &BlockPos, layer: LightLayer) -> u8;

    /// `setLightValue(pos, layer, value)` — установить значение света.
    fn set_light_value(&self, pos: &BlockPos, layer: LightLayer, value: u8);

    /// `propagateLight(pos, layer)` — распространить свет.
    fn propagate_light(&self, pos: &BlockPos, layer: LightLayer);

    /// `updateLight(pos, layer)` — обновить свет.
    fn update_light(&self, pos: &BlockPos, layer: LightLayer);

    /// `checkLight(pos, layer)` — проверить свет.
    fn check_light(&self, pos: &BlockPos, layer: LightLayer);

    /// `getChunk(x, z)` — получить чанк.
    fn get_chunk(&self, x: i32, z: i32) -> Option<Arc<crate::world::level::chunk::chunk_access::ChunkAccess>>;

    /// `getChunkAt` — получить чанк по позиции.
    fn get_chunk_at(&self, pos: &BlockPos) -> Option<Arc<crate::world::level::chunk::chunk_access::ChunkAccess>> {
        let section_x = SectionPos::block_to_section_coord(pos.get_x() >> 4);
        let section_z = SectionPos::block_to_section_coord(pos.get_z() >> 4);
        self.get_chunk(section_x, section_z)
    }

    /// `getState(pos)` — состояние блока.
    fn get_state(&self, pos: &BlockPos) -> BlockState {
        if let Some(chunk) = self.get_chunk_at(pos) {
            chunk.get_block_state(pos)
        } else {
            BlockState::air()
        }
    }

    /// `getOpacity(state)` — непрозрачность.
    fn get_opacity(&self, state: &BlockState) -> i32 {
        state.get_light_block().max(1)
    }

    /// `getLightEmission(state)` — излучение света.
    fn get_light_emission(&self, state: &BlockState) -> u8 {
        state.get_light_emission()
    }

    /// `hasDifferentLightProperties(state1, state2)` — проверка разных свойств света.
    fn has_different_light_properties(&self, state1: &BlockState, state2: &BlockState) -> bool {
        state2 != state1 && (
            state2.get_light_block() != state1.get_light_block() ||
            state2.get_light_emission() != state1.get_light_emission() ||
            state2.use_shape_for_light_occlusion() != state1.use_shape_for_light_occlusion()
        )
    }
}
