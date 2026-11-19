//! Перенос `net.minecraft.world.entity.ai.goal.Goal`.
//!
//! `Goal` — абстрактная цель AI. Флаги (`Flag`) — наборы контролов,
//! которые цель занимает (MOVE/LOOK/JUMP/TARGET). `GoalSelector`
//! вытесняет цели с конфликтующими флагами по приоритету.

use crate::world::entity::mob::Mob;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// `Goal.Flag` — флаги контролов.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GoalFlag {
    /// `MOVE`.
    Move,
    /// `LOOK`.
    Look,
    /// `JUMP`.
    Jump,
    /// `TARGET`.
    Target,
}

/// `EnumSet<Goal.Flag>`.
#[derive(Debug, Clone, Default)]
pub struct GoalFlagSet(pub HashSet<GoalFlag>);

impl std::iter::FromIterator<GoalFlag> for GoalFlagSet {
    fn from_iter<I: IntoIterator<Item = GoalFlag>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl GoalFlagSet {
    /// Пустой набор.
    pub fn empty() -> Self {
        Self(HashSet::new())
    }
    /// Вставить флаг.
    pub fn insert(&mut self, flag: GoalFlag) -> bool {
        self.0.insert(flag)
    }
    /// Проверить наличие флага.
    pub fn contains(&self, flag: GoalFlag) -> bool {
        self.0.contains(&flag)
    }
    /// Пересечение с другим набором (`flagSet.intersects`).
    pub fn intersects(&self, other: &GoalFlagSet) -> bool {
        self.0.iter().any(|f| other.0.contains(f))
    }
    /// Объединение.
    pub fn union(&self, other: &GoalFlagSet) -> GoalFlagSet {
        let mut s = self.0.clone();
        s.extend(other.0.iter().copied());
        GoalFlagSet(s)
    }
}

/// `Goal` — цель AI (аналог abstract class). Без `Send + Sync`:
/// `Mob` сам однопоточный (`Rc<RefCell>` random/navigation),
/// цели не покидают поток владельца — как в Java.
pub trait Goal {
    /// `canUse()` — можно ли начать. `mob` — как `this.mob`
    /// в Java (цели получают моба параметром, т.к. не могут
    /// хранить ссылку на владеющий их GoalSelector моб).
    fn can_use(&mut self, mob: &mut Mob) -> bool;
    /// `canContinueToUse()` — продолжать ли.
    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.can_use(mob)
    }
    /// `isInterruptable()`.
    fn is_interruptable(&self) -> bool {
        true
    }
    /// `start()`.
    fn start(&mut self, _mob: &mut Mob) {}
    /// `stop()`.
    fn stop(&mut self, _mob: &mut Mob) {}
    /// `requiresUpdateEveryTick()`.
    fn requires_update_every_tick(&self) -> bool {
        false
    }
    /// `tick()`.
    fn tick(&mut self, _mob: &mut Mob) {}
    /// `setFlags(flags)`.
    fn set_flags(&mut self, _flags: GoalFlagSet) {}
    /// `getFlags()`.
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::empty()
    }
    /// `adjustedTickDelay(reduction)` — для целей не на каждый тик
    /// возвращает `reducedTickDelay`; переопределяется для
    /// `requiresUpdateEveryTick == true`.
    fn adjusted_tick_delay(&self, adjustment: i32) -> i32 {
        if self.requires_update_every_tick() {
            adjustment
        } else {
            crate::util::mth::positive_ceil_div(adjustment, 2)
        }
    }
    /// `reducedTickDelay(reduction)` — `ceilDiv(reduction, 2)`.
    fn reduced_tick_delay_static(reduction: i32) -> i32
    where
        Self: Sized,
    {
        crate::util::mth::positive_ceil_div(reduction, 2)
    }
}

/// `WrappedGoal` — обёртка цели с приоритетом.
pub struct WrappedGoal {
    /// `goal`.
    pub goal: Arc<Mutex<dyn Goal>>,
    /// `priority`.
    pub priority: i32,
    /// `isRunning`.
    pub is_running: bool,
}

impl WrappedGoal {
    /// `new(priority, goal)`.
    pub fn new(priority: i32, goal: Arc<Mutex<dyn Goal>>) -> Self {
        Self { goal, priority, is_running: false }
    }

    /// `canBeReplacedBy(other)` — `this.isInterruptable() &&
    /// other.priority < this.priority`.
    pub fn can_be_replaced_by(&self, other_priority: i32, other_interruptable: bool) -> bool {
        // Java: this.isInterruptable() && other.getPriority() < this.getPriority()
        // `other` здесь — новый WrappedGoal.
        let _ = other_interruptable;
        self.is_interruptable() && other_priority < self.priority
    }

    pub fn is_interruptable(&self) -> bool {
        self.goal.lock().unwrap().is_interruptable()
    }

    /// `canUse()`.
    pub fn can_use(&self, mob: &mut Mob) -> bool {
        self.goal.lock().unwrap().can_use(mob)
    }
    /// `canContinueToUse()`.
    pub fn can_continue_to_use(&self, mob: &mut Mob) -> bool {
        self.goal.lock().unwrap().can_continue_to_use(mob)
    }
    /// `isInterruptable()`.
    pub fn is_interruptable_pub(&self) -> bool {
        self.is_interruptable()
    }
    /// `start()`.
    pub fn start(&mut self, mob: &mut Mob) {
        if !self.is_running {
            self.is_running = true;
            self.goal.lock().unwrap().start(mob);
        }
    }
    /// `stop()`.
    pub fn stop(&mut self, mob: &mut Mob) {
        if self.is_running {
            self.is_running = false;
            self.goal.lock().unwrap().stop(mob);
        }
    }
    /// `requiresUpdateEveryTick()`.
    pub fn requires_update_every_tick(&self) -> bool {
        self.goal.lock().unwrap().requires_update_every_tick()
    }
    /// `adjustedTickDelay`.
    pub fn adjusted_tick_delay(&self, adjustment: i32) -> i32 {
        self.goal.lock().unwrap().adjusted_tick_delay(adjustment)
    }
    /// `tick()`.
    pub fn tick(&self, mob: &mut Mob) {
        self.goal.lock().unwrap().tick(mob);
    }
    /// `setFlags`.
    pub fn set_flags(&self, flags: GoalFlagSet) {
        self.goal.lock().unwrap().set_flags(flags);
    }
    /// `getFlags`.
    pub fn get_flags(&self) -> GoalFlagSet {
        self.goal.lock().unwrap().get_flags()
    }
    /// `isRunning()`.
    pub fn is_running(&self) -> bool {
        self.is_running
    }
    /// `getPriority()`.
    pub fn get_priority(&self) -> i32 {
        self.priority
    }
    /// `getGoal()`.
    pub fn get_goal(&self) -> &Arc<Mutex<dyn Goal>> {
        &self.goal
    }
}

/// Сравнение WrappedGoal по идентичности `goal` (как `equals` в Java).
impl PartialEq for WrappedGoal {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.goal, &other.goal)
    }
}
impl Eq for WrappedGoal {}
impl std::hash::Hash for WrappedGoal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.goal).hash(state);
    }
}

/// NO_GOAL — `new WrappedGoal(Integer.MAX_VALUE, goal canUse=false)`
/// с `isRunning() == false` всегда.
pub fn no_goal() -> WrappedGoal {
    struct NoGoalImpl;
    impl Goal for NoGoalImpl {
        fn can_use(&mut self, _mob: &mut crate::world::entity::mob::Mob) -> bool {
            false
        }
    }
    WrappedGoal {
        goal: Arc::new(Mutex::new(NoGoalImpl)),
        priority: i32::MAX,
        is_running: false,
    }
}

/// Доступ к Arc<Mutex<dyn Goal>> для построения целей.
pub type DynGoal = Arc<Mutex<dyn Goal>>;

/// Удобный конструктор: `Arc::new(Mutex::new(goal))`.
pub fn dyn_goal<G: Goal + 'static>(goal: G) -> DynGoal {
    Arc::new(Mutex::new(goal))
}

#[cfg(test)]
mod tests {
    use crate::world::entity::mob::Mob;
use super::*;
    fn test_mob() -> crate::world::entity::mob::Mob {
        let living = crate::world::entity::living_entity::LivingEntity::new(
            crate::core::holder::Holder::Direct(
                crate::world::entity::entity_type::slime_entity_type(),
            ),
            None,
        );
        crate::world::entity::mob::Mob::new(living)
    }

    struct TestGoal {
        can: bool,
        flags: GoalFlagSet,
    }
    impl Goal for TestGoal {
        fn can_use(&mut self, _mob: &mut crate::world::entity::mob::Mob) -> bool {
            self.can
        }
        fn get_flags(&self) -> GoalFlagSet {
            self.flags.clone()
        }
    }

    #[test]
    fn goal_flag_set() {
        let mut s = GoalFlagSet::empty();
        s.insert(GoalFlag::Move);
        s.insert(GoalFlag::Look);
        assert!(s.contains(GoalFlag::Move));
        assert!(!s.contains(GoalFlag::Jump));
        let o = GoalFlagSet::empty();
        assert!(!s.intersects(&o));
    }

    #[test]
    fn wrapped_goal_lifecycle() {
        let mut wg = WrappedGoal::new(
            1,
            dyn_goal(TestGoal { can: true, flags: GoalFlagSet::empty() }),
        );
        assert!(!wg.is_running());
        let mut mob = test_mob();
        wg.start(&mut mob);
        assert!(wg.is_running());
        wg.stop(&mut mob);
        assert!(!wg.is_running());
    }

    #[test]
    fn reduced_tick_delay() {
        assert_eq!(crate::util::mth::positive_ceil_div(10, 2), 5);
        assert_eq!(crate::util::mth::positive_ceil_div(11, 2), 6);
    }
}
