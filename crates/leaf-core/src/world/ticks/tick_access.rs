//! `net.minecraft.world.ticks` — интерфейсы доступа к тикам.

use crate::core::block_pos::BlockPos;

use super::scheduled_tick::{SavedTick, ScheduledTick, TickType};

/// `TickAccess<T>` — базовый доступ к тикам.
pub trait TickAccess<T: TickType> {
    /// `schedule(tick)`.
    fn schedule(&self, tick: ScheduledTick<T>);
    /// `hasScheduledTick(pos, type)`.
    fn has_scheduled_tick(&self, pos: BlockPos, type_: &T) -> bool;
    /// `count()`.
    fn count(&self) -> usize;
}

/// `TickContainerAccess<T>` — доступ к контейнеру тиков чанка.
pub trait TickContainerAccess<T: TickType>: TickAccess<T> {}

/// `LevelTickAccess<T>` — доступ к тикам уровня.
pub trait LevelTickAccess<T: TickType>: TickAccess<T> {
    /// `willTickThisTick(pos, type)`.
    fn will_tick_this_tick(&self, pos: BlockPos, type_: &T) -> bool;
}

/// `SerializableTickContainer<T>` — контейнер, сериализуемый в `SavedTick`.
pub trait SerializableTickContainer<T: TickType>: TickContainerAccess<T> {
    /// `pack(gameTime)` → список сохранённых тиков.
    fn pack(&self, game_time: i64) -> Vec<SavedTick<T>>
    where
        T: Clone;
}
