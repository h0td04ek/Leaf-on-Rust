//! `net.minecraft.world.ticks.ProtoChunkTicks` — тики proto-чанка
//! (сохраняются как `SavedTick` с delay=0 до загрузки).

use std::collections::HashSet;
use std::sync::Mutex;

use crate::core::block_pos::BlockPos;

use super::scheduled_tick::{SavedTick, ScheduledTick, TickKey, TickType};
use super::tick_access::{SerializableTickContainer, TickAccess, TickContainerAccess};

/// `ProtoChunkTicks<T>`.
pub struct ProtoChunkTicks<T: TickType> {
    /// `ticks`.
    ticks: Mutex<Vec<SavedTick<T>>>,
    /// `ticksPerPosition`.
    ticks_per_position: Mutex<HashSet<TickKey>>,
}

impl<T: TickType> Default for ProtoChunkTicks<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TickType> ProtoChunkTicks<T> {
    /// `new()`.
    pub fn new() -> Self {
        Self {
            ticks: Mutex::new(Vec::new()),
            ticks_per_position: Mutex::new(HashSet::new()),
        }
    }

    /// `load(ticks)`.
    pub fn load(ticks: Vec<SavedTick<T>>) -> Self
    where
        T: Clone,
    {
        let this = Self::new();
        {
            let mut set = this.ticks_per_position.lock().unwrap();
            let mut list = this.ticks.lock().unwrap();
            for t in ticks {
                if set.insert(t.key()) {
                    list.push(t);
                }
            }
        }
        this
    }

    /// `scheduledTicks()` — копия списка.
    pub fn scheduled_ticks(&self) -> Vec<SavedTick<T>>
    where
        T: Clone,
    {
        self.ticks.lock().unwrap().clone()
    }
}

impl<T: TickType + Clone> TickAccess<T> for ProtoChunkTicks<T> {
    fn schedule(&self, tick: ScheduledTick<T>) {
        let saved = SavedTick::new(tick.type_, tick.pos, 0, tick.priority);
        if self
            .ticks_per_position
            .lock()
            .unwrap()
            .insert(saved.key())
        {
            self.ticks.lock().unwrap().push(saved);
        }
    }

    fn has_scheduled_tick(&self, pos: BlockPos, type_: &T) -> bool {
        self.ticks_per_position
            .lock()
            .unwrap()
            .contains(&TickKey::of(pos, type_))
    }

    fn count(&self) -> usize {
        self.ticks.lock().unwrap().len()
    }
}

impl<T: TickType + Clone> TickContainerAccess<T> for ProtoChunkTicks<T> {}

impl<T: TickType + Clone> SerializableTickContainer<T> for ProtoChunkTicks<T> {
    fn pack(&self, _game_time: i64) -> Vec<SavedTick<T>> {
        self.ticks.lock().unwrap().clone()
    }
}
