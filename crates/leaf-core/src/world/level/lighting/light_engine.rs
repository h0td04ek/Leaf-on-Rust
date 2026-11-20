//! Перенос `net.minecraft.world.level.lighting.LightEngine` — базовый движок освещения.

use crate::core::block_pos::BlockPos;
use crate::core::section_pos::SectionPos;
use crate::world::level::block::state::block_state::BlockState;
use std::sync::Arc;

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
        has_different_light_properties(state1, state2)
    }
}

/// `LightEngine.isEmptyShape(state)` — `!canOcclude || !useShapeForLightOcclusion`.
pub fn is_empty_shape(state: &BlockState) -> bool {
    !state.can_occlude() || !state.use_shape_for_light_occlusion()
}

/// `LightEngine.getOcclusionShape(state, direction)`.
pub fn get_occlusion_shape(
    state: &BlockState,
    direction: crate::core::direction::Direction,
) -> crate::world::phys::shapes::VoxelShape {
    if is_empty_shape(state) {
        crate::world::phys::shapes::VoxelShape::empty()
    } else {
        state.face_occlusion_shape_unleveled(direction)
    }
}

/// `LightEngine.getLightBlockInto(state1, state2, direction, default)`.
pub fn get_light_block_into(
    state1: &BlockState,
    state2: &BlockState,
    direction: crate::core::direction::Direction,
    default_return_value: i32,
) -> i32 {
    let is_empty_shape1 = is_empty_shape(state1);
    let is_empty_shape2 = is_empty_shape(state2);
    if is_empty_shape1 && is_empty_shape2 {
        return default_return_value;
    }
    let shape1 = if is_empty_shape1 {
        crate::world::phys::shapes::VoxelShape::empty()
    } else {
        state1.occlusion_shape_unleveled()
    };
    let shape2 = if is_empty_shape2 {
        crate::world::phys::shapes::VoxelShape::empty()
    } else {
        state2.occlusion_shape_unleveled()
    };
    if crate::world::phys::shapes::merged_face_occludes(&shape1, &shape2, direction) {
        16
    } else {
        default_return_value
    }
}

/// `LightEngine.shapeOccludes(state1, state2, direction)`.
pub fn shape_occludes(
    state1: &BlockState,
    state2: &BlockState,
    direction: crate::core::direction::Direction,
) -> bool {
    let occlusion_shape = get_occlusion_shape(state1, direction);
    let occlusion_shape1 = get_occlusion_shape(state2, direction.get_opposite());
    crate::world::phys::shapes::face_shape_occludes(&occlusion_shape, &occlusion_shape1)
}

/// `LightEngine.hasDifferentLightProperties(state1, state2)`.
pub fn has_different_light_properties(state1: &BlockState, state2: &BlockState) -> bool {
    state2 != state1
        && (state2.get_light_block() != state1.get_light_block()
            || state2.get_light_emission() != state1.get_light_emission()
            || state2.use_shape_for_light_occlusion()
            || state1.use_shape_for_light_occlusion())
}
