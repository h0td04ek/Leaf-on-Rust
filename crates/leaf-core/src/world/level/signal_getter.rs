//! `net.minecraft.world.level.SignalGetter` — редстоун-сигналы.
//!
//! Отличие от Java: `BlockState::{get_signal,get_direct_signal,
//! is_redstone_conductor}` требуют `&dyn LevelReader`, поэтому
//! при `as_level_reader() == None` возвращаем 0 (в Java `this`
//! — `BlockGetter`; для `Level`/`ChunkAccess` LevelReader всегда
//! доступен).

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::world::level::block_getter::BlockGetter;
use crate::world::level::block::state::block_state::BlockState;

/// `SignalGetter extends BlockGetter`.
pub trait SignalGetter: BlockGetter {
    /// `getDirectSignal(pos, direction)`.
    fn get_direct_signal(&self, pos: BlockPos, direction: Direction) -> i32 {
        let Some(state) = self.get_block_state(pos) else {
            return 0;
        };
        let Some(level) = self.as_level_reader() else {
            return 0;
        };
        state.get_direct_signal(level, pos, direction)
    }

    /// `getDirectSignalTo(pos)` — max по 6 соседям с ранним выходом 15.
    fn get_direct_signal_to(&self, pos: BlockPos) -> i32 {
        let mut i = self.get_direct_signal(pos.below(), Direction::DOWN);
        if i >= 15 {
            return i;
        }
        i = i.max(self.get_direct_signal(pos.above(), Direction::UP));
        if i >= 15 {
            return i;
        }
        i = i.max(self.get_direct_signal(pos.north(), Direction::NORTH));
        if i >= 15 {
            return i;
        }
        i = i.max(self.get_direct_signal(pos.south(), Direction::SOUTH));
        if i >= 15 {
            return i;
        }
        i = i.max(self.get_direct_signal(pos.west(), Direction::WEST));
        if i >= 15 {
            return i;
        }
        i.max(self.get_direct_signal(pos.east(), Direction::EAST))
    }

    /// `getControlInputSignal(pos, direction, diodesOnly)`.
    fn get_control_input_signal(
        &self,
        pos: BlockPos,
        direction: Direction,
        diodes_only: bool,
    ) -> i32 {
        let Some(state) = self.get_block_state(pos) else {
            return 0;
        };
        if diodes_only {
            return if is_diode(&state) {
                self.get_direct_signal(pos, direction)
            } else {
                0
            };
        }
        if is_block(&state, "redstone_block") {
            15
        } else if is_block(&state, "redstone_wire") {
            state.get_int_property("power").unwrap_or(0)
        } else if state.is_signal_source() {
            self.get_direct_signal(pos, direction)
        } else {
            0
        }
    }

    /// `hasSignal(pos, direction)`.
    fn has_signal(&self, pos: BlockPos, direction: Direction) -> bool {
        self.get_signal(pos, direction) > 0
    }

    /// `getSignal(pos, direction)`.
    fn get_signal(&self, pos: BlockPos, direction: Direction) -> i32 {
        let Some(state) = self.get_block_state(pos) else {
            return 0;
        };
        let Some(level) = self.as_level_reader() else {
            return 0;
        };
        let signal = state.get_signal(level, pos, direction);
        if state.is_redstone_conductor(level, pos) {
            signal.max(self.get_direct_signal_to(pos))
        } else {
            signal
        }
    }

    /// `hasNeighborSignal(pos)`.
    fn has_neighbor_signal(&self, pos: BlockPos) -> bool {
        self.get_signal(pos.below(), Direction::DOWN) > 0
            || self.get_signal(pos.above(), Direction::UP) > 0
            || self.get_signal(pos.north(), Direction::NORTH) > 0
            || self.get_signal(pos.south(), Direction::SOUTH) > 0
            || self.get_signal(pos.west(), Direction::WEST) > 0
            || self.get_signal(pos.east(), Direction::EAST) > 0
    }

    /// `getBestNeighborSignal(pos)`.
    fn get_best_neighbor_signal(&self, pos: BlockPos) -> i32 {
        let mut i = 0;
        for direction in Direction::VALUES {
            let signal = self.get_signal(pos.relative(direction), direction);
            if signal >= 15 {
                return 15;
            }
            if signal > i {
                i = signal;
            }
        }
        i
    }
}

/// `DiodeBlock.isDiode(state)` — repeater/comparator.
fn is_diode(state: &BlockState) -> bool {
    is_block(state, "repeater") || is_block(state, "comparator")
}

/// `state.is(Blocks.X)` по path (minecraft namespace).
fn is_block(state: &BlockState, path: &str) -> bool {
    state.block.id.namespace == "minecraft" && state.block.id.path == path
}
