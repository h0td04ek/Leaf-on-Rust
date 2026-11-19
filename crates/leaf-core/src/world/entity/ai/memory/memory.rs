//! Перенос `net.minecraft.world.entity.ai.memory` —
//! `ExpirableValue`, `MemoryStatus`, `WalkTarget`,
//! `MemoryModuleType` (реестр всех типов памяти Brain),
//! `NearestVisibleLivingEntities`.
//!
//! Java дженерики `MemoryModuleType<U>` в Rust стираются до
//! type-id: значение хранится как `MemoryValue` (enum-обёртка
//! над Box<dyn Any>), доступ — через `MemoryStore::get_typed`.

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::global_pos::GlobalPos;
use crate::core::section_pos::SectionPos;
use crate::resources::identifier::Identifier;
use crate::util::uuid::Uuid;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::entity::entity::Entity;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::level::pathfinder::path::Path;
use crate::world::phys::vec3::Vec3;

/// `ExpirableValue<T>` — значение с TTL в тиках; `Long.MAX_VALUE`
/// = никогда не истекает.
///
/// Java разделяет ссылку на один объект между Brain-копиями
/// (`copyWithoutBehaviors`) — здесь `Rc<RefCell<Inner>>` даёт
/// ту же shared-семантику: `Clone` делит value+TTL.
#[derive(Clone)]
pub struct ExpirableValue {
    inner: std::rc::Rc<std::cell::RefCell<ExpirableInner>>,
}

struct ExpirableInner {
    /// `value` — type-erased значение.
    value: std::rc::Rc<dyn Any>,
    /// `timeToLive`.
    time_to_live: i64,
}

impl ExpirableValue {
    /// `new(value, timeToLive)`.
    pub fn new(value: impl Any + 'static, time_to_live: i64) -> Self {
        Self {
            inner: std::rc::Rc::new(std::cell::RefCell::new(ExpirableInner {
                value: std::rc::Rc::new(value),
                time_to_live,
            })),
        }
    }

    /// `of(value)` — TTL = Long.MAX_VALUE.
    pub fn of(value: impl Any + 'static) -> Self {
        Self::new(value, i64::MAX)
    }

    /// `of(value, timeToLive)`.
    pub fn of_ttl(value: impl Any + 'static, time_to_live: i64) -> Self {
        Self::new(value, time_to_live)
    }

    /// `tick()` — декремент TTL, если значение может истечь.
    pub fn tick(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.time_to_live != i64::MAX {
            inner.time_to_live -= 1;
        }
    }

    /// `getTimeToLive()`.
    pub fn get_time_to_live(&self) -> i64 {
        self.inner.borrow().time_to_live
    }

    /// `hasExpired()`.
    pub fn has_expired(&self) -> bool {
        self.get_time_to_live() <= 0
    }

    /// `canExpire()` — `@VisibleForDebug`.
    pub fn can_expire(&self) -> bool {
        self.get_time_to_live() != i64::MAX
    }

    /// `getValue()` с даункастом до `T`.
    pub fn get_value<T: Any + 'static>(&self) -> Option<T>
    where
        T: Clone,
    {
        self.inner
            .borrow()
            .value
            .downcast_ref::<T>()
            .cloned()
    }

    /// `getValueRef` — ссылка на type-erased значение (только
    /// внутри callback, RefCell уже заимствован).
    pub fn with_value<R>(&self, f: impl FnOnce(&dyn Any) -> R) -> R {
        f(&*self.inner.borrow().value)
    }

    /// Тип значения (TypeId).
    pub fn value_type_id(&self) -> TypeId {
        self.inner.borrow().value.type_id()
    }
}

/// `MemoryStatus` — статус памяти в `MemoryModuleType.checkMemory`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MemoryStatus {
    /// `VALUE_PRESENT`.
    ValuePresent,
    /// `VALUE_ABSENT`.
    ValueAbsent,
    /// `REGISTERED`.
    Registered,
}

/// `PositionTracker` — трекер позиции для `WalkTarget`/
/// `LOOK_TARGET` (BlockPos/Entity/Memory-варианты).
pub trait PositionTracker {
    /// `currentPosition()`.
    fn current_position(&self) -> Vec3;
    /// `currentBlockPosition()`.
    fn current_block_position(&self) -> BlockPos;
    /// `isVisibleBy(entity)`.
    fn is_visible_by(&self, entity: &LivingEntity) -> bool;
}

/// `BlockPosTracker` — статичная позиция.
pub struct BlockPosTracker {
    block_pos: BlockPos,
    center_position: Vec3,
}

impl BlockPosTracker {
    /// `new(blockPos)`.
    pub fn new(block_pos: BlockPos) -> Self {
        Self {
            block_pos,
            center_position: block_pos.get_center(),
        }
    }

    /// `new(centerPosition)` — BlockPos из Vec3.
    pub fn of_vec3(center_position: Vec3) -> Self {
        Self {
            block_pos: BlockPos::containing(center_position.x, center_position.y, center_position.z),
            center_position,
        }
    }

    /// `getBlockPos()`.
    pub fn block_pos(&self) -> BlockPos {
        self.block_pos
    }
}

impl PositionTracker for BlockPosTracker {
    fn current_position(&self) -> Vec3 {
        self.center_position
    }
    fn current_block_position(&self) -> BlockPos {
        self.block_pos
    }
    fn is_visible_by(&self, _entity: &LivingEntity) -> bool {
        true
    }
}

/// `EntityTracker` — трекер за сущностью (UUID-снапшот +
/// резолв через уровень при необходимости).
pub struct EntityTracker {
    /// `entity` — снапшот (Entity клонируется при создании).
    pub entity: Entity,
    track_eye_height: bool,
    target_eye_height: bool,
}

impl EntityTracker {
    /// `new(entity, trackEyeHeight)`.
    pub fn new(entity: Entity, track_eye_height: bool) -> Self {
        Self::full(entity, track_eye_height, false)
    }

    /// `new(entity, trackEyeHeight, targetEyeHeight)`.
    pub fn full(entity: Entity, track_eye_height: bool, target_eye_height: bool) -> Self {
        Self {
            entity,
            track_eye_height,
            target_eye_height,
        }
    }

    /// `getEntity()`.
    pub fn get_entity(&self) -> &Entity {
        &self.entity
    }

    /// `getEntityUUID()`.
    pub fn get_entity_uuid(&self) -> Uuid {
        self.entity.uuid
    }
}

impl PositionTracker for EntityTracker {
    fn current_position(&self) -> Vec3 {
        if self.track_eye_height {
            self.entity.position().add(0.0, self.entity.get_eye_height() as f64, 0.0)
        } else {
            self.entity.position()
        }
    }
    fn current_block_position(&self) -> BlockPos {
        if self.target_eye_height {
            BlockPos::containing(self.entity.get_eye_position().x, self.entity.get_eye_position().y, self.entity.get_eye_position().z)
        } else {
            self.entity.block_position()
        }
    }
    fn is_visible_by(&self, entity: &LivingEntity) -> bool {
        if self.entity.is_living_entity() {
            let target = LivingEntity::from_entity(self.entity.clone());
            if !target.is_alive() {
                return false;
            }
            // `getBrain().getMemory(NEAREST_VISIBLE_LIVING_ENTITIES)` —
            // у нас brain может отсутствовать; проверяем через
            // memory-хук на LivingEntity.
            entity
                .brain_nearest_visible_contains(&target)
                .unwrap_or(false)
        } else {
            true
        }
    }
}

/// `WalkTarget` — цель ходьбы с модификатором скорости и
/// дистанцией "достаточно близко".
#[derive(Clone)]
pub struct WalkTarget {
    /// `target` — Rc для Clone (Brain.getMemory клонирует).
    pub target: std::rc::Rc<dyn PositionTracker>,
    /// `speedModifier`.
    pub speed_modifier: f32,
    /// `closeEnoughDist`.
    pub close_enough_dist: i32,
}

impl WalkTarget {
    /// `new(pos, speedModifier, closeEnoughDist)`.
    pub fn new_block(block_pos: BlockPos, speed_modifier: f32, close_enough_dist: i32) -> Self {
        Self::of_tracker(std::rc::Rc::new(BlockPosTracker::new(block_pos)), speed_modifier, close_enough_dist)
    }

    /// `new(Vec3, speedModifier, closeEnoughDist)`.
    pub fn new_vec3(vector_pos: Vec3, speed_modifier: f32, close_enough_dist: i32) -> Self {
        Self::of_tracker(std::rc::Rc::new(BlockPosTracker::of_vec3(vector_pos)), speed_modifier, close_enough_dist)
    }

    /// `new(entity, speedModifier, closeEnoughDist)`.
    pub fn new_entity(entity: Entity, speed_modifier: f32, close_enough_dist: i32) -> Self {
        Self::of_tracker(std::rc::Rc::new(EntityTracker::new(entity, false)), speed_modifier, close_enough_dist)
    }

    /// `new(target, speedModifier, closeEnoughDist)`.
    pub fn of_tracker(target: std::rc::Rc<dyn PositionTracker>, speed_modifier: f32, close_enough_dist: i32) -> Self {
        Self {
            target,
            speed_modifier,
            close_enough_dist,
        }
    }

    /// `getTarget()`.
    pub fn get_target(&self) -> &dyn PositionTracker {
        self.target.as_ref()
    }

    /// `getSpeedModifier()`.
    pub fn get_speed_modifier(&self) -> f32 {
        self.speed_modifier
    }

    /// `getCloseEnoughDist()`.
    pub fn get_close_enough_dist(&self) -> i32 {
        self.close_enough_dist
    }
}

/// `MemoryModuleType<U>` — тип памяти Brain. Java-реестр
/// `BuiltInRegistries.MEMORY_MODULE_TYPE`; здесь — ключ-идентификатор
/// + фабрики для всех ~100 vanilla-ключей.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemoryModuleType {
    /// Регистрационный id (`minecraft:home` и т.д.).
    pub id: Identifier,
    /// `codec != null` — Java хранит `Optional<Codec<ExpirableValue<U>>>`;
    /// у нас кодеки NBT-части Brain (большинство типов без кодека).
    pub has_codec: bool,
}

impl MemoryModuleType {
    /// `new MemoryModuleType(optionalCodec)` — приватный конструктор
    /// Java; `has_codec` отражает наличие кодека.
    pub fn new(id: &str, has_codec: bool) -> Self {
        Self {
            id: Identifier::minecraft(id),
            has_codec,
        }
    }

    /// `toString()` — ключ реестра.
    pub fn to_string(&self) -> String {
        self.id.to_string()
    }

    /// Стабильный хэш-ключ для HashMap.
    pub fn key(&self) -> &Identifier {
        &self.id
    }
}

/// Реестр всех vanilla `MemoryModuleType` (порядок = Java-поля).
pub mod keys {
    use super::MemoryModuleType;
    use crate::resources::identifier::Identifier;

    macro_rules! mem_key {
        ($name:ident, $id:literal, $codec:expr) => {
            /// `MemoryModuleType::$name`.
            pub fn $name() -> MemoryModuleType {
                MemoryModuleType { id: Identifier::minecraft($id), has_codec: $codec }
            }
        };
    }

    mem_key!(dummy, "dummy", false);
    mem_key!(home, "home", true);
    mem_key!(job_site, "job_site", true);
    mem_key!(potential_job_site, "potential_job_site", true);
    mem_key!(meeting_point, "meeting_point", true);
    mem_key!(secondary_job_site, "secondary_job_site", false);
    mem_key!(nearest_living_entities, "mobs", false);
    mem_key!(nearest_visible_living_entities, "visible_mobs", false);
    mem_key!(visible_villager_babies, "visible_villager_babies", false);
    mem_key!(nearest_players, "nearest_players", false);
    mem_key!(nearest_visible_player, "nearest_visible_player", false);
    mem_key!(nearest_visible_attackable_player, "nearest_visible_targetable_player", false);
    mem_key!(nearest_visible_attackable_players, "nearest_visible_targetable_players", false);
    mem_key!(walk_target, "walk_target", false);
    mem_key!(look_target, "look_target", false);
    mem_key!(attack_target, "attack_target", false);
    mem_key!(attack_cooling_down, "attack_cooling_down", false);
    mem_key!(interaction_target, "interaction_target", false);
    mem_key!(breed_target, "breed_target", false);
    mem_key!(ride_target, "ride_target", false);
    mem_key!(path, "path", false);
    mem_key!(interactable_doors, "interactable_doors", false);
    mem_key!(doors_to_close, "doors_to_close", false);
    mem_key!(nearest_bed, "nearest_bed", false);
    mem_key!(hurt_by, "hurt_by", false);
    mem_key!(hurt_by_entity, "hurt_by_entity", false);
    mem_key!(avoid_target, "avoid_target", false);
    mem_key!(nearest_hostile, "nearest_hostile", false);
    mem_key!(nearest_attackable, "nearest_attackable", false);
    mem_key!(hiding_place, "hiding_place", false);
    mem_key!(heard_bell_time, "heard_bell_time", false);
    mem_key!(cant_reach_walk_target_since, "cant_reach_walk_target_since", false);
    mem_key!(golem_detected_recently, "golem_detected_recently", true);
    mem_key!(danger_detected_recently, "danger_detected_recently", true);
    mem_key!(last_slept, "last_slept", true);
    mem_key!(last_woken, "last_woken", true);
    mem_key!(last_worked_at_poi, "last_worked_at_poi", true);
    mem_key!(nearest_visible_adult, "nearest_visible_adult", false);
    mem_key!(nearest_visible_wanted_item, "nearest_visible_wanted_item", false);
    mem_key!(nearest_visible_nemesis, "nearest_visible_nemesis", false);
    mem_key!(play_dead_ticks, "play_dead_ticks", true);
    mem_key!(tempting_player, "tempting_player", false);
    mem_key!(temptation_cooldown_ticks, "temptation_cooldown_ticks", true);
    mem_key!(gaze_cooldown_ticks, "gaze_cooldown_ticks", true);
    mem_key!(is_tempted, "is_tempted", true);
    mem_key!(long_jump_cooldown_ticks, "long_jump_cooling_down", true);
    mem_key!(long_jump_mid_jump, "long_jump_mid_jump", false);
    mem_key!(has_hunting_cooldown, "has_hunting_cooldown", false);
    mem_key!(ram_cooldown_ticks, "ram_cooldown_ticks", false);
    mem_key!(ram_target, "ram_target", false);
    mem_key!(is_in_water, "is_in_water", true);
    mem_key!(is_pregnant, "is_pregnant", true);
    mem_key!(is_panicking, "is_panicking", true);
    mem_key!(unreachable_tongue_targets, "unreachable_tongue_targets", false);
    mem_key!(visited_block_positions, "visited_block_positions", false);
    mem_key!(unreachable_transport_block_positions, "unreachable_transport_block_positions", false);
    mem_key!(transport_items_cooldown_ticks, "transport_items_cooldown_ticks", false);
    mem_key!(charge_cooldown_ticks, "charge_cooldown_ticks", true);
    mem_key!(attack_target_cooldown, "attack_target_cooldown", true);
    mem_key!(spear_fleeing_time, "spear_fleeing_time", false);
    mem_key!(spear_fleeing_position, "spear_fleeing_position", false);
    mem_key!(spear_charge_position, "spear_charge_position", false);
    mem_key!(spear_engage_time, "spear_engage_time", false);
    mem_key!(spear_status, "spear_status", false);
    mem_key!(angry_at, "angry_at", true);
    mem_key!(universal_anger, "universal_anger", true);
    mem_key!(admiring_item, "admiring_item", true);
    mem_key!(time_trying_to_reach_admire_item, "time_trying_to_reach_admire_item", false);
    mem_key!(disable_walk_to_admire_item, "disable_walk_to_admire_item", false);
    mem_key!(admiring_disabled, "admiring_disabled", false);
    mem_key!(hunted_recently, "hunted_recently", false);
    mem_key!(celebrate_location, "celebrate_location", false);
    mem_key!(dancing, "dancing", false);
    mem_key!(nearest_visible_huntable_hoglin, "nearest_visible_huntable_hoglin", false);
    mem_key!(nearest_visible_baby_hoglin, "nearest_visible_baby_hoglin", false);
    mem_key!(nearest_targetable_player_not_wearing_gold, "nearest_targetable_player_not_wearing_gold", false);
    mem_key!(nearby_adult_piglins, "nearby_adult_piglins", false);
    mem_key!(nearest_visible_adult_piglins, "nearest_visible_adult_piglins", false);
    mem_key!(nearest_visible_adult_hoglins, "nearest_visible_adult_hoglins", false);
    mem_key!(nearest_visible_adult_piglin, "nearest_visible_adult_piglin", false);
    mem_key!(nearest_visible_zombified, "nearest_visible_zombified", false);
    mem_key!(visible_adult_piglin_count, "visible_adult_piglin_count", false);
    mem_key!(visible_adult_hoglin_count, "visible_adult_hoglin_count", false);
    mem_key!(nearest_player_holding_wanted_item, "nearest_player_holding_wanted_item", false);
    mem_key!(ate_recently, "ate_recently", false);
    mem_key!(nearest_repellent, "nearest_repellent", false);
    mem_key!(pacified, "pacified", false);
    mem_key!(roar_target, "roar_target", false);
    mem_key!(disturbance_location, "disturbance_location", false);
    mem_key!(recent_projectile, "recent_projectile", true);
    mem_key!(is_sniffing, "is_sniffing", true);
    mem_key!(is_emerging, "is_emerging", true);
    mem_key!(roar_sound_delay, "roar_sound_delay", true);
    mem_key!(dig_cooldown, "dig_cooldown", true);
    mem_key!(roar_sound_cooldown, "roar_sound_cooldown", true);
    mem_key!(sniff_cooldown, "sniff_cooldown", true);
    mem_key!(touch_cooldown, "touch_cooldown", true);
    mem_key!(vibration_cooldown, "vibration_cooldown", true);
    mem_key!(sonic_boom_cooldown, "sonic_boom_cooldown", true);
    mem_key!(sonic_boom_sound_cooldown, "sonic_boom_sound_cooldown", true);
    mem_key!(sonic_boom_sound_delay, "sonic_boom_sound_delay", true);
    mem_key!(liked_player, "liked_player", true);
    mem_key!(liked_noteblock_position, "liked_noteblock", true);
    mem_key!(liked_noteblock_cooldown_ticks, "liked_noteblock_cooldown_ticks", true);
    mem_key!(item_pickup_cooldown_ticks, "item_pickup_cooldown_ticks", true);
    mem_key!(sniffer_explored_positions, "sniffer_explored_positions", false);
    mem_key!(sniffer_sniffing_target, "sniffer_sniffing_target", false);
    mem_key!(sniffer_digging, "sniffer_digging", false);
    mem_key!(sniffer_happy, "sniffer_happy", false);
    mem_key!(breeze_jump_cooldown, "breeze_jump_cooldown", true);
    mem_key!(breeze_shoot, "breeze_shoot", true);
    mem_key!(breeze_shoot_charging, "breeze_shoot_charging", true);
    mem_key!(breeze_shoot_recovering, "breeze_shoot_recover", true);
    mem_key!(breeze_shoot_cooldown, "breeze_shoot_cooldown", true);
    mem_key!(breeze_jump_inhaling, "breeze_jump_inhaling", true);
    mem_key!(breeze_jump_target, "breeze_jump_target", true);
    mem_key!(breeze_leaving_water, "breeze_leaving_water", true);

    /// Полный список всех зарегистрированных типов (для
    /// `Brain`-сериализации и отладки).
    pub fn all() -> Vec<MemoryModuleType> {
        vec![
            dummy(), home(), job_site(), potential_job_site(), meeting_point(),
            secondary_job_site(), nearest_living_entities(), nearest_visible_living_entities(),
            visible_villager_babies(), nearest_players(), nearest_visible_player(),
            nearest_visible_attackable_player(), nearest_visible_attackable_players(),
            walk_target(), look_target(), attack_target(), attack_cooling_down(),
            interaction_target(), breed_target(), ride_target(), path(),
            interactable_doors(), doors_to_close(), nearest_bed(), hurt_by(),
            hurt_by_entity(), avoid_target(), nearest_hostile(), nearest_attackable(),
            hiding_place(), heard_bell_time(), cant_reach_walk_target_since(),
            golem_detected_recently(), danger_detected_recently(), last_slept(),
            last_woken(), last_worked_at_poi(), nearest_visible_adult(),
            nearest_visible_wanted_item(), nearest_visible_nemesis(), play_dead_ticks(),
            tempting_player(), temptation_cooldown_ticks(), gaze_cooldown_ticks(),
            is_tempted(), long_jump_cooldown_ticks(), long_jump_mid_jump(),
            has_hunting_cooldown(), ram_cooldown_ticks(), ram_target(), is_in_water(),
            is_pregnant(), is_panicking(), unreachable_tongue_targets(),
            visited_block_positions(), unreachable_transport_block_positions(),
            transport_items_cooldown_ticks(), charge_cooldown_ticks(),
            attack_target_cooldown(), spear_fleeing_time(), spear_fleeing_position(),
            spear_charge_position(), spear_engage_time(), spear_status(), angry_at(),
            universal_anger(), admiring_item(), time_trying_to_reach_admire_item(),
            disable_walk_to_admire_item(), admiring_disabled(), hunted_recently(),
            celebrate_location(), dancing(), nearest_visible_huntable_hoglin(),
            nearest_visible_baby_hoglin(), nearest_targetable_player_not_wearing_gold(),
            nearby_adult_piglins(), nearest_visible_adult_piglins(),
            nearest_visible_adult_hoglins(), nearest_visible_adult_piglin(),
            nearest_visible_zombified(), visible_adult_piglin_count(),
            visible_adult_hoglin_count(), nearest_player_holding_wanted_item(),
            ate_recently(), nearest_repellent(), pacified(), roar_target(),
            disturbance_location(), recent_projectile(), is_sniffing(), is_emerging(),
            roar_sound_delay(), dig_cooldown(), roar_sound_cooldown(), sniff_cooldown(),
            touch_cooldown(), vibration_cooldown(), sonic_boom_cooldown(),
            sonic_boom_sound_cooldown(), sonic_boom_sound_delay(), liked_player(),
            liked_noteblock_position(), liked_noteblock_cooldown_ticks(),
            item_pickup_cooldown_ticks(), sniffer_explored_positions(),
            sniffer_sniffing_target(), sniffer_digging(), sniffer_happy(),
            breeze_jump_cooldown(), breeze_shoot(), breeze_shoot_charging(),
            breeze_shoot_recovering(), breeze_shoot_cooldown(), breeze_jump_inhaling(),
            breeze_jump_target(), breeze_leaving_water(),
        ]
    }
}

/// `NearestVisibleLivingEntities` — список видимых сущностей +
/// кэш LOS-проверок (`lineOfSightTest`).
pub struct NearestVisibleLivingEntities {
    /// `nearbyEntities` — снапшоты LivingEntity.
    pub nearby_entities: Vec<LivingEntity>,
    /// `lineOfSightTest` — кэш `computeIfAbsent` по UUID.
    /// `Rc<RefCell>` — shared-семантика Java-объекта при
    /// клонировании из памяти Brain.
    pub los_cache: std::rc::Rc<std::cell::RefCell<HashMap<Uuid, bool>>>,
    /// Уровень для LOS-проверок (нужен `isEntityTargetable`).
    pub level: Option<Arc<dyn crate::world::level::level_reader::LevelReader>>,
    /// Владелец (чьи глаза/взгляд используются).
    pub owner: Option<LivingEntity>,
}

impl NearestVisibleLivingEntities {
    /// `empty()`.
    pub fn empty() -> Self {
        Self {
            nearby_entities: Vec::new(),
            los_cache: std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
            level: None,
            owner: None,
        }
    }

    /// `new(level, entity, nearbyEntities)` — строит LOS-тест через
    /// `Sensor::isEntityTargetable`.
    pub fn new(
        level: Arc<dyn crate::world::level::level_reader::LevelReader>,
        entity: &LivingEntity,
        nearby_entities: Vec<LivingEntity>,
    ) -> Self {
        Self {
            nearby_entities,
            los_cache: std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
            level: Some(level),
            owner: Some(entity.clone()),
        }
    }

    /// `lineOfSightTest.test(target)` — с кэшем по UUID.
    pub fn line_of_sight_test(&self, target: &LivingEntity) -> bool {
        let uuid = target.get_uuid();
        if let Some(&v) = self.los_cache.borrow().get(&uuid) {
            return v;
        }
        let result = match (&self.level, &self.owner) {
            (Some(l), Some(o)) => crate::world::entity::ai::sensing::sensor::is_entity_targetable(
                l,
                o,
                target,
            ),
            _ => false,
        };
        self.los_cache.borrow_mut().insert(uuid, result);
        result
    }

    /// `findClosest(predicate)` — первый прошедший predicate+LOS
    /// (Leaf-оптимизация: ранний выход).
    pub fn find_closest(&self, predicate: &dyn Fn(&LivingEntity) -> bool) -> Option<LivingEntity> {
        for e in &self.nearby_entities {
            if predicate(e) && self.line_of_sight_test(e) {
                return Some(e.clone());
            }
        }
        None
    }

    /// `findAll(predicate)` — все прошедшие predicate+LOS.
    pub fn find_all(&self, predicate: &dyn Fn(&LivingEntity) -> bool) -> Vec<LivingEntity> {
        self.nearby_entities
            .iter()
            .filter(|t| predicate(t) && self.line_of_sight_test(t))
            .cloned()
            .collect()
    }

    /// `contains(entity)` — в списке и видим.
    pub fn contains(&self, entity: &LivingEntity) -> bool {
        let uuid = entity.get_uuid();
        self.nearby_entities.iter().any(|e| e.get_uuid() == uuid)
            && self.line_of_sight_test(entity)
    }

    /// `contains(predicate)`.
    pub fn contains_predicate(&self, predicate: &dyn Fn(&LivingEntity) -> bool) -> bool {
        self.nearby_entities
            .iter()
            .any(|e| predicate(e) && self.line_of_sight_test(e))
    }
}

impl Clone for NearestVisibleLivingEntities {
    fn clone(&self) -> Self {
        Self {
            nearby_entities: self.nearby_entities.clone(),
            los_cache: self.los_cache.clone(),
            level: self.level.clone(),
            owner: self.owner.clone(),
        }
    }
}

/// `MemoryModuleType.checkMemory`-статусы используются
/// `Behavior`-классами; хранилище — в `Brain`.
#[allow(dead_code)]
pub fn memory_status_for(
    status: MemoryStatus,
    _ty: &MemoryModuleType,
) -> MemoryStatus {
    status
}
