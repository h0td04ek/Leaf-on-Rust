//! `net.minecraft.world.ticks.LevelChunkTicks` — очередь тиков одного чанка.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::sync::{Arc, Mutex};

use crate::core::block_pos::BlockPos;

use super::scheduled_tick::{drain_order_cmp, SavedTick, ScheduledTick, TickKey, TickType};
use super::tick_access::{SerializableTickContainer, TickAccess, TickContainerAccess};

struct DrainOrdered<T: TickType>(ScheduledTick<T>);

impl<T: TickType> PartialEq for DrainOrdered<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<T: TickType> Eq for DrainOrdered<T> {}

impl<T: TickType> PartialOrd for DrainOrdered<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: TickType> Ord for DrainOrdered<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        drain_order_cmp(&self.0, &other.0)
    }
}

/// Вызывается после добавления тика и освобождения блокировки очереди.
pub type OnTickAdded<T> = Arc<dyn Fn(&ScheduledTick<T>) + Send + Sync>;

pub struct LevelChunkTicks<T: TickType> {
    state: Mutex<TickQueue<T>>,
}

struct TickQueue<T: TickType> {
    scheduled: BinaryHeap<Reverse<DrainOrdered<T>>>,
    pending: Vec<SavedTick<T>>,
    keys: HashSet<TickKey>,
    on_tick_added: Option<OnTickAdded<T>>,
    dirty: bool,
    last_saved: i64,
}

impl<T: TickType> Default for LevelChunkTicks<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TickType> LevelChunkTicks<T> {
    pub fn new() -> Self {
        Self::with_pending_ticks(Vec::new())
    }

    pub fn with_pending_ticks(mut pending: Vec<SavedTick<T>>) -> Self {
        let mut keys = HashSet::with_capacity(pending.len());
        pending.retain(|tick| keys.insert(tick.key()));
        Self {
            state: Mutex::new(TickQueue {
                scheduled: BinaryHeap::new(),
                pending,
                keys,
                on_tick_added: None,
                dirty: false,
                last_saved: i64::MIN,
            }),
        }
    }

    pub fn set_on_tick_added(&self, callback: Option<OnTickAdded<T>>) {
        self.state.lock().unwrap().on_tick_added = callback;
    }

    pub fn peek(&self) -> Option<ScheduledTick<T>> {
        let state = self.state.lock().unwrap();
        state.scheduled.peek().map(|Reverse(item)| item.0.clone())
    }

    pub fn poll(&self) -> Option<ScheduledTick<T>> {
        let mut state = self.state.lock().unwrap();
        let Reverse(item) = state.scheduled.pop()?;
        state.keys.remove(&item.0.key());
        state.dirty = true;
        Some(item.0)
    }

    pub fn remove_if(&self, predicate: &dyn Fn(&ScheduledTick<T>) -> bool) {
        let mut state = self.state.lock().unwrap();
        let TickQueue {
            scheduled,
            keys,
            dirty,
            ..
        } = &mut *state;
        scheduled.retain(|Reverse(item)| {
            if predicate(&item.0) {
                keys.remove(&item.0.key());
                *dirty = true;
                false
            } else {
                true
            }
        });
    }

    pub fn get_all(&self) -> Vec<ScheduledTick<T>> {
        let state = self.state.lock().unwrap();
        state
            .scheduled
            .iter()
            .map(|Reverse(item)| item.0.clone())
            .collect()
    }

    pub fn moonrise_is_dirty(&self, tick: i64) -> bool {
        let state = self.state.lock().unwrap();
        state.dirty || (!state.scheduled.is_empty() && tick != state.last_saved)
    }

    pub fn moonrise_clear_dirty(&self) {
        self.state.lock().unwrap().dirty = false;
    }

    /// Привязывает сохранённые задержки к моменту активации чанка.
    pub fn unpack(&self, game_time: i64) {
        let (callback, unpacked) = {
            let mut state = self.state.lock().unwrap();
            let pending = std::mem::take(&mut state.pending);
            if pending.is_empty() {
                return;
            }
            state.last_saved = game_time;
            let order = -(pending.len() as i64);
            let unpacked: Vec<_> = pending
                .iter()
                .enumerate()
                .map(|(index, tick)| tick.unpack(game_time, order + index as i64))
                .collect();
            state.scheduled.extend(
                unpacked
                    .iter()
                    .cloned()
                    .map(|tick| Reverse(DrainOrdered(tick))),
            );
            (state.on_tick_added.clone(), unpacked)
        };
        if let Some(callback) = callback {
            for tick in &unpacked {
                callback(tick);
            }
        }
    }

    pub fn pending_ticks(&self) -> Option<Vec<SavedTick<T>>> {
        let state = self.state.lock().unwrap();
        (!state.pending.is_empty()).then(|| state.pending.clone())
    }
}

impl<T: TickType> TickAccess<T> for LevelChunkTicks<T> {
    fn schedule(&self, tick: ScheduledTick<T>) {
        let callback = {
            let mut state = self.state.lock().unwrap();
            if !state.keys.insert(tick.key()) {
                return;
            }
            state.scheduled.push(Reverse(DrainOrdered(tick.clone())));
            state.dirty = true;
            state.on_tick_added.clone()
        };
        if let Some(callback) = callback {
            callback(&tick);
        }
    }

    fn has_scheduled_tick(&self, pos: BlockPos, type_: &T) -> bool {
        self.state
            .lock()
            .unwrap()
            .keys
            .contains(&TickKey::of(pos, type_))
    }

    fn count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.pending.len() + state.scheduled.len()
    }
}

impl<T: TickType> TickContainerAccess<T> for LevelChunkTicks<T> {}

impl<T: TickType> SerializableTickContainer<T> for LevelChunkTicks<T> {
    fn pack(&self, game_time: i64) -> Vec<SavedTick<T>> {
        let mut state = self.state.lock().unwrap();
        state.last_saved = game_time;
        let mut list = state.pending.clone();
        let mut scheduled: Vec<_> = state
            .scheduled
            .iter()
            .map(|Reverse(item)| &item.0)
            .collect();
        scheduled.sort_unstable_by(|a, b| drain_order_cmp(a, b));
        list.extend(
            scheduled
                .into_iter()
                .map(|tick| tick.to_saved_tick(game_time)),
        );
        list
    }
}
