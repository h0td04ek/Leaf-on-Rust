//! Перенос `GateBehavior`, `RunOne`, `TriggerGate` —
//! контейнеры поведений с weighted ShufflingList.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use crate::world::entity::ai::behavior::behavior::{
    BehaviorControl, BehaviorStatus, ShufflingList,
};
use crate::world::entity::ai::behavior::simple::{conds, PointBehavior};
use crate::world::entity::ai::memory::memory::{MemoryModuleType, MemoryStatus};
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;

/// `GateBehavior.OrderPolicy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderPolicy {
    /// `ORDERED` — без перемешивания.
    Ordered,
    /// `SHUFFLED` — weighted shuffle.
    Shuffled,
}

/// `GateBehavior.RunningPolicy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunningPolicy {
    /// `RUN_ONE` — запустить первое подошедшее.
    RunOne,
    /// `TRY_ALL` — запустить все.
    TryAll,
}

/// `GateBehavior<E>` — gating-контейнер поведений.
pub struct GateBehavior {
    /// `entryCondition`.
    pub entry_condition: HashMap<MemoryModuleType, MemoryStatus>,
    /// `exitErasedMemories`.
    pub exit_erased_memories: HashSet<MemoryModuleType>,
    /// `orderPolicy`/`runningPolicy`.
    order_policy: OrderPolicy,
    running_policy: RunningPolicy,
    /// `behaviors` — weighted ShufflingList.
    behaviors: ShufflingList<Box<dyn BehaviorControl>>,
    /// `status`.
    status: BehaviorStatus,
}

impl GateBehavior {
    /// `new(entryCondition, exitErasedMemories, orderPolicy,
    /// runningPolicy, durations)`.
    /// `durations` — Vec<(behavior, weight)>.
    pub fn new(
        entry_condition: HashMap<MemoryModuleType, MemoryStatus>,
        exit_erased_memories: HashSet<MemoryModuleType>,
        order_policy: OrderPolicy,
        running_policy: RunningPolicy,
        durations: Vec<(Box<dyn BehaviorControl>, i32)>,
    ) -> Self {
        let mut behaviors = ShufflingList::new();
        for (b, w) in durations {
            behaviors.add(b, w);
        }
        Self {
            entry_condition,
            exit_erased_memories,
            order_policy,
            running_policy,
            behaviors,
            status: BehaviorStatus::Stopped,
        }
    }

    /// Доступ к children (для отладки/регистрации).
    pub fn behaviors_mut(&mut self) -> &mut ShufflingList<Box<dyn BehaviorControl>> {
        &mut self.behaviors
    }
}

impl BehaviorControl for GateBehavior {
    fn status(&self) -> BehaviorStatus {
        self.status
    }
    fn set_status(&mut self, status: BehaviorStatus) {
        self.status = status;
    }
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
        &self.entry_condition
    }
    fn durations(&self) -> (i32, i32) {
        (0, 0)
    }
    fn end_timestamp(&self) -> i64 {
        0
    }
    fn set_end_timestamp(&mut self, _timestamp: i64) {}

    /// `tryStart` — финальный: hasRequiredMemories → RUNNING,
    /// orderPolicy.apply, runningPolicy.apply.
    fn try_start(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) -> bool {
        if !self.has_required_memories(mob) {
            return false;
        }
        self.status = BehaviorStatus::Running;
        if self.order_policy == OrderPolicy::Shuffled {
            self.behaviors.shuffle();
        }
        // RUN_ONE: первый STOPPED child с успешным tryStart.
        // TRY_ALL: tryStart всех STOPPED.
        for entry in self.behaviors.entries_mut() {
            let b = &mut entry.data;
            if b.status() != BehaviorStatus::Stopped {
                continue;
            }
            if b.try_start(level, mob, game_time) {
                if self.running_policy == RunningPolicy::RunOne {
                    break;
                }
            }
        }
        true
    }

    /// `tickOrStop` — финальный: тикать RUNNING children;
    /// ни одного RUNNING → doStop.
    fn tick_or_stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        for entry in self.behaviors.entries_mut() {
            let b = &mut entry.data;
            if b.status() == BehaviorStatus::Running {
                b.tick_or_stop(level, mob, game_time);
            }
        }
        let any_running = self
            .behaviors
            .iter()
            .any(|b| b.status() == BehaviorStatus::Running);
        if !any_running {
            self.do_stop(level, mob, game_time);
        }
    }

    /// `doStop` — финальный: STOPPED, stop RUNNING children,
    /// erase exitErasedMemories.
    fn do_stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        self.status = BehaviorStatus::Stopped;
        for entry in self.behaviors.entries_mut() {
            let b = &mut entry.data;
            if b.status() == BehaviorStatus::Running {
                b.do_stop(level, mob, game_time);
            }
        }
        for ty in &self.exit_erased_memories {
            mob.living.brain_erase(ty);
        }
    }

    fn debug_string(&self) -> String {
        "GateBehavior".to_string()
    }
}

/// `RunOne.create(entryCondition, durations)` —
/// GateBehavior(SHUFFLED, RUN_ONE, без exit-стирания).
pub fn run_one(
    entry_condition: HashMap<MemoryModuleType, MemoryStatus>,
    durations: Vec<(Box<dyn BehaviorControl>, i32)>,
) -> GateBehavior {
    GateBehavior::new(
        entry_condition,
        HashSet::new(),
        OrderPolicy::Shuffled,
        RunningPolicy::RunOne,
        durations,
    )
}

/// `Trigger<E>` для TriggerGate.
pub type GateTrigger = Rc<RefCell<dyn FnMut(&Arc<dyn LevelReader>, &mut Mob, i64) -> bool>>;

/// `TriggerGate.triggerGate(triggers, orderPolicy, runningPolicy)` —
/// OneShot, который при tryStart прогоняет триггеры:
/// SHUFFLED → перемешать, RUN_ONE → стоп на первом true.
pub fn trigger_gate(
    triggers: Vec<(GateTrigger, i32)>,
    order_policy: OrderPolicy,
    running_policy: RunningPolicy,
) -> PointBehavior {
    // Weighted-список триггеров с собственным random (как ShufflingList).
    let list = Rc::new(RefCell::new(ShufflingList::new()));
    {
        let mut l = list.borrow_mut();
        for (t, w) in triggers {
            l.add(t, w);
        }
    }
    PointBehavior::point(move |level, mob, game_time| {
        let mut l = list.borrow_mut();
        if order_policy == OrderPolicy::Shuffled {
            l.shuffle();
        }
        // Прогон: RUN_ONE → break на первом true; TRY_ALL → все.
        // Итог всегда true (Java возвращает true после цикла).
        for entry in l.entries_mut() {
            let fired = (entry.data.borrow_mut())(level, mob, game_time);
            if fired && running_policy == RunningPolicy::RunOne {
                break;
            }
        }
        true
    })
}

/// `TriggerGate.triggerOneShuffled(triggers)`.
pub fn trigger_one_shuffled(triggers: Vec<(GateTrigger, i32)>) -> PointBehavior {
    trigger_gate(triggers, OrderPolicy::Shuffled, RunningPolicy::RunOne)
}
