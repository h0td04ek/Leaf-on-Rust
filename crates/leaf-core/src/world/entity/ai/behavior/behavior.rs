//! Перенос `net.minecraft.world.entity.ai.behavior` —
//! фреймворк Brain-поведений: `BehaviorControl`, `Behavior`
//! (entryCondition + duration), `OneShot`, `ShufflingList`.
//!
//! В Java `BehaviorControl<E extends LivingEntity>`; у нас
//! `E` = `Mob` (PathfinderMob-эквивалент). Объекты хранятся как
//! `Box<dyn BehaviorControl>` в `Brain`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::world::entity::ai::memory::memory::{MemoryModuleType, MemoryStatus};
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;

/// `Behavior.Status` — STOPPED/RUNNING.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorStatus {
    /// `STOPPED`.
    Stopped,
    /// `RUNNING`.
    Running,
}

/// `BehaviorControl<E>` — контролируемое поведение Brain.
/// Методы `try_start`/`tick_or_stop`/`do_stop` имеют дефолтную
/// Java-семантику `Behavior` (entryCondition + duration);
/// наследники переопределяют hooks.
pub trait BehaviorControl {
    /// `getStatus()`.
    fn status(&self) -> BehaviorStatus;
    /// Установка статуса (аналог поля `status` в Java).
    fn set_status(&mut self, status: BehaviorStatus);

    /// `entryCondition` — Map<MemoryModuleType, MemoryStatus>.
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus>;

    /// `(minDuration, maxDuration)`.
    fn durations(&self) -> (i32, i32);

    /// `endTimestamp`.
    fn end_timestamp(&self) -> i64;
    /// Установка `endTimestamp`.
    fn set_end_timestamp(&mut self, timestamp: i64);

    // ---- hooks (переопределяются наследниками) ----

    /// `start(level, entity, gameTime)`.
    fn start(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) {}

    /// `tick(level, owner, gameTime)`.
    fn tick(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) {}

    /// `stop(level, entity, gameTime)`.
    fn stop(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) {}

    /// `canStillUse(level, entity, gameTime)` — default false
    /// (Java `Behavior.canStillUse` → false, OneShot → true
    /// пока не timedOut).
    fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) -> bool {
        false
    }

    /// `checkExtraStartConditions(level, owner)` — default true.
    fn check_extra_start_conditions(&mut self, _level: &Arc<dyn LevelReader>, _mob: &Mob) -> bool {
        true
    }

    /// `debugString()` — имя класса для отладки.
    fn debug_string(&self) -> String {
        "Behavior".to_string()
    }

    // ---- final-методы Java Behavior ----

    /// `timedOut(gameTime)` — `gameTime > endTimestamp`.
    fn timed_out(&self, game_time: i64) -> bool {
        game_time > self.end_timestamp()
    }

    /// `hasRequiredMemories(owner)` — все entryCondition
    /// удовлетворены через `Brain.checkMemory`.
    fn has_required_memories(&self, mob: &Mob) -> bool {
        let Some(brain) = mob.living.brain.clone() else {
            return false;
        };
        self.entry_conditions().iter().all(|(ty, status)| {
            brain.check_memory(ty, *status)
        })
    }

    /// `tryStart(level, owner, gameTime)` — финальный метод:
    /// hasRequiredMemories + checkExtraStartConditions →
    /// status=RUNNING, endTimestamp=gameTime+duration, start().
    /// Paper tickRates опущены (как в Sensor).
    fn try_start(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) -> bool {
        if self.has_required_memories(mob) && self.check_extra_start_conditions(level, mob) {
            self.set_status(BehaviorStatus::Running);
            let (min_d, max_d) = self.durations();
            let duration = {
                let mut r = mob.random.borrow_mut();
                min_d + r.next_int_bound((max_d + 1 - min_d).max(1))
            };
            self.set_end_timestamp(game_time + duration as i64);
            self.start(level, mob, game_time);
            true
        } else {
            false
        }
    }

    /// `tickOrStop(level, entity, gameTime)` — финальный:
    /// !timedOut && canStillUse → tick, иначе doStop.
    fn tick_or_stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        if !self.timed_out(game_time) && self.can_still_use(level, mob, game_time) {
            self.tick(level, mob, game_time);
        } else {
            self.do_stop(level, mob, game_time);
        }
    }

    /// `doStop(level, entity, gameTime)` — финальный:
    /// status=STOPPED + stop().
    fn do_stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        self.set_status(BehaviorStatus::Stopped);
        self.stop(level, mob, game_time);
    }
}

/// Удобная база для наследников `Behavior`: хранит
/// entryCondition/status/durations/timestamp. Каждый behavior
/// содержит `BehaviorBase` и реализует `BehaviorControl`,
/// делегируя полевые методы базе.
pub struct BehaviorBase {
    /// `entryCondition`.
    pub entry_condition: HashMap<MemoryModuleType, MemoryStatus>,
    /// `status`.
    pub status: BehaviorStatus,
    /// `endTimestamp`.
    pub end_timestamp: i64,
    /// `minDuration`/`maxDuration`.
    pub min_duration: i32,
    pub max_duration: i32,
}

impl BehaviorBase {
    /// `new(entryCondition)` — duration=60.
    pub fn new(entry_condition: HashMap<MemoryModuleType, MemoryStatus>) -> Self {
        Self::with_duration(entry_condition, 60)
    }

    /// `new(entryCondition, duration)`.
    pub fn with_duration(entry_condition: HashMap<MemoryModuleType, MemoryStatus>, duration: i32) -> Self {
        Self {
            entry_condition,
            status: BehaviorStatus::Stopped,
            end_timestamp: 0,
            min_duration: duration,
            max_duration: duration,
        }
    }

    /// `new(entryCondition, minDuration, maxDuration)`.
    pub fn with_range(
        entry_condition: HashMap<MemoryModuleType, MemoryStatus>,
        min_duration: i32,
        max_duration: i32,
    ) -> Self {
        Self {
            entry_condition,
            status: BehaviorStatus::Stopped,
            end_timestamp: 0,
            min_duration,
            max_duration,
        }
    }

    /// Хелпер: entryCondition из списка (type,status) пар.
    pub fn conditions(pairs: &[(MemoryModuleType, MemoryStatus)]) -> HashMap<MemoryModuleType, MemoryStatus> {
        pairs.iter().cloned().collect()
    }
}

/// `ShufflingList<E>` — список с перемешиванием (GateBehavior,
/// random behavior selection). Порт Fisher-Yates на Vec.
pub struct ShufflingList<T> {
    items: Vec<T>,
}

impl<T> ShufflingList<T> {
    /// `new(Collection)`.
    pub fn new(items: Vec<T>) -> Self {
        Self { items }
    }

    /// `shuffle(RandomSource)` — Fisher-Yates.
    pub fn shuffle(&mut self, random: &mut dyn crate::random::RandomSource) {
        let n = self.items.len();
        for i in (1..n).rev() {
            let j = random.next_int_bound((i + 1) as i32) as usize;
            self.items.swap(i, j);
        }
    }

    /// `get(index)`.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    /// `size()`.
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// `isEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Доступ к внутреннему Vec.
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    /// Внутренний Vec (mut).
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.items
    }
}

/// Пара (priority, behavior) для `Brain.addActivity`.
pub type PriorityBehavior = (i32, Box<dyn BehaviorControl>);

/// `Pair<MemoryModuleType, MemoryStatus>` — требование activity.
pub type MemoryStatusPair = (MemoryModuleType, MemoryStatus);

/// Хелпер: `ImmutableSet.of(MemoryModuleType.X, MemoryStatus.Y)`-стиль.
pub fn mem_cond(
    ty: MemoryModuleType,
    status: MemoryStatus,
) -> MemoryStatusPair {
    (ty, status)
}

/// Собрать HashSet условий из массива пар.
pub fn cond_set(pairs: &[MemoryStatusPair]) -> HashSet<MemoryStatusPair> {
    pairs.iter().cloned().collect()
}
