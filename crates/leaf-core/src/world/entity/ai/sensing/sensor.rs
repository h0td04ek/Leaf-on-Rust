//! Перенос `net.minecraft.world.entity.ai.sensing.Sensor` —
//! базовый класс сенсоров Brain (scanRate-тикование, статические
//! TargetingConditions для targetable/attackable проверок).

use std::cell::RefCell;
use std::sync::{Arc, Mutex, OnceLock};

use crate::entity::ai::attributes::attributes::follow_range;
use crate::random::RandomSource;
use crate::world::entity::ai::memory::memory::MemoryModuleType;
use crate::world::entity::ai::memory::keys;
use crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;
use crate::resources::identifier::Identifier;

thread_local! {
    /// Статические условия Java `Sensor` — мутируются
    /// `updateTargetingConditionRanges` каждый scanRate-тик.
    static TARGET_CONDITIONS: RefCell<TargetingConditions> =
        RefCell::new(TargetingConditions::for_non_combat());
    static TARGET_CONDITIONS_IGNORE_INVIS: RefCell<TargetingConditions> = RefCell::new({
        let mut c = TargetingConditions::for_non_combat();
        c.ignore_invisibility_testing();
        c
    });
    static ATTACK_TARGET_CONDITIONS: RefCell<TargetingConditions> =
        RefCell::new(TargetingConditions::for_combat());
    static ATTACK_TARGET_CONDITIONS_IGNORE_INVIS: RefCell<TargetingConditions> = RefCell::new({
        let mut c = TargetingConditions::for_combat();
        c.ignore_invisibility_testing();
        c
    });
    static ATTACK_TARGET_CONDITIONS_IGNORE_LOS: RefCell<TargetingConditions> = RefCell::new({
        let mut c = TargetingConditions::for_combat();
        c.ignore_line_of_sight();
        c
    });
    static ATTACK_TARGET_CONDITIONS_IGNORE_INVIS_AND_LOS: RefCell<TargetingConditions> =
        RefCell::new({
            let mut c = TargetingConditions::for_combat();
            c.ignore_line_of_sight();
            c.ignore_invisibility_testing();
            c
        });
}

/// `RandomSource.createThreadSafe()` — общий RNG сенсоров.
fn sensor_random() -> &'static Mutex<Box<dyn RandomSource>> {
    static R: OnceLock<Mutex<Box<dyn RandomSource>>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(crate::random::create(0)))
}

/// `rememberPositives(ticks, predicate)` — после первого true
/// ещё `ticks` вызовов возвращает true.
pub fn remember_positives<'a, T, U>(
    ticks: i32,
    predicate: impl Fn(&T, &U) -> bool + 'a,
) -> impl FnMut(&T, &U) -> bool + 'a {
    let counter = std::cell::Cell::new(0i32);
    move |t: &T, u: &U| {
        if predicate(t, u) {
            counter.set(ticks);
            true
        } else {
            let v = counter.get() - 1;
            counter.set(v);
            v >= 0
        }
    }
}

/// `Sensor<E extends LivingEntity>` — базовый сенсор.
/// В Rust — trait; `tick` финальный (scanRate), `do_tick` —
/// реализация.
pub trait Sensor {
    /// `scanRate` — интервал тиков между doTick.
    fn scan_rate(&self) -> i32;

    /// `doTick(level, entity)` — основная логика.
    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob);

    /// `requires()` — обязательные memory-типы.
    fn requires(&self) -> Vec<MemoryModuleType>;

    /// Текущее `timeToTick`.
    fn time_to_tick(&self) -> i64;
    /// Установить `timeToTick`.
    fn set_time_to_tick(&mut self, v: i64);

    /// Финальный `tick(level, entity)`: `--timeToTick <= 0` →
    /// `updateTargetingConditionRanges` + `doTick` + reset.
    fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let mut tt = self.time_to_tick() - 1;
        if tt <= 0 {
            tt = self.scan_rate() as i64;
            // Paper tickRates опущены — scanRate как дефолт.
            update_targeting_condition_ranges(mob);
            self.do_tick(level, mob);
        }
        self.set_time_to_tick(tt);
    }

}

/// `updateTargetingConditionRanges(entity)` — FOLLOW_RANGE
/// атрибут → range всех статических условий.
pub fn update_targeting_condition_ranges(mob: &Mob) {
    let follow = mob
        .living
        .get_attribute_value_by_id_ro(&follow_range());
    TARGET_CONDITIONS.with(|c| {
        c.borrow_mut().range(follow);
    });
    TARGET_CONDITIONS_IGNORE_INVIS.with(|c| {
        c.borrow_mut().range(follow);
    });
    ATTACK_TARGET_CONDITIONS.with(|c| {
        c.borrow_mut().range(follow);
    });
    ATTACK_TARGET_CONDITIONS_IGNORE_INVIS.with(|c| {
        c.borrow_mut().range(follow);
    });
    ATTACK_TARGET_CONDITIONS_IGNORE_LOS.with(|c| {
        c.borrow_mut().range(follow);
    });
    ATTACK_TARGET_CONDITIONS_IGNORE_INVIS_AND_LOS.with(|c| {
        c.borrow_mut().range(follow);
    });
}

/// `Sensor.isEntityTargetable(level, entity, target)`.
pub fn is_entity_targetable(
    level: &Arc<dyn LevelReader>,
    entity: &LivingEntity,
    target: &LivingEntity,
) -> bool {
    let is_attack_target = entity
        .brain_is_memory_value(&keys::attack_target(), target)
        .unwrap_or(false);
    if is_attack_target {
        TARGET_CONDITIONS_IGNORE_INVIS.with(|c| c.borrow().test(level, None, target))
    } else {
        TARGET_CONDITIONS.with(|c| c.borrow().test(level, None, target))
    }
}

/// `Sensor.isEntityAttackable(level, entity, target)`.
pub fn is_entity_attackable(
    level: &Arc<dyn LevelReader>,
    entity: &LivingEntity,
    target: &LivingEntity,
) -> bool {
    let is_attack_target = entity
        .brain_is_memory_value(&keys::attack_target(), target)
        .unwrap_or(false);
    if is_attack_target {
        ATTACK_TARGET_CONDITIONS_IGNORE_INVIS.with(|c| c.borrow().test(level, None, target))
    } else {
        ATTACK_TARGET_CONDITIONS.with(|c| c.borrow().test(level, None, target))
    }
}

/// `Sensor.isEntityAttackableIgnoringLineOfSight`.
pub fn is_entity_attackable_ignoring_los(
    level: &Arc<dyn LevelReader>,
    entity: &LivingEntity,
    target: &LivingEntity,
) -> bool {
    let is_attack_target = entity
        .brain_is_memory_value(&keys::attack_target(), target)
        .unwrap_or(false);
    if is_attack_target {
        ATTACK_TARGET_CONDITIONS_IGNORE_INVIS_AND_LOS
            .with(|c| c.borrow().test(level, None, target))
    } else {
        ATTACK_TARGET_CONDITIONS_IGNORE_LOS.with(|c| c.borrow().test(level, None, target))
    }
}

/// `Sensor.wasEntityAttackableLastNTicks(entity, ticks)` —
/// stateful предикат (BiPredicate<level, target>).
pub fn was_entity_attackable_last_n_ticks(
    entity: LivingEntity,
    ticks: i32,
) -> impl FnMut(&Arc<dyn LevelReader>, &LivingEntity) -> bool {
    remember_positives(ticks, move |level: &Arc<dyn LevelReader>, target: &LivingEntity| {
        is_entity_attackable(level, &entity, target)
    })
}

/// Хелпер для наследников: общий scanRate-тикер.
pub struct SensorState {
    /// `timeToTick`.
    pub time_to_tick: i64,
    /// `scanRate`.
    pub scan_rate: i32,
}

impl SensorState {
    /// `new(scanRate)` — `timeToTick = random.nextInt(scanRate)`.
    pub fn new(scan_rate: i32) -> Self {
        let initial = {
            let mut r = sensor_random().lock().unwrap();
            r.next_int_bound(scan_rate.max(1)) as i64
        };
        Self {
            time_to_tick: initial,
            scan_rate,
        }
    }

    /// `new()` — scanRate=20.
    pub fn default_rate() -> Self {
        Self::new(20)
    }

    /// Реализация `Sensor::tick` для сенсоров на SensorState.
    /// Возвращает true, если в этом тике нужен `do_tick`.
    pub fn tick_due(&mut self, mob: &Mob) -> bool {
        self.time_to_tick -= 1;
        if self.time_to_tick <= 0 {
            self.time_to_tick = self.scan_rate as i64;
            update_targeting_condition_ranges(mob);
            true
        } else {
            false
        }
    }
}

/// `entity_type_is(entity, id)` — проверка типа по registry-id
/// (`minecraft:villager` и т.д.) для фильтров сенсоров.
pub fn entity_type_is(entity: &LivingEntity, id: &str) -> bool {
    let want = Identifier::minecraft(id);
    crate::world::entity::entity_type::holder_type_id(&entity.entity.entity_type)
        .map(|t| t == want)
        .unwrap_or(false)
}
