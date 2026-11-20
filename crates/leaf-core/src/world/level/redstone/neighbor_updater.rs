//! `net.minecraft.world.level.redstone.NeighborUpdater` —
//! интерфейс обновления соседних блоков + `executeShapeUpdate`.


use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::level_accessor::LevelAccessor;
use crate::world::level::redstone::orientation::Orientation;

/// `UPDATE_ORDER` — порядок обхода соседей.
pub const UPDATE_ORDER: [Direction; 6] = [
    Direction::WEST,
    Direction::EAST,
    Direction::DOWN,
    Direction::UP,
    Direction::NORTH,
    Direction::SOUTH,
];

/// `NeighborUpdater`.
pub trait NeighborUpdater {
    /// `shapeUpdate(direction, state, pos, neighborPos, flags, recursionLeft)`.
    fn shape_update(
        &self,
        direction: Direction,
        state: BlockState,
        pos: BlockPos,
        neighbor_pos: BlockPos,
        flags: i32,
        recursion_left: i32,
    );

    /// `neighborChanged(pos, neighborBlock, orientation)`.
    fn neighbor_changed(
        &self,
        pos: BlockPos,
        neighbor_block: &BlockState,
        orientation: Option<&Orientation>,
    );

    /// `neighborChanged(state, pos, neighborBlock, orientation, movedByPiston)`.
    fn neighbor_changed_state(
        &self,
        state: BlockState,
        pos: BlockPos,
        neighbor_block: &BlockState,
        orientation: Option<&Orientation>,
        moved_by_piston: bool,
    );

    /// `updateNeighborsAtExceptFromFacing(pos, block, facing, orientation)`.
    fn update_neighbors_at_except_from_facing(
        &self,
        pos: BlockPos,
        block: &BlockState,
        facing: Option<Direction>,
        _orientation: Option<&Orientation>,
    ) {
        for direction in UPDATE_ORDER {
            if Some(direction) != facing {
                self.neighbor_changed(pos.relative(direction), block, None);
            }
        }
    }
}

/// `NeighborUpdater.executeShapeUpdate(level, direction, pos,
/// neighborPos, neighborState, flags, recursionLeft)`.
///
/// `Block.UPDATE_SKIP_SHAPE_UPDATE_ON_WIRE` — реэкспорт.
pub use crate::world::level::block::update_flags::UPDATE_SKIP_SHAPE_UPDATE_ON_WIRE;

/// `Blocks.REDSTONE_WIRE` id для проверки пропуска.
const REDSTONE_WIRE_ID: &str = "minecraft:redstone_wire";

pub fn execute_shape_update(
    level: &dyn LevelAccessor,
    direction: Direction,
    pos: BlockPos,
    neighbor_pos: BlockPos,
    neighbor_state: &BlockState,
    flags: i32,
    recursion_left: i32,
) {
    let block_state = level
        .get_block_state(pos)
        .unwrap_or_else(BlockState::air);
    if (flags & UPDATE_SKIP_SHAPE_UPDATE_ON_WIRE) == 0
        || !block_state.is_block_id(REDSTONE_WIRE_ID)
    {
        let level_reader: &dyn crate::world::level::level_reader::LevelReader = level;
        let tick_access: &dyn crate::world::level::scheduled_tick_access::ScheduledTickAccess = level;
        let new_state = block_state.update_shape(
            level_reader,
            tick_access,
            pos,
            direction,
            neighbor_pos,
            neighbor_state,
            level.get_random(),
        );
        update_or_destroy(&block_state, &new_state, level, pos, flags, recursion_left);
    }
}

/// `Block.updateOrDestroy(oldState, newState, level, pos, flags, recursionLeft)`.
pub use crate::world::level::block::update_flags::UPDATE_SUPPRESS_DROPS;

pub fn update_or_destroy(
    old_state: &BlockState,
    new_state: &BlockState,
    level: &dyn LevelAccessor,
    pos: BlockPos,
    flags: i32,
    recursion_left: i32,
) {
    // BlockState == в Java — ссылочное/значимое равенство
    if !block_states_equal(old_state, new_state) {
        if new_state.is_air() {
            if !level.is_client_side() {
                crate::world::level::level_writer::LevelWriter::destroy_block(level, pos, (flags & UPDATE_SUPPRESS_DROPS) == 0, None, recursion_left);
            }
        } else {
            crate::world::level::level_writer::LevelWriter::set_block(
                level,
                pos,
                new_state.clone(),
                flags & !UPDATE_SUPPRESS_DROPS,
                recursion_left,
            );
        }
    }
}

/// Сравнение BlockState — по block id + значениям.
fn block_states_equal(a: &BlockState, b: &BlockState) -> bool {
    std::sync::Arc::ptr_eq(&a.block, &b.block)
        && a.values == b.values
}
