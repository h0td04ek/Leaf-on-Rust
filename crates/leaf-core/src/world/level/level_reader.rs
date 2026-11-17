//! Перенос `net.minecraft.world.level.LevelReader` — read-only
//! доступ к уровню для предикатов и паттернов.

use crate::core::block_pos::BlockPos;
use crate::world::level::block::entity::block_entity::BlockEntityAccess;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use std::sync::Arc;

/// `LevelReader` — минимальный read-интерфейс уровня.
///
/// Реализуется `Level`/`ServerLevel`/`WorldGenLevel` по мере порта.
pub trait LevelReader {
    /// `getBlockState(pos)` — `None` если чанк не загружен
    /// (в Java возвращает air — здесь Option честнее).
    fn get_block_state(&self, pos: BlockPos) -> Option<BlockState>;

    /// `getFluidState(pos)`.
    fn get_fluid_state(&self, pos: BlockPos) -> Option<FluidState> {
        None
    }

    /// `getBlockEntity(pos)`.
    fn get_block_entity(&self, pos: BlockPos) -> Option<Arc<dyn BlockEntityAccess>> {
        None
    }

    /// `hasChunkAt(pos)`.
    fn has_chunk_at(&self, pos: BlockPos) -> bool;

    /// `isLoaded(pos)` — `hasChunk` + позиция внутри build height.
    fn is_loaded(&self, pos: BlockPos) -> bool {
        self.has_chunk_at(pos)
    }

    /// `getMinBuildHeight`.
    fn min_build_height(&self) -> i32 {
        -64
    }

    /// `getHeight`.
    fn height(&self) -> i32 {
        384
    }
}
