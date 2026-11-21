//! `net.minecraft.world.level.EmptyBlockGetter` — singleton
//! пустого `BlockGetter` (всё air/empty).

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::world::level::block::entity::block_entity::BlockEntityAccess;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::block_getter::BlockGetter;

/// `EmptyBlockGetter.INSTANCE`.
pub struct EmptyBlockGetter;

/// Глобальный инстанс.
pub static EMPTY_BLOCK_GETTER: EmptyBlockGetter = EmptyBlockGetter;

impl crate::world::level::level_height_accessor::LevelHeightAccessor for EmptyBlockGetter {
    fn get_min_y(&self) -> i32 {
        0
    }
    fn get_height(&self) -> i32 {
        0
    }
}

impl BlockGetter for EmptyBlockGetter {
    fn get_block_entity(&self, _pos: BlockPos) -> Option<Arc<dyn BlockEntityAccess>> {
        None
    }

    /// Paper `getBlockStateIfLoaded` → `Some(air)`.
    fn get_block_state_if_loaded(&self, _pos: BlockPos) -> Option<BlockState> {
        Some(BlockState::air())
    }

    /// Paper `getFluidIfLoaded` → `Some(empty)`.
    fn get_fluid_if_loaded(&self, _pos: BlockPos) -> Option<FluidState> {
        Some(FluidState::empty())
    }

    fn get_block_state(&self, _pos: BlockPos) -> Option<BlockState> {
        Some(BlockState::air())
    }

    fn get_fluid_state(&self, _pos: BlockPos) -> Option<FluidState> {
        Some(FluidState::empty())
    }
}
