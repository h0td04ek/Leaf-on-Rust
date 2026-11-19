//! Перенос `net.minecraft.world.entity.ai.sensing.*Sensor` —
//! все 25 vanilla-сенсоров Brain.
//!
//! Сенсоры работают с `&mut Mob` и пишут в `mob.living.brain`
//! через type-erased memory API. Entity-специфичные проверки
//! (Hoglin/Piglin/Villager/Frog/Armadillo) — через хуки
//! `LivingEntity`, добавленные для переопределения в
//! конкретных классах мобов.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::entity::ai::attributes::attributes::{follow_range, tempt_range};
use crate::world::entity::ai::memory::keys;
use crate::world::entity::ai::memory::memory::{
    MemoryModuleType, NearestVisibleLivingEntities,
};
use crate::world::entity::ai::memory::memory::MemoryModuleType as MMT;
use crate::world::entity::ai::sensing::sensor::{
    entity_type_is, is_entity_attackable, is_entity_targetable, Sensor, SensorState,
};
use crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions;
use crate::world::entity::entity::Entity;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::item::item_stack::ItemStack;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::aabb::Aabb;

// ===================== DummySensor =====================

/// `DummySensor` — пустой сенсор.
pub struct DummySensor {
    state: SensorState,
}

impl DummySensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::default_rate() }
    }
}

impl Sensor for DummySensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }
    fn do_tick(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob) {}
    fn requires(&self) -> Vec<MMT> { Vec::new() }
}

// ===================== NearestLivingEntitySensor =====================

/// `NearestLivingEntitySensor` — скан LivingEntity в FOLLOW_RANGE
/// → NEAREST_LIVING_ENTITIES + NEAREST_VISIBLE_LIVING_ENTITIES.
pub struct NearestLivingEntitySensor {
    state: SensorState,
}

impl NearestLivingEntitySensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::default_rate() }
    }
}

impl Sensor for NearestLivingEntitySensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let range = mob.living.get_attribute_value_by_id_ro(&follow_range());
        let bb = mob.living.entity.get_bounding_box().inflate(range, range, range);
        let pos = mob.living.entity.position();
        let mut entities: Vec<LivingEntity> = level
            .entities_in_box(&bb)
            .into_iter()
            .filter(|e| e.is_living_entity())
            .filter(|e| {
                let le = LivingEntity::from_entity(e.clone());
                le.is_alive()
                    && e.distance_to_sqr_pos(pos.x, pos.y, pos.z) <= range * range
            })
            .map(LivingEntity::from_entity)
            .collect();
        // сортировка по расстоянию до моба (fastBitRadixSort-экв.)
        entities.sort_by(|a, b| {
            a.entity
                .distance_to_sqr_pos(pos.x, pos.y, pos.z)
                .partial_cmp(&b.entity.distance_to_sqr_pos(pos.x, pos.y, pos.z))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let nvle = NearestVisibleLivingEntities::new(level.clone(), &mob.living, entities.clone());
        mob.living.brain_set_value(&keys::nearest_living_entities(), entities);
        mob.living
            .brain_set_value(&keys::nearest_visible_living_entities(), nvle);
    }

    fn requires(&self) -> Vec<MMT> {
        vec![
            keys::nearest_living_entities(),
            keys::nearest_visible_living_entities(),
        ]
    }
}

// ===================== NearestVisibleLivingEntitySensor =====================

/// `NearestVisibleLivingEntitySensor` — базовый класс:
/// ближайшая видимая сущность по `isMatchingEntity`.
pub struct NearestVisibleLivingEntitySensor {
    state: SensorState,
    memory: MMT,
    /// Предикат совпадения (level, owner, target).
    matcher: Rc<dyn Fn(&Arc<dyn LevelReader>, &LivingEntity, &LivingEntity) -> bool>,
}

impl NearestVisibleLivingEntitySensor {
    fn new(
        memory: MMT,
        matcher: impl Fn(&Arc<dyn LevelReader>, &LivingEntity, &LivingEntity) -> bool + 'static,
    ) -> Self {
        Self {
            state: SensorState::default_rate(),
            memory,
            matcher: Rc::new(matcher),
        }
    }
}

impl Sensor for NearestVisibleLivingEntitySensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let nvle: Option<NearestVisibleLivingEntities> =
            mob.living.brain_get(&keys::nearest_visible_living_entities());
        let found = nvle.and_then(|nv| {
            let matcher = self.matcher.clone();
            let lvl = level.clone();
            let owner = mob.living.clone();
            nv.find_closest(&move |t| matcher(&lvl, &owner, t))
        });
        mob.living.brain_set(&self.memory, found);
    }

    fn requires(&self) -> Vec<MMT> {
        vec![self.memory.clone()]
    }
}

// ===================== AdultSensor =====================

/// `AdultSensor` — ближайший видимый взрослый того же типа.
pub struct AdultSensor {
    inner: NearestVisibleLivingEntitySensor,
}

impl AdultSensor {
    /// `new()` — same-type adults.
    pub fn new() -> Self {
        Self {
            inner: NearestVisibleLivingEntitySensor::new(
                keys::nearest_visible_adult(),
                |_level, owner, target| {
                    crate::world::entity::entity_type::holder_type_id(&owner.entity.entity_type)
                        == crate::world::entity::entity_type::holder_type_id(&target.entity.entity_type)
                        && !target.entity.is_baby()
                },
            ),
        }
    }
}

impl Sensor for AdultSensor {
    fn scan_rate(&self) -> i32 { self.inner.scan_rate() }
    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) { self.inner.do_tick(level, mob) }
    fn requires(&self) -> Vec<MMT> {
        let mut r = self.inner.requires();
        r.push(keys::nearest_visible_living_entities());
        r
    }
    fn time_to_tick(&self) -> i64 { self.inner.time_to_tick() }
    fn set_time_to_tick(&mut self, v: i64) { self.inner.set_time_to_tick(v) }
}

/// `AdultSensorAnyType` — ближайший взрослый из
/// FOLLOWABLE_FRIENDLY_MOBS.
pub struct AdultSensorAnyType {
    inner: NearestVisibleLivingEntitySensor,
}

impl AdultSensorAnyType {
    /// `new()`.
    pub fn new() -> Self {
        Self {
            inner: NearestVisibleLivingEntitySensor::new(
                keys::nearest_visible_adult(),
                |_level, _owner, target| {
                    target
                        .entity_type_value()
                        .map(|t| {
                            crate::tags::entity_type_tags::entity_type_is(
                                t,
                                crate::tags::entity_type_tags::keys::followable_friendly_mobs(),
                            )
                        })
                        .unwrap_or(false)
                        && !target.entity.is_baby()
                },
            ),
        }
    }
}

impl Sensor for AdultSensorAnyType {
    fn scan_rate(&self) -> i32 { self.inner.scan_rate() }
    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) { self.inner.do_tick(level, mob) }
    fn requires(&self) -> Vec<MMT> {
        let mut r = self.inner.requires();
        r.push(keys::nearest_visible_living_entities());
        r
    }
    fn time_to_tick(&self) -> i64 { self.inner.time_to_tick() }
    fn set_time_to_tick(&mut self, v: i64) { self.inner.set_time_to_tick(v) }
}

// ===================== AxolotlAttackablesSensor =====================

/// `AxolotlAttackablesSensor` — attackables в воде (tag-based).
pub struct AxolotlAttackablesSensor {
    inner: NearestVisibleLivingEntitySensor,
}

impl AxolotlAttackablesSensor {
    /// `TARGET_DETECTION_DISTANCE` = 8 → dist² 64.
    pub const TARGET_DETECTION_DISTANCE_SQ: f64 = 64.0;

    /// `new()`.
    pub fn new() -> Self {
        Self {
            inner: NearestVisibleLivingEntitySensor::new(
                keys::nearest_attackable(),
                |level, entity, target| {
                    let close = target.distance_to_sqr(&entity.entity) <= Self::TARGET_DETECTION_DISTANCE_SQ;
                    close
                        && target.entity.is_in_water()
                        && (entity_type_tag_check(target, "axolotl_always_hostiles")
                            || (!entity.brain_has(&keys::has_hunting_cooldown())
                                && entity_type_tag_check(target, "axolotl_hunt_targets")))
                        && is_entity_attackable(level, entity, target)
                },
            ),
        }
    }
}

/// `entity.getType().is(EntityTypeTags.X)` — тег-проверка по id.
fn entity_type_tag_check(entity: &LivingEntity, tag: &str) -> bool {
    let Some(t) = entity.entity_type_value() else { return false };
    let tag_key = match tag {
        "axolotl_hunt_targets" => crate::tags::entity_type_tags::keys::axolotl_hunt_targets(),
        "axolotl_always_hostiles" => crate::tags::entity_type_tags::keys::axolotl_always_hostiles(),
        _ => return false,
    };
    crate::tags::entity_type_tags::entity_type_is(t, tag_key)
}

impl Sensor for AxolotlAttackablesSensor {
    fn scan_rate(&self) -> i32 { self.inner.scan_rate() }
    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        self.inner.do_tick(level, mob)
    }
    fn requires(&self) -> Vec<MMT> {
        let mut r = self.inner.requires();
        r.push(keys::nearest_visible_living_entities());
        r
    }
    fn time_to_tick(&self) -> i64 { self.inner.time_to_tick() }
    fn set_time_to_tick(&mut self, v: i64) { self.inner.set_time_to_tick(v) }
}

// ===================== FrogAttackablesSensor =====================

/// `FrogAttackablesSensor` — съедобные цели лягушки.
pub struct FrogAttackablesSensor {
    inner: NearestVisibleLivingEntitySensor,
}

impl FrogAttackablesSensor {
    /// `TARGET_DETECTION_DISTANCE` = 10.
    pub const TARGET_DETECTION_DISTANCE: f64 = 10.0;

    /// `new()`.
    pub fn new() -> Self {
        Self {
            inner: NearestVisibleLivingEntitySensor::new(
                keys::nearest_attackable(),
                |level, entity, target| {
                    let mut unreachable: Vec<crate::util::uuid::Uuid> = entity
                        .brain_get(&keys::unreachable_tongue_targets())
                        .unwrap_or_default();
                    unreachable.retain(|_| false);
                    entity.frog_can_eat(target)
                        && !entity.brain_has(&keys::has_hunting_cooldown())
                        && target.entity.distance_to_sqr(&entity.entity) <= Self::TARGET_DETECTION_DISTANCE * Self::TARGET_DETECTION_DISTANCE
                        && is_entity_attackable(level, entity, target)
                        && {
                            let list: Vec<crate::util::uuid::Uuid> = entity
                                .brain_get::<Vec<crate::util::uuid::Uuid>>(&keys::unreachable_tongue_targets())
                                .unwrap_or_default();
                            !list.contains(&target.get_uuid())
                        }
                },
            ),
        }
    }
}

impl Sensor for FrogAttackablesSensor {
    fn scan_rate(&self) -> i32 { self.inner.scan_rate() }
    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) { self.inner.do_tick(level, mob) }
    fn requires(&self) -> Vec<MMT> {
        let mut r = self.inner.requires();
        r.push(keys::nearest_visible_living_entities());
        r.push(keys::unreachable_tongue_targets());
        r.push(keys::has_hunting_cooldown());
        r
    }
    fn time_to_tick(&self) -> i64 { self.inner.time_to_tick() }
    fn set_time_to_tick(&mut self, v: i64) { self.inner.set_time_to_tick(v) }
}

// ===================== IsInWaterSensor =====================

/// `IsInWaterSensor` — IS_IN_WATER unit-память.
pub struct IsInWaterSensor {
    state: SensorState,
}

impl IsInWaterSensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::default_rate() }
    }
}

impl Sensor for IsInWaterSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        if mob.living.entity.is_in_water() {
            mob.living.brain_set_value(&keys::is_in_water(), ());
        } else {
            mob.living.brain_erase(&keys::is_in_water());
        }
    }

    fn requires(&self) -> Vec<MMT> {
        vec![keys::is_in_water()]
    }
}

// ===================== HurtBySensor =====================

/// `HurtBySensor` — HURT_BY/HURT_BY_ENTITY из lastDamageSource.
pub struct HurtBySensor {
    state: SensorState,
}

impl HurtBySensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::default_rate() }
    }
}

impl Sensor for HurtBySensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        if let Some(source) = mob.living.get_last_damage_source() {
            mob.living.brain_set_value(&keys::hurt_by(), source.clone());
            if let Some(causing) = source.get_causing_entity() {
                if causing.is_living_entity() {
                    let le = LivingEntity::from_entity(causing.clone());
                    mob.living.brain_set_value(&keys::hurt_by_entity(), le);
                }
            }
        } else {
            mob.living.brain_erase(&keys::hurt_by());
        }
        // HURT_BY_ENTITY валидация: жив && в том же уровне.
        let stale = mob
            .living
            .brain_get::<LivingEntity>(&keys::hurt_by_entity())
            .map(|attacker| {
                !attacker.is_alive()
                    || attacker
                        .entity
                        .level
                        .as_ref()
                        .map(|l| !Arc::ptr_eq(l, level))
                        .unwrap_or(true)
            })
            .unwrap_or(false);
        if stale {
            mob.living.brain_erase(&keys::hurt_by_entity());
        }
    }

    fn requires(&self) -> Vec<MMT> {
        vec![keys::hurt_by(), keys::hurt_by_entity()]
    }
}

// ===================== MobSensor =====================

/// `MobSensor<T>` — generic-детектор: если любая сущность из
/// NEAREST_LIVING_ENTITIES проходит `mob_test` → `toSet`=true
/// с TTL. Для Armadillo-испуга.
pub struct MobSensor {
    state: SensorState,
    /// `mobTest(sensingEntity, candidate)`.
    mob_test: Rc<dyn Fn(&LivingEntity, &LivingEntity) -> bool>,
    /// `readyTest(entity)`.
    ready_test: Rc<dyn Fn(&LivingEntity) -> bool>,
    /// `toSet` memory-тип.
    to_set: MMT,
    /// `memoryTimeToLive`.
    memory_time_to_live: i64,
}

impl MobSensor {
    /// `new(scanRate, mobTest, readyTest, toSet, ttl)`.
    pub fn new(
        scan_rate: i32,
        mob_test: impl Fn(&LivingEntity, &LivingEntity) -> bool + 'static,
        ready_test: impl Fn(&LivingEntity) -> bool + 'static,
        to_set: MMT,
        memory_time_to_live: i64,
    ) -> Self {
        Self {
            state: SensorState::new(scan_rate),
            mob_test: Rc::new(mob_test),
            ready_test: Rc::new(ready_test),
            to_set,
            memory_time_to_live,
        }
    }

    /// `ARMADILLO_SCARE_DETECTED` — scanRate=5,
    /// `Armadillo::isScaredBy`, `canStayRolledUp`,
    /// DANGER_DETECTED_RECENTLY, ttl=80.
    pub fn armadillo_scare() -> Self {
        Self::new(
            5,
            |sensing, candidate| sensing.armadillo_is_scared_by(candidate),
            |sensing| sensing.armadillo_can_stay_rolled_up(),
            keys::danger_detected_recently(),
            80,
        )
    }

    /// `checkForMobsNearby(entity)`.
    pub fn check_for_mobs_nearby(&self, mob: &mut Mob) {
        let entities: Vec<LivingEntity> = mob
            .living
            .brain_get(&keys::nearest_living_entities())
            .unwrap_or_default();
        let test = self.mob_test.clone();
        let owner = mob.living.clone();
        if entities.iter().any(|e| test(&owner, e)) {
            self.mob_detected(mob);
        }
    }

    /// `mobDetected(entity)`.
    pub fn mob_detected(&self, mob: &mut Mob) {
        mob.living
            .brain_set_with_expiry(&self.to_set, true, self.memory_time_to_live);
    }

    /// `clearMemory(entity)`.
    pub fn clear_memory(&self, mob: &mut Mob) {
        mob.living.brain_erase(&self.to_set);
    }
}

impl Sensor for MobSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        if !(self.ready_test)(&mob.living) {
            self.clear_memory(mob);
        } else {
            self.check_for_mobs_nearby(mob);
        }
    }

    fn requires(&self) -> Vec<MMT> {
        vec![keys::nearest_living_entities()]
    }
}

// ===================== GolemSensor =====================

/// `GolemSensor` — IRON_GOLEM рядом → GOLEM_DETECTED_RECENTLY.
pub struct GolemSensor {
    state: SensorState,
}

impl GolemSensor {
    /// `GOLEM_SCAN_RATE`/`MEMORY_TIME_TO_LIVE`.
    pub const GOLEM_SCAN_RATE: i32 = 200;
    pub const MEMORY_TIME_TO_LIVE: i64 = 599;

    /// `new()` — scanRate=200.
    pub fn new() -> Self {
        Self { state: SensorState::new(Self::GOLEM_SCAN_RATE) }
    }

    /// `checkForNearbyGolem(livingEntity)` — static.
    pub fn check_for_nearby_golem(mob: &mut Mob) {
        let entities: Vec<LivingEntity> = mob
            .living
            .brain_get(&keys::nearest_living_entities())
            .unwrap_or_default();
        if entities.iter().any(|e| entity_type_is(e, "iron_golem")) {
            Self::golem_detected(mob);
        }
    }

    /// `golemDetected(livingEntity)` — static.
    pub fn golem_detected(mob: &mut Mob) {
        mob.living.brain_set_with_expiry(
            &keys::golem_detected_recently(),
            true,
            Self::MEMORY_TIME_TO_LIVE,
        );
    }
}

impl Sensor for GolemSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        Self::check_for_nearby_golem(mob);
    }

    fn requires(&self) -> Vec<MMT> {
        vec![keys::nearest_living_entities()]
    }
}

// ===================== VillagerHostilesSensor =====================

/// `VillagerHostilesSensor` — враждебные мобы в пределах
/// per-type дистанций → NEAREST_HOSTILE.
pub struct VillagerHostilesSensor {
    inner: NearestVisibleLivingEntitySensor,
}

impl VillagerHostilesSensor {
    /// `ACCEPTABLE_DISTANCE_FROM_HOSTILES` — (type, dist).
    const HOSTILES: &[(&str, f64)] = &[
        ("drowned", 8.0), ("evoker", 12.0), ("husk", 8.0),
        ("illusioner", 12.0), ("pillager", 15.0), ("ravager", 12.0),
        ("vex", 8.0), ("vindicator", 10.0), ("zoglin", 10.0),
        ("zombie", 8.0), ("zombie_villager", 8.0),
    ];

    /// `new()`.
    pub fn new() -> Self {
        Self {
            inner: NearestVisibleLivingEntitySensor::new(
                keys::nearest_hostile(),
                |_level, entity, target| {
                    let dist = Self::HOSTILES
                        .iter()
                        .find(|(t, _)| entity_type_is(target, t))
                        .map(|(_, d)| *d);
                    match dist {
                        Some(d) => target.distance_to_sqr(&entity.entity) <= d * d,
                        None => false,
                    }
                },
            ),
        }
    }
}

impl Sensor for VillagerHostilesSensor {
    fn scan_rate(&self) -> i32 { self.inner.scan_rate() }
    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) { self.inner.do_tick(level, mob) }
    fn requires(&self) -> Vec<MMT> {
        let mut r = self.inner.requires();
        r.push(keys::nearest_visible_living_entities());
        r
    }
    fn time_to_tick(&self) -> i64 { self.inner.time_to_tick() }
    fn set_time_to_tick(&mut self, v: i64) { self.inner.set_time_to_tick(v) }
}

// ===================== VillagerBabiesSensor =====================

/// `VillagerBabiesSensor` — видимые villager-детёныши.
pub struct VillagerBabiesSensor {
    state: SensorState,
}

impl VillagerBabiesSensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::default_rate() }
    }
}

impl Sensor for VillagerBabiesSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let nvle: Option<NearestVisibleLivingEntities> =
            mob.living.brain_get(&keys::nearest_visible_living_entities());
        let babies: Vec<LivingEntity> = nvle
            .map(|nv| {
                nv.nearby_entities
                    .iter()
                    .filter(|t| {
                        entity_type_is(t, "villager")
                            && t.entity.is_baby()
                            && nv.line_of_sight_test(t)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        mob.living.brain_set_value(&keys::visible_villager_babies(), babies);
    }

    fn requires(&self) -> Vec<MMT> {
        vec![keys::visible_villager_babies()]
    }
}

// ===================== PlayerSensor =====================

/// `PlayerSensor` — игроки рядом → NEAREST_PLAYERS и
/// visible/attackable варианты.
pub struct PlayerSensor {
    state: SensorState,
}

impl PlayerSensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::default_rate() }
    }

    /// `getFollowDistance(entity)` — FOLLOW_RANGE.
    fn get_follow_distance(mob: &Mob) -> f64 {
        mob.living.get_attribute_value_by_id_ro(&follow_range())
    }
}

impl Sensor for PlayerSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let distance = Self::get_follow_distance(mob);
        let dist_sq = distance * distance;
        let pos = mob.living.entity.position();
        // players() → entities_in_box большого радиуса, фильтр
        // по типу player + !spectator + distSq.
        let reach = distance.max(1.0);
        let bb = mob.living.entity.get_bounding_box().inflate(reach, reach, reach);
        let mut players: Vec<LivingEntity> = level
            .entities_in_box(&bb)
            .into_iter()
            .filter(|e| e.is_living_entity())
            .filter(|e| {
                !e.is_spectator()
                    && e.distance_to_sqr_pos(pos.x, pos.y, pos.z) < dist_sq
            })
            .map(|e| {
                let mut le = LivingEntity::from_entity(e);
                // только игроки: entity_type == player
                if entity_type_is(&le, "player") {
                    Some(le)
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        players.sort_by(|a, b| {
            a.entity
                .distance_to_sqr_pos(pos.x, pos.y, pos.z)
                .partial_cmp(&b.entity.distance_to_sqr_pos(pos.x, pos.y, pos.z))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let targetable: Vec<LivingEntity> = players
            .iter()
            .filter(|p| is_entity_targetable(level, &mob.living, p))
            .cloned()
            .collect();
        let attackable: Vec<LivingEntity> = targetable
            .iter()
            .filter(|p| is_entity_attackable(level, &mob.living, p))
            .cloned()
            .collect();
        mob.living.brain_set_value(&keys::nearest_players(), players);
        mob.living.brain_set(
            &keys::nearest_visible_player(),
            targetable.first().cloned(),
        );
        mob.living
            .brain_set_value(&keys::nearest_visible_attackable_players(), attackable.clone());
        mob.living.brain_set(
            &keys::nearest_visible_attackable_player(),
            attackable.first().cloned(),
        );
    }

    fn requires(&self) -> Vec<MMT> {
        vec![
            keys::nearest_players(),
            keys::nearest_visible_player(),
            keys::nearest_visible_attackable_player(),
            keys::nearest_visible_attackable_players(),
        ]
    }
}

// ===================== NearestBedSensor =====================

/// `NearestBedSensor` — ближайший HOME-POI → NEAREST_BED
/// (только для baby-мобов; AcquirePoi-путь — через
/// `LevelReader::home_poi_near` до порта PoiManager).
pub struct NearestBedSensor {
    state: SensorState,
    /// `triedCount`/`lastUpdate`/`batchCache` — кэш POI-поиска.
    tried_count: i32,
    last_update: i64,
    batch_cache: std::collections::HashMap<i64, i64>,
}

impl NearestBedSensor {
    /// `CACHE_TIMEOUT`/`BATCH_SIZE`/`RATE`.
    pub const CACHE_TIMEOUT: i64 = 40;
    pub const BATCH_SIZE: i32 = 5;
    pub const RATE: i32 = 20;

    /// `new()` — scanRate=20.
    pub fn new() -> Self {
        Self {
            state: SensorState::new(Self::RATE),
            tried_count: 0,
            last_update: 0,
            batch_cache: std::collections::HashMap::new(),
        }
    }
}

impl Sensor for NearestBedSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        if mob.living.entity.is_baby() {
            self.tried_count = 0;
            self.last_update = level.game_time()
                + {
                    let mut r = mob.random.borrow_mut();
                    r.next_int_bound(20) as i64
                };
            let center = mob.living.entity.block_position();
            // predicate: кэшированное/превышение batch → false;
            // иначе triedCount++ >= 5 → false; иначе кэш + true.
            let mut tried = self.tried_count;
            let cache_timeout = self.last_update + Self::CACHE_TIMEOUT;
            let candidate = (0..Self::BATCH_SIZE * 8)
                .filter_map(|i| {
                    let dx = (i % 9) - 4;
                    let dy = ((i / 9) % 5) - 2;
                    let dz = ((i / 45) % 9) - 4;
                    let pos = BlockPos::new(center.x + dx, center.y + dy, center.z + dz);
                    let packed = pos.as_long();
                    if self.batch_cache.contains_key(&packed) {
                        return None;
                    }
                    tried += 1;
                    if tried >= Self::BATCH_SIZE {
                        return None;
                    }
                    self.batch_cache.insert(packed, cache_timeout);
                    Some(pos)
                })
                .find_map(|pos| level.home_poi_near(pos, 1));
            self.tried_count = tried;
            // processPath: path.canReach → target POI → NEAREST_BED.
            match candidate {
                Some(poi) => {
                    // Java хранит GlobalPos(dimension, pos).
                    let gpos = level.dimension_id().map(|dim| {
                        crate::core::global_pos::GlobalPos::of(
                            crate::resources::ResourceKey::create_with_registry_name(
                                crate::core::registry::registries::dimension(),
                                dim.clone(),
                            ),
                            poi,
                        )
                    });
                    mob.living.brain_set(&keys::nearest_bed(), gpos);
                }
                None => {
                    if self.tried_count < Self::BATCH_SIZE {
                        self.batch_cache
                            .retain(|_, &mut ttl| ttl >= self.last_update);
                    }
                }
            }
        }
    }

    fn requires(&self) -> Vec<MMT> {
        vec![keys::nearest_bed()]
    }
}

// ===================== SecondaryPoiSensor =====================

/// `SecondaryPoiSensor` — SECONDARY_JOB_SITE для villager-профессий
/// с secondaryPoi (cleric warps и т.п.). До порта VillagerData —
/// через `villager_secondary_poi_blocks` hook.
pub struct SecondaryPoiSensor {
    state: SensorState,
}

impl SecondaryPoiSensor {
    /// `SCAN_RATE` = 40.
    pub const SCAN_RATE: i32 = 40;

    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::new(Self::SCAN_RATE) }
    }
}

impl Sensor for SecondaryPoiSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let Some(poi_blocks) = mob.living.villager_secondary_poi_blocks() else {
            mob.living.brain_erase(&keys::secondary_job_site());
            return;
        };
        let dim_id = level.dimension_id().cloned();
        let Some(dim_id) = dim_id else {
            mob.living.brain_erase(&keys::secondary_job_site());
            return;
        };
        let dim = crate::resources::ResourceKey::<crate::world::level::Level>::create_with_registry_name(
            crate::core::registry::registries::dimension(),
            dim_id,
        );
        let center = mob.living.entity.block_position();
        let mut list: Vec<crate::core::global_pos::GlobalPos> = Vec::new();
        'outer: for dx in -4..=4 {
            for dy in -2..=2 {
                for dz in -4..=4 {
                    let pos = BlockPos::new(center.x + dx, center.y + dy, center.z + dz);
                    if let Some(state) = level.get_block_state(pos) {
                        if poi_blocks.contains(&state.block.id) {
                            list.push(crate::core::global_pos::GlobalPos::of(
                                dim.clone(),
                                pos,
                            ));
                            if list.len() > 4096 {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        if !list.is_empty() {
            mob.living.brain_set_value(&keys::secondary_job_site(), list);
        } else {
            mob.living.brain_erase(&keys::secondary_job_site());
        }
    }

    fn requires(&self) -> Vec<MMT> {
        vec![keys::secondary_job_site()]
    }
}

// ===================== PiglinSpecificSensor =====================

/// `PiglinSpecificSensor` — все piglin/hoglin/player memory.
pub struct PiglinSpecificSensor {
    state: SensorState,
}

impl PiglinSpecificSensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::default_rate() }
    }
}

impl Sensor for PiglinSpecificSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        // NEAREST_REPELLENT — repellent в 8x4.
        let center = mob.living.entity.block_position();
        let repellent = BlockPos::find_closest_match(&center, 8, 4, &mut |pos| {
            level
                .get_block_state(*pos)
                .map(|s| {
                    crate::tags::is_block_in_named_tag(
                        &s.block.id,
                        &crate::tags::block_tags::BlockTags::piglin_repellents(),
                    ) && {
                        if s.block.id.path == "soul_campfire" {
                            level.is_lit_campfire(*pos)
                        } else {
                            true
                        }
                    }
                })
                .unwrap_or(false)
        });
        mob.living.brain_set(&keys::nearest_repellent(), repellent);

        let nvle: Option<NearestVisibleLivingEntities> =
            mob.living.brain_get(&keys::nearest_visible_living_entities());
        let empty = NearestVisibleLivingEntities::empty();
        let nvle = nvle.as_ref().unwrap_or(&empty);
        let visible: Vec<LivingEntity> = nvle.find_all(&|_| true);

        let mut nemesis: Option<LivingEntity> = None;
        let mut huntable_hoglin: Option<LivingEntity> = None;
        let mut baby_hoglin: Option<LivingEntity> = None;
        let mut zombified: Option<LivingEntity> = None;
        let mut player_no_gold: Option<LivingEntity> = None;
        let mut player_loved: Option<LivingEntity> = None;
        let mut adult_piglins: Vec<LivingEntity> = Vec::new();
        let mut adult_hoglin_count = 0i32;
        let owner = mob.living.clone();

        for target in &visible {
            let tid = crate::world::entity::entity_type::holder_type_id(&target.entity.entity_type)
                .map(|i| i.path.clone())
                .unwrap_or_default();
            if tid == "hoglin" {
                if target.entity.is_baby() {
                    if baby_hoglin.is_none() {
                        baby_hoglin = Some(target.clone());
                    }
                } else {
                    adult_hoglin_count += 1;
                    if huntable_hoglin.is_none() && target.can_be_hunted() {
                        huntable_hoglin = Some(target.clone());
                    }
                }
            } else if tid == "piglin_brute" {
                adult_piglins.push(target.clone());
            } else if tid == "piglin" {
                if target.entity.is_baby() {
                    // baby piglin — optional3 в Java не сохраняется
                } else {
                    adult_piglins.push(target.clone());
                }
            } else if tid == "player" {
                if player_no_gold.is_none()
                    && !target.piglin_ai_is_wearing_safe_armor()
                    && owner.can_attack(target)
                {
                    player_no_gold = Some(target.clone());
                }
                if player_loved.is_none()
                    && !target.is_spectator()
                    && target.piglin_ai_is_player_holding_loved_item()
                {
                    player_loved = Some(target.clone());
                }
            } else if nemesis.is_none()
                && (tid == "wither_skeleton" || tid == "wither")
            {
                nemesis = Some(target.clone());
            } else if zombified.is_none()
                && target
                    .entity
                    .entity_type
                    .value()
                    .map(|t| LivingEntity::piglin_ai_is_zombified_type(&t.id))
                    .unwrap_or(false)
            {
                zombified = Some(target.clone());
            }
        }

        // NEARBY_ADULT_PIGLINS — все взрослые AbstractPiglin
        // из NEAREST_LIVING_ENTITIES (видимые + невидимые).
        let nearby: Vec<LivingEntity> = mob
            .living
            .brain_get(&keys::nearest_living_entities())
            .unwrap_or_default();
        let nearby_adults: Vec<LivingEntity> = nearby
            .into_iter()
            .filter(|e| {
                let tid = crate::world::entity::entity_type::holder_type_id(&e.entity.entity_type)
                    .map(|i| i.path)
                    .unwrap_or_default();
                (tid == "piglin" || tid == "piglin_brute") && e.is_adult()
            })
            .collect();

        mob.living.brain_set(&keys::nearest_visible_nemesis(), nemesis);
        mob.living.brain_set(&keys::nearest_visible_huntable_hoglin(), huntable_hoglin);
        mob.living.brain_set(&keys::nearest_visible_baby_hoglin(), baby_hoglin);
        mob.living.brain_set(&keys::nearest_visible_zombified(), zombified);
        mob.living.brain_set(&keys::nearest_targetable_player_not_wearing_gold(), player_no_gold);
        mob.living.brain_set(&keys::nearest_player_holding_wanted_item(), player_loved);
        let count = adult_piglins.len() as i32;
        mob.living.brain_set_value(&keys::nearby_adult_piglins(), nearby_adults);
        mob.living.brain_set_value(&keys::nearest_visible_adult_piglins(), adult_piglins);
        mob.living.brain_set_value(&keys::visible_adult_piglin_count(), count);
        mob.living.brain_set_value(&keys::visible_adult_hoglin_count(), adult_hoglin_count);
    }

    fn requires(&self) -> Vec<MMT> {
        vec![
            keys::nearest_visible_living_entities(),
            keys::nearest_living_entities(),
            keys::nearest_visible_nemesis(),
            keys::nearest_targetable_player_not_wearing_gold(),
            keys::nearest_player_holding_wanted_item(),
            keys::nearest_visible_huntable_hoglin(),
            keys::nearest_visible_baby_hoglin(),
            keys::nearest_visible_adult_piglins(),
            keys::nearby_adult_piglins(),
            keys::visible_adult_piglin_count(),
            keys::visible_adult_hoglin_count(),
            keys::nearest_repellent(),
        ]
    }
}

// ===================== PiglinBruteSpecificSensor =====================

/// `PiglinBruteSpecificSensor` — nemesis + nearby adults.
pub struct PiglinBruteSpecificSensor {
    state: SensorState,
}

impl PiglinBruteSpecificSensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::default_rate() }
    }
}

impl Sensor for PiglinBruteSpecificSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let nvle: Option<NearestVisibleLivingEntities> =
            mob.living.brain_get(&keys::nearest_visible_living_entities());
        let empty = NearestVisibleLivingEntities::empty();
        let nvle = nvle.as_ref().unwrap_or(&empty);
        let nemesis = nvle
            .find_closest(&|t| {
                let tid = crate::world::entity::entity_type::holder_type_id(&t.entity.entity_type)
                    .map(|i| i.path)
                    .unwrap_or_default();
                tid == "wither_skeleton" || tid == "wither"
            });

        let nearby: Vec<LivingEntity> = mob
            .living
            .brain_get(&keys::nearest_living_entities())
            .unwrap_or_default();
        let adults: Vec<LivingEntity> = nearby
            .into_iter()
            .filter(|e| {
                let tid = crate::world::entity::entity_type::holder_type_id(&e.entity.entity_type)
                    .map(|i| i.path)
                    .unwrap_or_default();
                (tid == "piglin" || tid == "piglin_brute") && e.is_adult()
            })
            .collect();

        mob.living.brain_set(&keys::nearest_visible_nemesis(), nemesis);
        mob.living.brain_set_value(&keys::nearby_adult_piglins(), adults);
    }

    fn requires(&self) -> Vec<MMT> {
        vec![
            keys::nearest_visible_living_entities(),
            keys::nearest_visible_nemesis(),
            keys::nearby_adult_piglins(),
        ]
    }
}

// ===================== HoglinSpecificSensor =====================

/// `HoglinSpecificSensor` — piglin/hoglin adults + repellent.
pub struct HoglinSpecificSensor {
    state: SensorState,
}

impl HoglinSpecificSensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::default_rate() }
    }
}

impl Sensor for HoglinSpecificSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let center = mob.living.entity.block_position();
        let repellent = BlockPos::find_closest_match(&center, 8, 4, &mut |pos| {
            level
                .get_block_state(*pos)
                .map(|s| {
                    crate::tags::is_block_in_named_tag(
                        &s.block.id,
                        &crate::tags::block_tags::BlockTags::hoglin_repellents(),
                    )
                })
                .unwrap_or(false)
        });
        mob.living.brain_set(&keys::nearest_repellent(), repellent);

        let nvle: Option<NearestVisibleLivingEntities> =
            mob.living.brain_get(&keys::nearest_visible_living_entities());
        let empty = NearestVisibleLivingEntities::empty();
        let nvle = nvle.as_ref().unwrap_or(&empty);
        let mut piglin: Option<LivingEntity> = None;
        let mut piglin_count = 0i32;
        let mut hoglins: Vec<LivingEntity> = Vec::new();
        for target in nvle.find_all(&|t| {
            let tid = crate::world::entity::entity_type::holder_type_id(&t.entity.entity_type)
                .map(|i| i.path)
                .unwrap_or_default();
            !t.entity.is_baby() && (tid == "piglin" || tid == "hoglin")
        }) {
            let tid = crate::world::entity::entity_type::holder_type_id(&target.entity.entity_type)
                .map(|i| i.path.clone())
                .unwrap_or_default();
            if tid == "piglin" {
                piglin_count += 1;
                if piglin.is_none() {
                    piglin = Some(target.clone());
                }
            } else if tid == "hoglin" {
                hoglins.push(target.clone());
            }
        }
        mob.living.brain_set(&keys::nearest_visible_adult_piglin(), piglin);
        mob.living.brain_set_value(&keys::nearest_visible_adult_hoglins(), hoglins.clone());
        mob.living.brain_set_value(&keys::visible_adult_piglin_count(), piglin_count);
        mob.living.brain_set_value(&keys::visible_adult_hoglin_count(), hoglins.len() as i32);
    }

    fn requires(&self) -> Vec<MMT> {
        vec![
            keys::nearest_visible_living_entities(),
            keys::nearest_repellent(),
            keys::nearest_visible_adult_piglin(),
            keys::nearest_visible_adult_hoglins(),
            keys::visible_adult_piglin_count(),
            keys::visible_adult_hoglin_count(),
        ]
    }
}

// ===================== NearestItemSensor =====================

/// `NearestItemSensor` — ближайший видимый ItemEntity,
/// который моб хочет подобрать → NEAREST_VISIBLE_WANTED_ITEM.
pub struct NearestItemSensor {
    state: SensorState,
}

/// `MAX_DISTANCE_TO_WANTED_ITEM` = 32 → distSq 1024.
const MAX_DIST_SQ: f64 = 32.0 * 32.0;

impl NearestItemSensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { state: SensorState::default_rate() }
    }
}

impl Sensor for NearestItemSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let pos = mob.living.entity.position();
        let bb = mob.living.entity.get_bounding_box().inflate(32.0, 16.0, 32.0);
        let mut items: Vec<Entity> = level
            .entities_in_box(&bb)
            .into_iter()
            .filter(|e| {
                e.distance_to_sqr_pos(pos.x, pos.y, pos.z) < MAX_DIST_SQ
                    && e.entity_type_is_id("item")
            })
            .collect();
        items.sort_by(|a, b| {
            a.distance_to_sqr_pos(pos.x, pos.y, pos.z)
                .partial_cmp(&b.distance_to_sqr_pos(pos.x, pos.y, pos.z))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        // wantsToPickUp + hasLineOfSight фильтр; ItemStack берём
        // через entity-hook `item_stack()` (ItemEntity-данные).
        let mut nearest: Option<Entity> = None;
        for item in items {
            let stack = item.item_entity_stack().unwrap_or_else(ItemStack::empty);
            if mob.wants_to_pick_up(level, &stack) && mob.has_line_of_sight(&item) {
                nearest = Some(item);
                break;
            }
        }
        mob.living
            .brain_set(&keys::nearest_visible_wanted_item(), nearest);
    }

    fn requires(&self) -> Vec<MMT> {
        vec![keys::nearest_visible_wanted_item()]
    }
}

// ===================== TemptingSensor =====================

/// `TemptingSensor` — TEMPTING_PLAYER по item-темптации.
/// `temptations` — `BiPredicate<PathfinderMob, ItemStack>`;
/// варианты forAnimal/axolotl/goat/... через конструкторы.
pub struct TemptingSensor {
    state: SensorState,
    /// `temptations(mob, stack)`.
    temptations: Rc<dyn Fn(&Mob, &ItemStack) -> bool>,
}

/// Статические TEMPT_TARGETING условия (копируются с range).
fn tempt_targeting(range: f64) -> TargetingConditions {
    let mut c = TargetingConditions::for_non_combat();
    c.ignore_line_of_sight();
    c.range(range);
    c
}

impl TemptingSensor {
    /// `new(temptations)` — BiPredicate-вариант.
    pub fn new(
        temptations: impl Fn(&Mob, &ItemStack) -> bool + 'static,
    ) -> Self {
        Self {
            state: SensorState::default_rate(),
            temptations: Rc::new(temptations),
        }
    }

    /// `forAnimal()` — `Animal::isFood(stack)`.
    pub fn for_animal() -> Self {
        Self::new(|mob, stack| mob.living.animal_is_food(stack))
    }

    /// `AXOLOTL_TEMPTATIONS` — axolotl food items.
    pub fn axolotl() -> Self {
        Self::new(|_mob, stack| {
            stack.is(&crate::resources::identifier::Identifier::minecraft("tropical_fish"))
                || stack.is(&crate::resources::identifier::Identifier::minecraft("pufferfish"))
                || stack.is(&crate::resources::identifier::Identifier::minecraft("salmon"))
        })
    }

    /// `item_path(stack)` -> path-строка предмета ("" если пусто).
    fn item_path(stack: &ItemStack) -> String {
        stack
            .item_identifier()
            .map(|i| i.path)
            .unwrap_or_default()
    }

    /// `GOAT_TEMPTATIONS` — goat food.
    pub fn goat() -> Self {
        Self::new(|_mob, stack| {
            matches!(
                Self::item_path(stack).as_str(),
                "wheat" | "hay_block" | "carrot" | "potato" | "beetroot"
                    | "torchflower_seeds" | "pitcher_pod"
            )
        })
    }

    /// `CAMEL_TEMPTATIONS`.
    pub fn camel() -> Self {
        Self::new(|_mob, stack| {
            matches!(Self::item_path(stack).as_str(), "cactus" | "wheat" | "hay_block")
        })
    }

    /// `ARMADILLO_TEMPTATIONS` — spider_eyes/seeds.
    pub fn armadillo() -> Self {
        Self::new(|_mob, stack| {
            matches!(
                Self::item_path(stack).as_str(),
                "spider_eye" | "sweet_berries" | "glow_berries" | "torchflower_seeds"
            )
        })
    }

    /// `SNIFFER_TEMPTATIONS` — torchflower/pitcher pod.
    pub fn sniffer() -> Self {
        Self::new(|_mob, stack| {
            matches!(Self::item_path(stack).as_str(), "torchflower_seeds" | "pitcher_pod")
        })
    }

    /// `HAPPY_GHAST_TEMPTATIONS` — ghast food.
    pub fn happy_ghast() -> Self {
        Self::new(|_mob, stack| {
            matches!(Self::item_path(stack).as_str(), "snowball" | "powder_snow_bucket")
        })
    }

    /// `FROG_TEMPTATIONS` — slimeballs.
    pub fn frog() -> Self {
        Self::new(|_mob, stack| Self::item_path(stack) == "slime_ball")
    }

    /// `NAUTILUS_TEMPTATIONS` — nautilus food.
    pub fn nautilus() -> Self {
        Self::new(|_mob, stack| {
            matches!(Self::item_path(stack).as_str(), "clownfish" | "tropical_fish")
        })
    }

    /// `playerHoldingTemptation(mob, player)`.
    fn player_holding_temptation(&self, mob: &Mob, player: &LivingEntity) -> bool {
        let main = player.get_main_hand_item_stack().clone();
        let off = player.get_offhand_item().clone();
        (self.temptations)(mob, &main) || (self.temptations)(mob, &off)
    }
}

impl Sensor for TemptingSensor {
    fn scan_rate(&self) -> i32 { self.state.scan_rate }
    fn time_to_tick(&self) -> i64 { self.state.time_to_tick }
    fn set_time_to_tick(&mut self, v: i64) { self.state.time_to_tick = v; }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        let range = mob
            .living
            .get_attribute_value_by_id_ro(&tempt_range());
        // TEMPT_TARGETING.copy().range(temptRange)
        let mut cond = tempt_targeting(range);
        let pos = mob.living.entity.position();
        let reach = range.max(1.0);
        let bb = mob.living.entity.get_bounding_box().inflate(reach, reach, reach);
        let mut candidates: Vec<LivingEntity> = level
            .entities_in_box(&bb)
            .into_iter()
            .filter(|e| e.is_living_entity())
            .filter(|e| !e.is_spectator())
            .filter(|e| entity_type_is(&LivingEntity::from_entity(e.clone()), "player"))
            .map(LivingEntity::from_entity)
            .collect();
        // targetingConditions.test + playerHoldingTemptation +
        // !hasPassenger — entity==null-ветка test (non-combat).
        candidates.retain(|p| {
            cond.test(level, None, p)
                && self.player_holding_temptation(mob, p)
                && !mob.living.entity.has_passenger(&p.entity)
        });
        if candidates.is_empty() {
            mob.living.brain_erase(&keys::tempting_player());
            return;
        }
        candidates.sort_by(|a, b| {
            a.entity
                .distance_to_sqr_pos(pos.x, pos.y, pos.z)
                .partial_cmp(&b.entity.distance_to_sqr_pos(pos.x, pos.y, pos.z))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        mob.living
            .brain_set_value(&keys::tempting_player(), candidates.remove(0));
    }

    fn requires(&self) -> Vec<MMT> {
        vec![keys::tempting_player()]
    }
}

// ===================== BreezeAttackEntitySensor =====================

/// `BreezeAttackEntitySensor` — NEAREST_ATTACKABLE из
/// NEAREST_LIVING_ENTITIES (non-spectator + attackable).
pub struct BreezeAttackEntitySensor {
    inner: NearestLivingEntitySensor,
}

impl BreezeAttackEntitySensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { inner: NearestLivingEntitySensor::new() }
    }
}

impl Sensor for BreezeAttackEntitySensor {
    fn scan_rate(&self) -> i32 { self.inner.scan_rate() }
    fn time_to_tick(&self) -> i64 { self.inner.time_to_tick() }
    fn set_time_to_tick(&mut self, v: i64) { self.inner.set_time_to_tick(v) }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        self.inner.do_tick(level, mob);
        let entities: Vec<LivingEntity> = mob
            .living
            .brain_get(&keys::nearest_living_entities())
            .unwrap_or_default();
        let found = entities
            .iter()
            .find(|e| {
                !e.is_spectator()
                    && is_entity_attackable(level, &mob.living, e)
            })
            .cloned();
        match found {
            Some(e) => mob.living.brain_set_value(&keys::nearest_attackable(), e),
            None => mob.living.brain_erase(&keys::nearest_attackable()),
        }
    }

    fn requires(&self) -> Vec<MMT> {
        let mut r = self.inner.requires();
        r.push(keys::nearest_attackable());
        r
    }
}

// ===================== WardenEntitySensor =====================

/// `WardenEntitySensor` — NEAREST_ATTACKABLE: сначала player,
/// потом не-player (по NEAREST_LIVING_ENTITIES + canTargetEntity).
pub struct WardenEntitySensor {
    inner: NearestLivingEntitySensor,
}

impl WardenEntitySensor {
    /// `new()`.
    pub fn new() -> Self {
        Self { inner: NearestLivingEntitySensor::new() }
    }
}

impl Sensor for WardenEntitySensor {
    fn scan_rate(&self) -> i32 { self.inner.scan_rate() }
    fn time_to_tick(&self) -> i64 { self.inner.time_to_tick() }
    fn set_time_to_tick(&mut self, v: i64) { self.inner.set_time_to_tick(v) }

    fn do_tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) {
        self.inner.do_tick(level, mob);
        let entities: Vec<LivingEntity> = mob
            .living
            .brain_get(&keys::nearest_living_entities())
            .unwrap_or_default();
        let get_closest = |pred: &dyn Fn(&LivingEntity) -> bool| -> Option<LivingEntity> {
            entities
                .iter()
                .filter(|e| mob.living.warden_can_target_entity(e))
                .find(|e| pred(e))
                .cloned()
        };
        let found = get_closest(&|e| entity_type_is(e, "player"))
            .or_else(|| get_closest(&|e| !entity_type_is(e, "player")));
        match found {
            Some(e) => mob.living.brain_set_value(&keys::nearest_attackable(), e),
            None => mob.living.brain_erase(&keys::nearest_attackable()),
        }
    }

    fn requires(&self) -> Vec<MMT> {
        let mut r = self.inner.requires();
        r.push(keys::nearest_attackable());
        r
    }
}
