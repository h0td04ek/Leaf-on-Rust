//! `net.minecraft.world.level.LevelSimulatedReader` и
//! `LevelSimulatedRW` — симулируемое чтение/запись (worldgen).


use crate::core::block_pos::BlockPos;
use crate::world::level::block::entity::block_entity::BlockEntityRef;
use crate::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::levelgen::heightmap::HeightmapType;

/// `LevelSimulatedReader`.
pub trait LevelSimulatedReader {
    /// `isStateAtPosition(pos, predicate)`.
    fn is_state_at_position(
        &self,
        pos: BlockPos,
        predicate: &dyn Fn(&BlockState) -> bool,
    ) -> bool;

    /// `isFluidAtPosition(pos, predicate)`.
    fn is_fluid_at_position(
        &self,
        pos: BlockPos,
        predicate: &dyn Fn(&FluidState) -> bool,
    ) -> bool;

    /// `getBlockEntity(pos)` — нетипизированный доступ
    /// (аналог `LevelReader.getBlockEntity(pos)`).
    fn get_block_entity(&self, pos: BlockPos) -> Option<BlockEntityRef>;

    /// `getBlockEntity(pos, type)` — фильтр по типу.
    fn get_block_entity_typed(
        &self,
        pos: BlockPos,
        type_: &BlockEntityType,
    ) -> Option<BlockEntityRef> {
        self.get_block_entity(pos)
            .filter(|be| be.block_entity_type_id() == &type_.id)
    }

    /// `getHeightmapPos(heightmapType, pos)`.
    fn get_heightmap_pos(&self, heightmap_type: HeightmapType, pos: BlockPos) -> BlockPos;
}

/// `LevelSimulatedRW` = `LevelSimulatedReader` + `LevelWriter`.
pub trait LevelSimulatedRw:
    LevelSimulatedReader + crate::world::level::level_writer::LevelWriter
{
}
