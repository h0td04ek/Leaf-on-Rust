//! `net.minecraft.world.ticks.LevelChunkTicks` — очередь тиков одного чанка.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::sync::{Arc, Mutex};

use crate::core::block_pos::BlockPos;

use super::scheduled_tick::{drain_order_cmp, SavedTick, ScheduledTick, TickKey, TickType};
use super::tick_access::{SerializableTickContainer, TickAccess, TickContainerAccess};

/// Обёртка для min-heap по `DRAIN_ORDER` (BinaryHeap — max-heap, поэтому Reverse).
#[derive(Debug)]
struct DrainOrdered<T: TickType>(ScheduledTick<T>);

impl<T: TickType> PartialEq for DrainOrdered<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.tick_eq_full(&other.0)
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
        // Reverse: верх кучи = наименьший по DRAIN_ORDER
        drain_order_cmp(&other.0, &self.0)
    }
}

/// `BiConsumer<LevelChunkTicks<T>, ScheduledTick<T>> onTickAdded` —
/// упрощённо до `Fn(&ScheduledTick<T>)` (контейнер передаётся через Weak
/// у LevelTicks).
pub type OnTickAdded<T> = Arc<dyn Fn(&ScheduledTick<T>) + Send + Sync>;

/// `LevelChunkTicks<T>`.
pub struct LevelChunkTicks<T: TickType> {
    /// `tickQueue` — PriorityQueue по DRAIN_ORDER.
    tick_queue: Mutex<BinaryHeap<Reverse<DrainOrdered<T>>>>,
    /// `pendingTicks` — тики, ещё не распакованные (из NBT).
    pending_ticks: Mutex<Option<Vec<SavedTick<T>>>>,
    /// `ticksPerPosition` — множество уникальных тиков.
    ticks_per_position: Mutex<HashSet<TickKey>>,
    /// `onTickAdded`.
    on_tick_added: Mutex<Option<OnTickAdded<T>>>,

    // Paper — rewrite chunk system: dirty-флаг для отложенного сохранения
    /// `dirty`.
    dirty: Mutex<bool>,
    /// `lastSaved`.
    last_saved: Mutex<i64>,
}

impl<T: TickType> Default for LevelChunkTicks<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TickType> LevelChunkTicks<T> {
    /// `new()`.
    pub fn new() -> Self {
        Self {
            tick_queue: Mutex::new(BinaryHeap::new()),
            pending_ticks: Mutex::new(None),
            ticks_per_position: Mutex::new(HashSet::new()),
            on_tick_added: Mutex::new(None),
            dirty: Mutex::new(false),
            last_saved: Mutex::new(i64::MIN),
        }
    }

    /// `new(pendingTicks)` — из сохранённого списка.
    pub fn with_pending_ticks(pending_ticks: Vec<SavedTick<T>>) -> Self
    where
        T: Clone,
    {
        let this = Self::new();
        {
            let mut set = this.ticks_per_position.lock().unwrap();
            for t in &pending_ticks {
                set.insert(t.key());
            }
        }
        *this.pending_ticks.lock().unwrap() = Some(pending_ticks);
        this
    }

    /// `setOnTickAdded(onTickAdded)`.
    pub fn set_on_tick_added(&self, on_tick_added: Option<OnTickAdded<T>>) {
        *self.on_tick_added.lock().unwrap() = on_tick_added;
    }

    /// `peek()`.
    pub fn peek(&self) -> Option<ScheduledTick<T>>
    where
        T: Clone,
    {
        self.tick_queue
            .lock()
            .unwrap()
            .peek()
            .map(|r| r.0 .0.clone())
    }

    /// `poll()`.
    pub fn poll(&self) -> Option<ScheduledTick<T>>
    where
        T: Clone,
    {
        let tick = self.tick_queue.lock().unwrap().pop().map(|r| r.0 .0);
        if let Some(t) = &tick {
            self.ticks_per_position.lock().unwrap().remove(&t.key());
            *self.dirty.lock().unwrap() = true;
        }
        tick
    }

    /// `scheduleUnchecked(tick)` — без проверки уникальности.
    fn schedule_unchecked(&self, tick: ScheduledTick<T>) {
        self.tick_queue
            .lock()
            .unwrap()
            .push(Reverse(DrainOrdered(tick.clone())));
        let cb = self.on_tick_added.lock().unwrap().clone();
        if let Some(cb) = cb {
            cb(&tick);
        }
    }

    /// `removeIf(predicate)`.
    pub fn remove_if(&self, predicate: &dyn Fn(&ScheduledTick<T>) -> bool) {
        let mut queue = self.tick_queue.lock().unwrap();
        let mut set = self.ticks_per_position.lock().unwrap();
        let old = std::mem::take(&mut *queue);
        let mut kept = BinaryHeap::new();
        for Reverse(item) in old {
            if predicate(&item.0) {
                set.remove(&item.0.key());
                *self.dirty.lock().unwrap() = true;
            } else {
                kept.push(Reverse(item));
            }
        }
        *queue = kept;
    }

    /// `getAll()` — stream всех тиков в очереди.
    pub fn get_all(&self) -> Vec<ScheduledTick<T>>
    where
        T: Clone,
    {
        self.tick_queue
            .lock()
            .unwrap()
            .iter()
            .map(|r| r.0 .0.clone())
            .collect()
    }

    /// `moonrise$isDirty(tick)`.
    pub fn moonrise_is_dirty(&self, tick: i64) -> bool {
        *self.dirty.lock().unwrap()
            || (!self.tick_queue.lock().unwrap().is_empty()
                && tick != *self.last_saved.lock().unwrap())
    }

    /// `moonrise$clearDirty()`.
    pub fn moonrise_clear_dirty(&self) {
        *self.dirty.lock().unwrap() = false;
    }

    /// `unpack(gameTime)` — распаковать pendingTicks в очередь.
    pub fn unpack(&self, game_time: i64)
    where
        T: Clone,
    {
        let pending = self.pending_ticks.lock().unwrap().take();
        if let Some(pending) = pending {
            *self.last_saved.lock().unwrap() = game_time;
            let mut i = -(pending.len() as i64);
            for saved in &pending {
                self.schedule_unchecked(saved.unpack(game_time, i));
                i += 1;
            }
        }
    }

    /// Доступ к pending ticks (для NBT-загрузки).
    pub fn pending_ticks(&self) -> Option<Vec<SavedTick<T>>>
    where
        T: Clone,
    {
        self.pending_ticks.lock().unwrap().clone()
    }
}

impl<T: TickType + Clone> TickAccess<T> for LevelChunkTicks<T> {
    fn schedule(&self, tick: ScheduledTick<T>) {
        if self
            .ticks_per_position
            .lock()
            .unwrap()
            .insert(tick.key())
        {
            self.schedule_unchecked(tick);
            *self.dirty.lock().unwrap() = true;
        }
    }

    fn has_scheduled_tick(&self, pos: BlockPos, type_: &T) -> bool {
        // probe-ключ: ScheduledTick::probe(type, pos).key()
        // TickKey не зависит от trigger/priority — достаточно pos+type_hash
        self.ticks_per_position
            .lock()
            .unwrap()
            .contains(&TickKey::of(pos, type_))
    }

    fn count(&self) -> usize {
        self.tick_queue.lock().unwrap().len()
            + self
                .pending_ticks
                .lock()
                .unwrap()
                .as_ref()
                .map(|p| p.len())
                .unwrap_or(0)
    }
}

impl<T: TickType + Clone> TickContainerAccess<T> for LevelChunkTicks<T> {}

impl<T: TickType + Clone> SerializableTickContainer<T> for LevelChunkTicks<T> {
    fn pack(&self, game_time: i64) -> Vec<SavedTick<T>> {
        *self.last_saved.lock().unwrap() = game_time;
        let mut list = Vec::new();
        if let Some(pending) = self.pending_ticks.lock().unwrap().as_ref() {
            list.extend(pending.iter().cloned());
        }
        for Reverse(item) in self.tick_queue.lock().unwrap().iter() {
            list.push(item.0.to_saved_tick(game_time));
        }
        list
    }
}
