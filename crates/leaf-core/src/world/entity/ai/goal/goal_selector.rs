//! Перенос `net.minecraft.world.entity.ai.goal.GoalSelector`.
//!
//! Хранит `availableGoals` (упорядоченный набор WrappedGoal),
//! `lockedFlags` (флаг → занявшая его цель) и `disabledFlags`.
//! `tick()` — cleanup → locked cleanup → update → tickRunningGoals.

use super::goal::{no_goal, DynGoal, Goal, GoalFlag, GoalFlagSet, WrappedGoal};
use crate::world::entity::mob::Mob;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// `GoalSelector`.
#[derive(Default)]
pub struct GoalSelector {
    /// `lockedFlags`.
    locked_flags: HashMap<GoalFlag, Arc<Mutex<WrappedGoal>>>,
    /// `availableGoals` — `ObjectLinkedOpenHashSet` (порядок вставки,
    /// без дубликатов по идентичности `goal`).
    available_goals: Vec<Arc<Mutex<WrappedGoal>>>,
    /// `disabledFlags`.
    disabled_flags: HashSet<GoalFlag>,
    /// Множество Arc-указателей для O(1) проверки дубликатов.
    goal_ptrs: HashSet<usize>,
}

impl GoalSelector {
    /// `new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `addGoal(priority, goal)`.
    pub fn add_goal(&mut self, priority: i32, goal: DynGoal) {
        let wg = Arc::new(Mutex::new(WrappedGoal::new(priority, goal)));
        let ptr = Arc::as_ptr(&wg) as usize;
        if self.goal_ptrs.insert(ptr) {
            self.available_goals.push(wg);
        }
    }

    /// `removeAllGoals(filter)` — удалить цели, для которых
    /// `filter(goal)` истинно. Останавливает бегущие.
    pub fn remove_all_goals<F: Fn(&dyn Goal) -> bool>(&mut self, filter: F, mob: &mut Mob) {
        let mut i = 0;
        while i < self.available_goals.len() {
            let should_remove = {
                let arc = self.available_goals[i].clone();
                let wg = arc.lock().unwrap();
                let inner = wg.goal.lock().unwrap();
                wg.is_running() && filter(&*inner)
            };
            if should_remove {
                self.available_goals[i].lock().unwrap().stop(mob);
            }
            if {
                let arc = self.available_goals[i].clone();
                let wg = arc.lock().unwrap();
                let inner = wg.goal.lock().unwrap();
                wg.is_running() && filter(&*inner)
            } {
                let removed = self.available_goals.remove(i);
                self.goal_ptrs.remove(&(Arc::as_ptr(&removed) as usize));
            } else {
                i += 1;
            }
        }
        // lockedFlags указывают на те же Arc — остановленные цели
        // будут вычищены в tick().
    }

    /// `removeGoal(goal)` — удалить конкретную цель (по Arc-идентичности
    /// внутреннего goal нельзя сравнить через dyn — сравниваем по ptr
    /// Arc<Mutex<dyn Goal>>).
    pub fn remove_goal(&mut self, goal_ptr: usize, mob: &mut Mob) {
        let mut i = 0;
        while i < self.available_goals.len() {
            let arc = self.available_goals[i].clone();
            let arc_ptr = Arc::as_ptr(&arc) as usize;
            let matches = {
                let mut wg = arc.lock().unwrap();
                let same = arc_ptr == goal_ptr;
                if same && wg.is_running() {
                    wg.stop(mob);
                }
                same
            };
            if matches {
                let removed = self.available_goals.remove(i);
                self.goal_ptrs.remove(&(Arc::as_ptr(&removed) as usize));
            } else {
                i += 1;
            }
        }
    }

    /// `tick()` — полный цикл GoalSelector.
    pub fn tick(&mut self, mob: &mut Mob) {
        // 1. goalCleanup: running goals с disabled-флагами или
        //    !canContinueToUse → stop.
        for wg_arc in &self.available_goals {
            let mut wg = wg_arc.lock().unwrap();
            if wg.is_running() {
                let flags = wg.get_flags();
                let disabled_hit = flags.intersects(&GoalFlagSet(self.disabled_flags.clone()));
                let cant_continue = !wg.can_continue_to_use(mob);
                if disabled_hit || cant_continue {
                    wg.stop(mob);
                }
            }
        }

        // 2. lockedFlags cleanup: убрать не-running.
        self.locked_flags
            .retain(|_, wg| wg.lock().unwrap().is_running());

        // 3. goalUpdate: для каждой не-running цели без disabled
        //    флагов, заменяющей все locked, и canUse → lock + start.
        for wg_arc in self.available_goals.clone() {
            // ВАЖНО: guard WrappedGoal нельзя держать во время
            // goal_can_be_replaced_for_all_flags — она локает тот же
            // Mutex (deadlock, std::sync::Mutex не реентерабелен).
            let (is_running, flags) = {
                let wg = wg_arc.lock().unwrap();
                if wg.is_running() {
                    (true, GoalFlagSet::empty())
                } else {
                    (false, wg.get_flags())
                }
            };
            if is_running {
                continue;
            }
            let disabled_hit = flags.intersects(&GoalFlagSet(self.disabled_flags.clone()));
            let can_replace = self.goal_can_be_replaced_for_all_flags(&flags, &wg_arc);
            let can_use_ok = !disabled_hit
                && can_replace
                && wg_arc.lock().unwrap().can_use(mob);
            if can_use_ok {
                // Остановить и заменить locked-цели по каждому флагу.
                for flag in flags.0.iter().copied() {
                    if let Some(old) = self.locked_flags.get(&flag) {
                        old.lock().unwrap().stop(mob);
                    }
                    self.locked_flags.insert(flag, wg_arc.clone());
                }
                wg_arc.lock().unwrap().start(mob);
            }
        }

        // 4. tickRunningGoals(true).
        self.tick_running_goals(true, mob);
    }

    /// `goalCanBeReplacedForAllFlags(goal, flagMap)` — для каждого флага
    /// цели locked-цель должна быть заменяема: `locked.isInterruptable()
    /// && candidate.priority < locked.priority`.
    fn goal_can_be_replaced_for_all_flags(
        &self,
        goal_flags: &GoalFlagSet,
        candidate: &Arc<Mutex<WrappedGoal>>,
    ) -> bool {
        let cand_priority = candidate.lock().unwrap().priority;
        for flag in goal_flags.0.iter() {
            let ok = match self.locked_flags.get(flag) {
                Some(locked) => {
                    let l = locked.lock().unwrap();
                    l.is_interruptable() && cand_priority < l.priority
                }
                None => true, // NO_GOAL имеет priority MAX_VALUE
            };
            if !ok {
                return false;
            }
        }
        true
    }

    /// `tickRunningGoals(tickAllRunning)`.
    pub fn tick_running_goals(&self, tick_all_running: bool, mob: &mut Mob) {
        for wg_arc in &self.available_goals {
            let wg = wg_arc.lock().unwrap();
            if wg.is_running() && (tick_all_running || wg.requires_update_every_tick()) {
                wg.tick(mob);
            }
        }
    }

    /// `getAvailableGoals()`.
    pub fn available_goals(&self) -> &[Arc<Mutex<WrappedGoal>>] {
        &self.available_goals
    }

    /// `disableControlFlag(flag)`.
    pub fn disable_control_flag(&mut self, flag: GoalFlag) {
        self.disabled_flags.insert(flag);
    }

    /// `enableControlFlag(flag)`.
    pub fn enable_control_flag(&mut self, flag: GoalFlag) {
        self.disabled_flags.remove(&flag);
    }

    /// `setControlFlag(flag, enabled)`.
    pub fn set_control_flag(&mut self, flag: GoalFlag, enabled: bool) {
        if enabled {
            self.enable_control_flag(flag);
        } else {
            self.disable_control_flag(flag);
        }
    }
}



/// Хелпер: получить `GoalFlagSet` из `HashSet<GoalFlag>` для
/// intersects-проверок (disabled_flags хранится как HashSet).
impl GoalFlagSet {
    /// Из HashSet.
    pub fn from_set(set: HashSet<GoalFlag>) -> Self {
        Self(set)
    }
}

/// `GoalSelector` не должен требовать `Debug` у целей.
impl std::fmt::Debug for GoalSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoalSelector")
            .field("available", &self.available_goals.len())
            .field("locked", &self.locked_flags.len())
            .field("disabled", &self.disabled_flags)
            .finish()
    }
}

// Подавить warning о неиспользуемом no_goal, пока он не нужен публично.
#[allow(dead_code)]
fn _unused() {
    let _ = no_goal();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    fn test_mob() -> crate::world::entity::mob::Mob {
        let living = crate::world::entity::living_entity::LivingEntity::new(
            crate::core::holder::Holder::Direct(
                crate::world::entity::entity_type::slime_entity_type(),
            ),
            None,
        );
        crate::world::entity::mob::Mob::new(living)
    }


    struct FlagGoal {
        flags: GoalFlagSet,
        can: AtomicBool,
        started: AtomicBool,
        stopped: AtomicBool,
    }
    impl Goal for FlagGoal {
        fn can_use(&mut self, _mob: &mut crate::world::entity::mob::Mob) -> bool {
            self.can.load(Ordering::SeqCst)
        }
        fn start(&mut self, _mob: &mut crate::world::entity::mob::Mob) {
            self.started.store(true, Ordering::SeqCst);
        }
        fn stop(&mut self, _mob: &mut crate::world::entity::mob::Mob) {
            self.stopped.store(true, Ordering::SeqCst);
        }
        fn get_flags(&self) -> GoalFlagSet {
            self.flags.clone()
        }
    }

    fn mk(flags: &[GoalFlag], can: bool) -> DynGoal {
        let mut s = GoalFlagSet::empty();
        for f in flags {
            s.insert(*f);
        }
        Arc::new(Mutex::new(FlagGoal {
            flags: s,
            can: AtomicBool::new(can),
            started: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
        }))
    }

    #[test]
    fn selector_starts_and_ticks() {
        let mut sel = GoalSelector::new();
        let g = mk(&[GoalFlag::Move], true);
        sel.add_goal(0, g.clone());
        let mut mob = test_mob();
        sel.tick(&mut mob);
        assert!(sel.available_goals()[0].lock().unwrap().is_running());
    }

    #[test]
    fn disabled_flag_blocks_goal() {
        let mut sel = GoalSelector::new();
        let g = mk(&[GoalFlag::Move], true);
        sel.add_goal(0, g);
        sel.disable_control_flag(GoalFlag::Move);
        let mut mob = test_mob();
        sel.tick(&mut mob);
        assert!(!sel.available_goals()[0].lock().unwrap().is_running());
    }
}
