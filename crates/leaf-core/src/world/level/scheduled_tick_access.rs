//! `net.minecraft.world.level.ScheduledTickAccess` — доступ к
//! планировщику тиков уровня.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::ticks::{LevelTickAccess, ScheduledTick, TickPriority};

/// Тип тика блока (`Block` как ключ реестрового синглтона).
#[derive(Clone, Debug)]
pub struct BlockTickType {
    /// Registry id блока.
    pub id: String,
}

impl BlockTickType {
    /// Из `BlockState`.
    pub fn of_state(state: &BlockState) -> Self {
        Self {
            id: state.block.id.to_string(),
        }
    }
}

impl crate::world::ticks::TickType for BlockTickType {
    fn tick_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.id.hash(&mut h);
        h.finish()
    }
    fn tick_eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Тип тика жидкости (`Fluid` как ключ реестрового синглтона).
#[derive(Clone, Debug)]
pub struct FluidTickType {
    /// Registry id жидкости.
    pub id: String,
}

impl FluidTickType {
    /// Из `FluidState`.
    pub fn of_state(state: &FluidState) -> Self {
        Self {
            id: state.fluid.id.to_string(),
        }
    }
}

impl crate::world::ticks::TickType for FluidTickType {
    fn tick_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.id.hash(&mut h);
        h.finish()
    }
    fn tick_eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// `ScheduledTickAccess` — object-safe (без generic-методов;
/// `createTick` конкретизирован под `Block`/`Fluid`).
pub trait ScheduledTickAccess {
    /// `createTick(pos, block, delay, priority)`.
    fn create_tick_block(
        &self,
        pos: BlockPos,
        type_: BlockTickType,
        delay: i32,
        priority: TickPriority,
    ) -> ScheduledTick<BlockTickType>;

    /// `createTick(pos, fluid, delay, priority)`.
    fn create_tick_fluid(
        &self,
        pos: BlockPos,
        type_: FluidTickType,
        delay: i32,
        priority: TickPriority,
    ) -> ScheduledTick<FluidTickType>;

    /// `getBlockTicks()`.
    fn get_block_ticks(&self) -> Arc<dyn LevelTickAccess<BlockTickType>>;

    /// `getFluidTicks()`.
    fn get_fluid_ticks(&self) -> Arc<dyn LevelTickAccess<FluidTickType>>;

    /// `scheduleTick(pos, block, delay, priority)`.
    fn schedule_tick_block(
        &self,
        pos: BlockPos,
        block: BlockTickType,
        delay: i32,
        priority: TickPriority,
    ) {
        let tick = self.create_tick_block(pos, block, delay, priority);
        self.get_block_ticks().schedule(tick);
    }

    /// `scheduleTick(pos, block, delay)`.
    fn schedule_tick_block_default(&self, pos: BlockPos, block: BlockTickType, delay: i32) {
        let tick = self.create_tick_block(pos, block, delay, TickPriority::Normal);
        self.get_block_ticks().schedule(tick);
    }

    /// `scheduleTick(pos, fluid, delay, priority)`.
    fn schedule_tick_fluid(
        &self,
        pos: BlockPos,
        fluid: FluidTickType,
        delay: i32,
        priority: TickPriority,
    ) {
        let tick = self.create_tick_fluid(pos, fluid, delay, priority);
        self.get_fluid_ticks().schedule(tick);
    }

    /// `scheduleTick(pos, fluid, delay)`.
    fn schedule_tick_fluid_default(&self, pos: BlockPos, fluid: FluidTickType, delay: i32) {
        let tick = self.create_tick_fluid(pos, fluid, delay, TickPriority::Normal);
        self.get_fluid_ticks().schedule(tick);
    }
}
