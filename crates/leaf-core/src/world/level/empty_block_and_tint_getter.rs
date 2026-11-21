//! `net.minecraft.world.level.EmptyBlockAndTintGetter` —
//! singleton пустого `BlockAndTintGetter`.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::world::level::block::entity::block_entity::BlockEntityAccess;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::block_and_tint_getter::BlockAndTintGetter;
use crate::world::level::block_getter::BlockGetter;
use crate::world::level::color_resolver::ColorResolver;
use crate::world::level::lighting::level_light_engine::LevelLightEngine;

/// `EmptyBlockAndTintGetter.INSTANCE`.
pub struct EmptyBlockAndTintGetter;

/// Глобальный инстанс.
pub static EMPTY_BLOCK_AND_TINT_GETTER: EmptyBlockAndTintGetter = EmptyBlockAndTintGetter;

impl crate::world::level::level_height_accessor::LevelHeightAccessor for EmptyBlockAndTintGetter {
    fn get_min_y(&self) -> i32 {
        0
    }
    fn get_height(&self) -> i32 {
        0
    }
}

impl BlockGetter for EmptyBlockAndTintGetter {
    fn get_block_entity(&self, _pos: BlockPos) -> Option<Arc<dyn BlockEntityAccess>> {
        None
    }

    /// Paper `getBlockStateIfLoaded` → `None`.
    fn get_block_state_if_loaded(&self, _pos: BlockPos) -> Option<BlockState> {
        None
    }

    /// Paper `getFluidIfLoaded` → `None`.
    fn get_fluid_if_loaded(&self, _pos: BlockPos) -> Option<FluidState> {
        None
    }

    fn get_block_state(&self, _pos: BlockPos) -> Option<BlockState> {
        Some(BlockState::air())
    }

    fn get_fluid_state(&self, _pos: BlockPos) -> Option<FluidState> {
        Some(FluidState::empty())
    }
}

impl BlockAndTintGetter for EmptyBlockAndTintGetter {
    fn get_shade(&self, _direction: Direction, _shade: bool) -> f32 {
        1.0
    }

    /// `LevelLightEngine.EMPTY` — owned empty engine.
    /// Возвращаем owned-копию через `OnceLock`-кэш.
    fn get_light_engine(&self) -> Option<&'static LevelLightEngine> {
        static EMPTY: std::sync::OnceLock<LevelLightEngine> = std::sync::OnceLock::new();
        Some(EMPTY.get_or_init(LevelLightEngine::empty))
    }

    fn get_block_tint(&self, _pos: BlockPos, _color_resolver: &dyn ColorResolver) -> i32 {
        -1
    }

    /// Java `getBrightness` через EMPTY-движок → 0;
    /// `canSeeSky` → false.
    fn get_brightness(
        &self,
        _light_layer: crate::world::level::light_layer::LightLayer,
        _pos: BlockPos,
    ) -> i32 {
        0
    }

    fn get_raw_brightness(&self, _pos: BlockPos, _amount_to_subtract: i32) -> i32 {
        0
    }
}
