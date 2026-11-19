//! Перенос `net.minecraft.world.entity.ai.Brain` — центральный
//! координатор AI мобов: memories (ExpirableValue), sensors,
//! activities + behaviors по приоритетам.
//!
//! Java `Brain<E extends LivingEntity>`; у нас `E` = `Mob`.
//! Все поля — `RefCell`/`Cell` (`&self` методы), т.к. Brain
//! тикается внутри `&mut Mob`, а sensors/behaviors реентерабельно
//! обращаются к `mob.living.brain`.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use crate::entity::schedule::activity::{activity_by_name, Activity};
use crate::world::entity::ai::behavior::behavior::{BehaviorControl, BehaviorStatus, MemoryStatusPair};
use crate::world::entity::ai::memory::memory::{
    ExpirableValue, MemoryModuleType, MemoryStatus,
};
use crate::world::entity::ai::sensing::sensor::Sensor;
use crate::world::entity::ai::sensing::sensor_type::SensorType;
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;

/// `EnvironmentAttribute<Activity>`-эквивалент для
/// `Brain.schedule` — расписание activity по позиции/времени.
/// До порта `EnvironmentAttributeSystem` храним как трейт-объект.
pub trait ActivitySchedule: Send {
    /// `getValue(schedule, pos)` — activity в позиции.
    fn activity_at(&self, level: &dyn LevelReader, pos: Vec3) -> Arc<Activity>;
}

/// `Brain` — см. Java `Brain`.
pub struct Brain {
    /// `memories` — Map<MemoryModuleType, Optional<ExpirableValue>>.
    /// `None` = `Optional.empty()`, `Some(None)`-аналог не нужен:
    /// отсутствие ключа = unregistered (getMemory → panic как в Java).
    memories: RefCell<HashMap<MemoryModuleType, Option<ExpirableValue>>>,
    /// `sensors` — SensorType → Sensor (по одному на тип).
    sensors: RefCell<Vec<(SensorType, RefCell<Box<dyn Sensor>>)>>,
    /// `availableBehaviorsByPriority` — priority → activity → set.
    /// Порядок Vec = порядок добавления (Java использует Set —
    /// дедуп по поведению не критичен, эквивалентные экземпляры
    /// уникальны по задаче).
    available_behaviors: RefCell<
        BTreeMap<i32, HashMap<Arc<Activity>, Vec<RefCell<Box<dyn BehaviorControl>>>>>,
    >,
    /// `activityRequirements` — activity → Set<Pair<type,status>>.
    activity_requirements: RefCell<HashMap<Arc<Activity>, HashSet<MemoryStatusPair>>>,
    /// `activityMemoriesToEraseWhenStopped`.
    activity_memories_to_erase: RefCell<HashMap<Arc<Activity>, HashSet<MemoryModuleType>>>,
    /// `coreActivities`.
    core_activities: RefCell<HashSet<Arc<Activity>>>,
    /// `activeActivities`.
    active_activities: RefCell<HashSet<Arc<Activity>>>,
    /// `defaultActivity`.
    default_activity: RefCell<Arc<Activity>>,
    /// `schedule`.
    schedule: RefCell<Option<Rc<dyn ActivitySchedule>>>,
    /// `lastScheduleUpdate`.
    last_schedule_update: Cell<i64>,
}

/// `SCHEDULE_UPDATE_DELAY`.
const SCHEDULE_UPDATE_DELAY: i64 = 20;

impl Brain {
    /// `new Brain(memoryTypes, sensorTypes, memoryValues)` —
    /// memories инициализируются всеми типами (пустые),
    /// сенсоры создаются фабрикой.
    pub fn new(
        memory_types: Vec<MemoryModuleType>,
        sensor_types: Vec<SensorType>,
    ) -> Self {
        let mut memories = HashMap::new();
        for ty in memory_types {
            memories.insert(ty, None);
        }
        let mut sensors = Vec::new();
        for st in sensor_types {
            if let Some(sensor) = crate::world::entity::ai::sensing::sensor_type::create_sensor(st.clone()) {
                sensors.push((st, RefCell::new(sensor)));
            }
        }
        // `sensor.requires()` — дополнительные memory-типы.
        for (_, s) in &sensors {
            for req in s.borrow().requires() {
                memories.entry(req).or_insert(None);
            }
        }
        Self {
            memories: RefCell::new(memories),
            sensors: RefCell::new(sensors),
            available_behaviors: RefCell::new(BTreeMap::new()),
            activity_requirements: RefCell::new(HashMap::new()),
            activity_memories_to_erase: RefCell::new(HashMap::new()),
            core_activities: RefCell::new(HashSet::new()),
            active_activities: RefCell::new(HashSet::new()),
            default_activity: RefCell::new(idle_activity()),
            schedule: RefCell::new(None),
            last_schedule_update: Cell::new(-9999),
        }
    }

    /// `Brain.provider(memoryTypes, sensorTypes)` — конструктор
    /// для `LivingEntity`-сабклассов; возвращает готовый Brain.
    pub fn provider(
        memory_types: Vec<MemoryModuleType>,
        sensor_types: Vec<SensorType>,
    ) -> Rc<Brain> {
        Rc::new(Self::new(memory_types, sensor_types))
    }

    // ---------- memories ----------

    /// `hasMemoryValue(type)` — VALUE_PRESENT.
    pub fn has_memory_value(&self, ty: &MemoryModuleType) -> bool {
        self.check_memory(ty, MemoryStatus::ValuePresent)
    }

    /// `checkMemory(type, status)`.
    pub fn check_memory(&self, ty: &MemoryModuleType, status: MemoryStatus) -> bool {
        let memories = self.memories.borrow();
        match memories.get(ty) {
            None => false,
            Some(opt) => match status {
                MemoryStatus::Registered => true,
                MemoryStatus::ValuePresent => opt.is_some(),
                MemoryStatus::ValueAbsent => opt.is_none(),
            },
        }
    }

    /// `setMemoryInternal` — пустые коллекции → erase.
    fn set_memory_internal(&self, ty: &MemoryModuleType, value: Option<ExpirableValue>) {
        let mut memories = self.memories.borrow_mut();
        if !memories.contains_key(ty) {
            return;
        }
        let is_empty_collection = value
            .as_ref()
            .map(|v| v.with_value(|inner| is_empty_collection_value(inner)))
            .unwrap_or(false);
        if is_empty_collection {
            drop(memories);
            self.erase_memory(ty);
        } else {
            memories.insert(ty.clone(), value);
        }
    }

    /// `setMemory(type, value)` — `ExpirableValue.of(value)`.
    pub fn set_memory<T: 'static>(&self, ty: &MemoryModuleType, value: Option<T>) {
        let ev = value.map(|v| ExpirableValue::of(v));
        self.set_memory_internal(ty, ev);
    }

    /// `setMemory(type, memory)` — не-Option перегрузка.
    pub fn set_memory_value<T: 'static>(&self, ty: &MemoryModuleType, value: T) {
        self.set_memory(ty, Some(value));
    }

    /// `setMemoryWithExpiry(type, memory, ttl)`.
    pub fn set_memory_with_expiry<T: 'static>(&self, ty: &MemoryModuleType, value: T, ttl: i64) {
        self.set_memory_internal(ty, Some(ExpirableValue::of_ttl(value, ttl)));
    }

    /// `eraseMemory(type)`.
    pub fn erase_memory(&self, ty: &MemoryModuleType) {
        self.set_memory_internal(ty, None);
    }

    /// `getMemory(type)` — клон значения типа T.
    /// Возвращает None если память отсутствует/пуста/тип не совпал.
    /// (Java бросает на unregistered — у нас None.)
    pub fn get_memory<T: Clone + 'static>(&self, ty: &MemoryModuleType) -> Option<T> {
        self.memories
            .borrow()
            .get(ty)
            .and_then(|o| o.as_ref())
            .and_then(|ev| ev.get_value::<T>())
    }

    /// `getMemory` через callback без клона (для больших значений).
    pub fn with_memory<R>(
        &self,
        ty: &MemoryModuleType,
        f: impl FnOnce(&dyn std::any::Any) -> R,
    ) -> Option<R> {
        self.memories
            .borrow()
            .get(ty)
            .and_then(|o| o.as_ref())
            .map(|ev| ev.with_value(f))
    }

    /// `getTimeUntilExpiry(type)`.
    pub fn get_time_until_expiry(&self, ty: &MemoryModuleType) -> i64 {
        self.memories
            .borrow()
            .get(ty)
            .and_then(|o| o.as_ref())
            .map(|ev| ev.get_time_to_live())
            .unwrap_or(0)
    }

    /// `isMemoryValue(type, value)` — для entity-значений
    /// сравнение по UUID (`LivingEntity`/`Entity`).
    pub fn is_memory_value_entity(&self, ty: &MemoryModuleType, entity: &crate::world::entity::living_entity::LivingEntity) -> bool {
        if !self.has_memory_value(ty) {
            return false;
        }
        self.with_memory(ty, |v| {
            if let Some(le) = v.downcast_ref::<crate::world::entity::living_entity::LivingEntity>() {
                le.get_uuid() == entity.get_uuid()
            } else if let Some(e) = v.downcast_ref::<crate::world::entity::entity::Entity>() {
                e.uuid == entity.entity.uuid
            } else {
                false
            }
        })
        .unwrap_or(false)
    }

    /// `clearMemories()`.
    pub fn clear_memories(&self) {
        let mut memories = self.memories.borrow_mut();
        for v in memories.values_mut() {
            *v = None;
        }
        drop(memories);
        self.available_behaviors.borrow_mut().clear();
    }

    // ---------- activities ----------

    /// `setSchedule(schedule)`.
    pub fn set_schedule(&self, schedule: Option<Rc<dyn ActivitySchedule>>) {
        *self.schedule.borrow_mut() = schedule;
    }

    /// `setCoreActivities(activities)`.
    pub fn set_core_activities(&self, activities: HashSet<Arc<Activity>>) {
        *self.core_activities.borrow_mut() = activities;
    }

    /// `getActiveActivities()` — @VisibleForDebug.
    pub fn get_active_activities(&self) -> HashSet<Arc<Activity>> {
        self.active_activities.borrow().clone()
    }

    /// `getActiveNonCoreActivity()`.
    pub fn get_active_non_core_activity(&self) -> Option<Arc<Activity>> {
        let core = self.core_activities.borrow();
        self.active_activities
            .borrow()
            .iter()
            .find(|a| !core.contains(*a))
            .cloned()
    }

    /// `useDefaultActivity()`.
    pub fn use_default_activity(&self) {
        let d = self.default_activity.borrow().clone();
        self.set_active_activity(d);
    }

    /// `setActiveActivityIfPossible(activity)`.
    pub fn set_active_activity_if_possible(&self, activity: Arc<Activity>) {
        if self.activity_requirements_are_met(&activity) {
            self.set_active_activity(activity);
        } else {
            self.use_default_activity();
        }
    }

    fn set_active_activity(&self, activity: Arc<Activity>) {
        if !self.is_active(&activity) {
            self.erase_memories_for_other_activities_than(&activity);
            let mut active = self.active_activities.borrow_mut();
            active.clear();
            active.extend(self.core_activities.borrow().iter().cloned());
            active.insert(activity);
        }
    }

    fn erase_memories_for_other_activities_than(&self, activity: &Arc<Activity>) {
        let to_erase: Vec<MemoryModuleType> = {
            let active = self.active_activities.borrow();
            let erase_map = self.activity_memories_to_erase.borrow();
            active
                .iter()
                .filter(|a| *a != activity)
                .flat_map(|a| erase_map.get(a).cloned().unwrap_or_default())
                .collect()
        };
        for ty in to_erase {
            self.erase_memory(&ty);
        }
    }

    /// `updateActivityFromSchedule(envAttrs, dayTime, pos)`.
    /// `envAttrs` не нужен — schedule уже ActivitySchedule.
    pub fn update_activity_from_schedule(&self, day_time: i64, pos: Vec3, level: &dyn LevelReader) {
        if day_time - self.last_schedule_update.get() > SCHEDULE_UPDATE_DELAY {
            self.last_schedule_update.set(day_time);
            let activity = self
                .schedule
                .borrow()
                .as_ref()
                .map(|s| s.activity_at(level, pos))
                .unwrap_or_else(|| idle_activity());
            if !self.active_activities.borrow().contains(&activity) {
                self.set_active_activity_if_possible(activity);
            }
        }
    }

    /// `setActiveActivityToFirstValid(activities)`.
    pub fn set_active_activity_to_first_valid(&self, activities: &[Arc<Activity>]) {
        for activity in activities {
            if self.activity_requirements_are_met(activity) {
                self.set_active_activity(activity.clone());
                break;
            }
        }
    }

    /// `setDefaultActivity(activity)`.
    pub fn set_default_activity(&self, activity: Arc<Activity>) {
        *self.default_activity.borrow_mut() = activity;
    }

    // ---------- addActivity ----------

    /// `createPriorityPairs(priorityStart, tasks)`.
    fn create_priority_pairs(
        priority_start: i32,
        tasks: Vec<Box<dyn BehaviorControl>>,
    ) -> Vec<(i32, Box<dyn BehaviorControl>)> {
        tasks
            .into_iter()
            .enumerate()
            .map(|(i, b)| (priority_start + i as i32, b))
            .collect()
    }

    /// `addActivity(activity, priorityStart, tasks)`.
    pub fn add_activity(
        &self,
        activity: Arc<Activity>,
        priority_start: i32,
        tasks: Vec<Box<dyn BehaviorControl>>,
    ) {
        let pairs = Self::create_priority_pairs(priority_start, tasks);
        self.add_activity_pairs(activity, pairs, HashSet::new(), HashSet::new());
    }

    /// `addActivityAndRemoveMemoryWhenStopped(activity,
    /// priorityStart, tasks, memoryType)`.
    pub fn add_activity_and_remove_memory_when_stopped(
        &self,
        activity: Arc<Activity>,
        priority_start: i32,
        tasks: Vec<Box<dyn BehaviorControl>>,
        memory_type: MemoryModuleType,
    ) {
        let mut reqs = HashSet::new();
        reqs.insert((memory_type.clone(), MemoryStatus::ValuePresent));
        let mut erase = HashSet::new();
        erase.insert(memory_type);
        let pairs = Self::create_priority_pairs(priority_start, tasks);
        self.add_activity_pairs(activity, pairs, reqs, erase);
    }

    /// `addActivity(activity, tasks)` — tasks уже с приоритетами.
    pub fn add_activity_pairs_list(
        &self,
        activity: Arc<Activity>,
        tasks: Vec<(i32, Box<dyn BehaviorControl>)>,
    ) {
        self.add_activity_pairs(activity, tasks, HashSet::new(), HashSet::new());
    }

    /// `addActivityWithConditions(activity, priorityStart,
    /// tasks, memoryStatuses)`.
    pub fn add_activity_with_conditions(
        &self,
        activity: Arc<Activity>,
        priority_start: i32,
        tasks: Vec<Box<dyn BehaviorControl>>,
        memory_statuses: HashSet<MemoryStatusPair>,
    ) {
        let pairs = Self::create_priority_pairs(priority_start, tasks);
        self.add_activity_with_conditions_pairs(activity, pairs, memory_statuses);
    }

    /// `addActivityWithConditions(activity, tasks, statuses)` —
    /// tasks уже с приоритетами.
    pub fn add_activity_with_conditions_pairs(
        &self,
        activity: Arc<Activity>,
        tasks: Vec<(i32, Box<dyn BehaviorControl>)>,
        memory_statuses: HashSet<MemoryStatusPair>,
    ) {
        self.add_activity_pairs(activity, tasks, memory_statuses, HashSet::new());
    }

    /// `addActivityAndRemoveMemoriesWhenStopped` — полная версия.
    pub fn add_activity_pairs(
        &self,
        activity: Arc<Activity>,
        tasks: Vec<(i32, Box<dyn BehaviorControl>)>,
        memory_statuses: HashSet<MemoryStatusPair>,
        memory_types: HashSet<MemoryModuleType>,
    ) {
        self.activity_requirements
            .borrow_mut()
            .insert(activity.clone(), memory_statuses);
        if !memory_types.is_empty() {
            self.activity_memories_to_erase
                .borrow_mut()
                .insert(activity.clone(), memory_types);
        }
        let mut avail = self.available_behaviors.borrow_mut();
        for (priority, behavior) in tasks {
            avail
                .entry(priority)
                .or_default()
                .entry(activity.clone())
                .or_default()
                .push(RefCell::new(behavior));
        }
    }

    /// `removeAllBehaviors()` — @VisibleForTesting.
    pub fn remove_all_behaviors(&self) {
        self.available_behaviors.borrow_mut().clear();
    }

    /// `isActive(activity)`.
    pub fn is_active(&self, activity: &Arc<Activity>) -> bool {
        self.active_activities.borrow().contains(activity)
    }

    /// `copyWithoutBehaviors()` — копия memories/sensors без behaviors.
    pub fn copy_without_behaviors(&self) -> Brain {
        let memories_snapshot: Vec<(MemoryModuleType, Option<ExpirableValue>)> = self
            .memories
            .borrow()
            .iter()
            .filter(|(_, v)| v.is_some())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let mut new_memories = HashMap::new();
        for k in self.memories.borrow().keys() {
            new_memories.insert(k.clone(), None);
        }
        for (k, v) in memories_snapshot {
            new_memories.insert(k, v);
        }
        Brain {
            memories: RefCell::new(new_memories),
            sensors: RefCell::new(Vec::new()),
            available_behaviors: RefCell::new(BTreeMap::new()),
            activity_requirements: RefCell::new(HashMap::new()),
            activity_memories_to_erase: RefCell::new(HashMap::new()),
            core_activities: RefCell::new(self.core_activities.borrow().clone()),
            active_activities: RefCell::new(HashSet::new()),
            default_activity: RefCell::new(self.default_activity.borrow().clone()),
            schedule: RefCell::new(self.schedule.borrow().clone()),
            last_schedule_update: Cell::new(-9999),
        }
    }

    // ---------- tick ----------

    /// `tick(level, entity)` — полный цикл Brain.
    pub fn tick(&self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        self.forget_outdated_memories();
        self.tick_sensors(level, mob);
        self.start_each_non_running_behavior(level, mob);
        self.tick_each_running_behavior(level, mob);
    }

    fn tick_sensors(&self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let sensors = self.sensors.borrow();
        for (_, sensor) in sensors.iter() {
            sensor.borrow_mut().tick(level, mob);
        }
    }

    fn forget_outdated_memories(&self) {
        let mut any_expired = false;
        {
            let mut memories = self.memories.borrow_mut();
            for opt in memories.values_mut() {
                if let Some(ev) = opt {
                    if ev.has_expired() {
                        any_expired = true;
                    }
                    ev.tick();
                }
            }
        }
        if any_expired {
            self.erase_outdated_memories();
        }
    }

    fn erase_outdated_memories(&self) {
        let expired: Vec<MemoryModuleType> = self
            .memories
            .borrow()
            .iter()
            .filter(|(_, o)| o.as_ref().map(|e| e.has_expired()).unwrap_or(false))
            .map(|(k, _)| k.clone())
            .collect();
        for ty in expired {
            self.erase_memory(&ty);
        }
    }

    /// `stopAll(level, owner)` — doStop всех RUNNING.
    pub fn stop_all(&self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let game_time = level.game_time();
        let avail = self.available_behaviors.borrow();
        for (_, by_activity) in avail.iter() {
            for (_, behaviors) in by_activity.iter() {
                for b in behaviors.iter() {
                    let mut b = b.borrow_mut();
                    if b.status() == BehaviorStatus::Running {
                        b.do_stop(level, mob, game_time);
                    }
                }
            }
        }
    }

    fn start_each_non_running_behavior(&self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let game_time = level.game_time();
        let active = self.active_activities.borrow().clone();
        let avail = self.available_behaviors.borrow();
        // Приоритеты по возрастанию (BTreeMap уже отсортирован).
        for (_, by_activity) in avail.iter() {
            for activity in &active {
                if let Some(behaviors) = by_activity.get(activity) {
                    for b in behaviors.iter() {
                        let mut b = b.borrow_mut();
                        if b.status() == BehaviorStatus::Stopped {
                            let _ = b.try_start(level, mob, game_time);
                        }
                    }
                }
            }
        }
    }

    fn tick_each_running_behavior(&self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let game_time = level.game_time();
        let avail = self.available_behaviors.borrow();
        for (_, by_activity) in avail.iter() {
            for (_, behaviors) in by_activity.iter() {
                for b in behaviors.iter() {
                    let mut b = b.borrow_mut();
                    if b.status() == BehaviorStatus::Running {
                        b.tick_or_stop(level, mob, game_time);
                    }
                }
            }
        }
    }

    // ---------- queries ----------

    fn activity_requirements_are_met(&self, activity: &Arc<Activity>) -> bool {
        let reqs = self.activity_requirements.borrow();
        let Some(set) = reqs.get(activity) else {
            return false;
        };
        set.iter().all(|(ty, status)| self.check_memory(ty, *status))
    }

    /// `isBrainDead()`.
    pub fn is_brain_dead(&self) -> bool {
        self.memories.borrow().is_empty()
            && self.sensors.borrow().is_empty()
            && self.available_behaviors.borrow().is_empty()
    }

    /// `getRunningBehaviors()` — @VisibleForDebug: список
    /// debug-строк RUNNING поведений.
    pub fn get_running_behaviors_debug(&self) -> Vec<String> {
        let avail = self.available_behaviors.borrow();
        let mut out = Vec::new();
        for (_, by_activity) in avail.iter() {
            for (_, behaviors) in by_activity.iter() {
                for b in behaviors.iter() {
                    let b = b.borrow();
                    if b.status() == BehaviorStatus::Running {
                        out.push(b.debug_string());
                    }
                }
            }
        }
        out
    }

    /// `getMemories()` — @VisibleForDebug: snapshot memories.
    pub fn get_memories_snapshot(&self) -> HashMap<MemoryModuleType, Option<ExpirableValue>> {
        self.memories.borrow().clone()
    }

    /// Доступ к sensors для отладки/регистрации.
    pub fn sensor_count(&self) -> usize {
        self.sensors.borrow().len()
    }

    /// `memories()` — итерация (MemoryModuleType, Option<value>)
    /// для сериализации; значения type-erased.
    pub fn memories_entries(&self) -> Vec<(MemoryModuleType, Option<ExpirableValue>)> {
        self.memories.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

/// `isEmptyCollection` — Collection/Set/List emptiness для
/// type-erased значений памяти.
fn is_empty_collection_value(v: &dyn std::any::Any) -> bool {
    use std::any::Any;
    if let Some(x) = v.downcast_ref::<Vec<crate::world::entity::living_entity::LivingEntity>>() {
        x.is_empty()
    } else if let Some(x) = v.downcast_ref::<Vec<crate::world::entity::entity::Entity>>() {
        x.is_empty()
    } else if let Some(x) = v.downcast_ref::<Vec<crate::core::global_pos::GlobalPos>>() {
        x.is_empty()
    } else if let Some(x) = v.downcast_ref::<Vec<crate::core::block_pos::BlockPos>>() {
        x.is_empty()
    } else if let Some(x) = v.downcast_ref::<Vec<crate::util::uuid::Uuid>>() {
        x.is_empty()
    } else if let Some(x) = v.downcast_ref::<std::collections::HashSet<crate::core::global_pos::GlobalPos>>() {
        x.is_empty()
    } else if let Some(x) = v.downcast_ref::<std::collections::HashSet<crate::util::uuid::Uuid>>() {
        x.is_empty()
    } else {
        // Vec<LivingEntity>-псевдонимы через Any не ловятся —
        // покрываем основные контейнеры.
        let _ = v as &dyn Any;
        false
    }
}

/// `Activity.IDLE` — helper для default-значений.
pub fn idle_activity() -> Arc<Activity> {
    activity_by_name("idle").expect("activity registry not bootstrapped")
}
