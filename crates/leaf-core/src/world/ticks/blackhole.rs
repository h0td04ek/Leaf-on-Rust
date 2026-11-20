//! `net.minecraft.world.ticks.BlackholeTickAccess` и
//! `WorldGenTickAccess` — «чёрные дыры» тиков и worldgen-доступ.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;

use super::scheduled_tick::{ScheduledTick, TickType};
use super::tick_access::{LevelTickAccess, TickAccess, TickContainerAccess};

/// Пустой контейнер тиков (`TickContainerAccess` blackhole).
pub struct BlackholeContainer<T: TickType>(std::marker::PhantomData<T>);

impl<T: TickType> Default for BlackholeContainer<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: TickType> TickAccess<T> for BlackholeContainer<T> {
    fn schedule(&self, _tick: ScheduledTick<T>) {}
    fn has_scheduled_tick(&self, _pos: BlockPos, _type_: &T) -> bool {
        false
    }
    fn count(&self) -> usize {
        0
    }
}
impl<T: TickType> TickContainerAccess<T> for BlackholeContainer<T> {}

/// Пустой level-доступ тиков (`LevelTickAccess` blackhole).
pub struct BlackholeLevel<T: TickType>(std::marker::PhantomData<T>);

impl<T: TickType> Default for BlackholeLevel<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: TickType> TickAccess<T> for BlackholeLevel<T> {
    fn schedule(&self, _tick: ScheduledTick<T>) {}
    fn has_scheduled_tick(&self, _pos: BlockPos, _type_: &T) -> bool {
        false
    }
    fn count(&self) -> usize {
        0
    }
}

impl<T: TickType> LevelTickAccess<T> for BlackholeLevel<T> {
    fn will_tick_this_tick(&self, _pos: BlockPos, _type_: &T) -> bool {
        false
    }
}

/// `BlackholeTickAccess.emptyContainer()`.
pub fn empty_container<T: TickType>() -> BlackholeContainer<T> {
    BlackholeContainer::default()
}

/// `BlackholeTickAccess.emptyLevelList()`.
pub fn empty_level_list<T: TickType>() -> BlackholeLevel<T> {
    BlackholeLevel::default()
}

/// `WorldGenTickAccess<T>` — делегирует в контейнер по позиции.
pub struct WorldGenTickAccess<T: TickType> {
    /// `containerGetter`.
    container_getter: Arc<dyn Fn(BlockPos) -> Arc<dyn TickContainerAccess<T>> + Send + Sync>,
}

impl<T: TickType> WorldGenTickAccess<T> {
    /// `new(containerGetter)`.
    pub fn new(
        container_getter: Arc<dyn Fn(BlockPos) -> Arc<dyn TickContainerAccess<T>> + Send + Sync>,
    ) -> Self {
        Self { container_getter }
    }
}

impl<T: TickType + Clone> TickAccess<T> for WorldGenTickAccess<T> {
    fn schedule(&self, tick: ScheduledTick<T>) {
        (self.container_getter)(tick.pos).schedule(tick);
    }

    fn has_scheduled_tick(&self, pos: BlockPos, type_: &T) -> bool {
        (self.container_getter)(pos).has_scheduled_tick(pos, type_)
    }

    fn count(&self) -> usize {
        0
    }
}

impl<T: TickType + Clone> LevelTickAccess<T> for WorldGenTickAccess<T> {
    fn will_tick_this_tick(&self, _pos: BlockPos, _type_: &T) -> bool {
        false
    }
}
