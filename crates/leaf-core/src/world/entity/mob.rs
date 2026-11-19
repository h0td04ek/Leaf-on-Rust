//! Перенос `net.minecraft.world.entity.Mob` — мобильная сущность
//! с AI (goal/target selectors, navigation, controls, sensing).
//!
//! `MobPathing` — трейт-адаптер для pathfinding (`&dyn MobPathing`),
//! чтобы pathfinder не зависел от конкретной `Mob` (аналог
//! `Mob`-параметров в Java NodeEvaluator/PathFinder).

use crate::core::block_pos::BlockPos;
use crate::random::{RandomSource, SingleThreadedRandomSource};
use crate::world::entity::ai::control::{
    BodyRotationControl, FlyingMoveControl, JumpControl, LookControl, MoveControl,
};
use crate::world::entity::ai::goal::GoalSelector;
use crate::world::entity::ai::sensing::Sensing;
use crate::world::entity::entity::Entity;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::leash_data::LeashData;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::vec3::Vec3;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// `MobPathing` — минимальный интерфейс моба для pathfinding/AI.
///
/// Все методы `&self` —evaluators работают с иммутабельной ссылкой
/// (в Java `mob` — обычная ссылка; мутации malus идут через
/// `set_pathfinding_malus` с `&self` + interior mutability).
pub trait MobPathing {
    /// `getBbWidth`/`getBbHeight`.
    fn bb_width(&self) -> f32;
    fn bb_height(&self) -> f32;

    /// `getX`/`getY`/`getZ`.
    fn get_x(&self) -> f64;
    fn get_y(&self) -> f64;
    fn get_z(&self) -> f64;

    /// `getBlockY`.
    fn get_block_y(&self) -> i32;

    /// `onGround`.
    fn on_ground(&self) -> bool;

    /// `isInWater`.
    fn is_in_water(&self) -> bool;

    /// `canStandOnFluid`.
    fn can_stand_on_fluid(
        &self,
        fluid: &crate::world::level::block::state::fluid_state::FluidState,
    ) -> bool;

    /// `maxUpStep`.
    fn max_up_step(&self) -> f32;

    /// `getMaxFallDistance`.
    fn get_max_fall_distance(&self) -> f32;

    /// `getBoundingBox`.
    fn bounding_box(&self) -> Aabb;

    /// `blockPosition`.
    fn block_position(&self) -> BlockPos;

    /// `level` — `Option<Arc<dyn LevelReader>>` (в Java не null).
    fn level(&self) -> Option<Arc<dyn LevelReader>>;

    /// `getPathfindingMalus`.
    fn get_pathfinding_malus(&self, path_type: PathType) -> f32;

    /// `setPathfindingMalus`.
    fn set_pathfinding_malus(&self, path_type: PathType, malus: f32);

    /// `getRandom().nextIntBetweenClosed(min, max)`-эквивалент.
    fn next_int_between(&self, min: i32, max: i32) -> i32;

    /// `hasLineOfSight(entity)`.
    fn has_line_of_sight(&self, entity: &Entity) -> bool;

    /// `getSpeed`.
    fn get_speed(&self) -> f32;

    /// `position`.
    fn position(&self) -> Vec3;
}

/// `Mob` — сущность с AI.
pub struct Mob {
    /// `LivingEntity`-часть (в Java — superclass).
    pub living: LivingEntity,
    /// `random` — `Entity.getRandom()` (Xoroshiro по умолчанию).
    pub random: std::rc::Rc<RefCell<Box<dyn RandomSource>>>,
    /// `goalSelector`.
    pub goal_selector: GoalSelector,
    /// `targetSelector`.
    pub target_selector: GoalSelector,
    /// `navigation` — `Rc<RefCell>` т.к. navigation вызывается
    /// из `Mob`-методов при заимствованном `self` (как в Java —
    /// navigation хранит ссылку на mob, у нас mob передаётся
    /// параметром).
    pub navigation: Option<Rc<RefCell<dyn crate::world::entity::ai::navigation::MobNavigation>>>,
    /// `lookControl`.
    pub look_control: LookControl,
    /// `moveControl`.
    pub move_control: MoveControl,
    /// `jumpControl`.
    pub jump_control: JumpControl,
    /// `bodyRotationControl`.
    pub body_rotation_control: BodyRotationControl,
    /// `flyingMoveControl` (для летающих мобов — создаётся
    /// переопределением `createMoveControl`).
    pub flying_move_control: Option<FlyingMoveControl>,
    /// `sensing`.
    pub sensing: Sensing,
    /// `pathfindingMalus` — `EnumMap<PathType, Float>`.
    pub pathfinding_malus: RefCell<HashMap<PathType, f32>>,
    /// `ambientSoundTime`.
    pub ambient_sound_time: i32,
    /// `xpReward`.
    pub xp_reward: i32,
    /// `canPickUpLoot`.
    pub can_pick_up_loot: bool,
    /// `target` — текущая цель (`LivingEntity` ссылкой —
    /// в Java это поле-ссылка; здесь `Option<LivingEntity>`
    /// клон, т.к. сущности у нас по значению).
    pub target: Option<LivingEntity>,
    /// `dropChances`.
    pub drop_chances: crate::world::entity::drop_chances::DropChances,
    /// `lootTable`/`lootTableSeed`.
    pub loot_table: Option<crate::resources::identifier::Identifier>,
    pub loot_table_seed: i64,
    /// `leashData`.
    pub leash_data: Option<LeashData>,
    /// `homePosition`/`homeRadius`.
    pub home_position: BlockPos,
    pub home_radius: i32,
    /// `noActionTime`.
    pub no_action_time: i32,
    /// `ambientSoundInterval`-поля (для наследников).
    pub spawn_anim: bool,
    /// `chargingCrossbow` — флаг анимации перезарядки арбалета
    /// (`CrossbowAttackMob.setChargingCrossbow`).
    pub charging_crossbow: bool,
    // `wasExploding`? нет — `persistent` флаги в Entity.
    // `ambientSoundTime` уже есть; `idleAnimationState`?
    // `swinging` есть в LivingEntity.
}

impl Mob {
    /// `Mob(type, level)` — конструктор: создаёт selectors,
    /// controls, navigation, sensing. `registerGoals` вызывается
    /// наследниками.
    pub fn new(living: LivingEntity) -> Self {
        let level = living.entity.level.clone();
        let navigation = Self::create_navigation_for(&living, level);
        let mut mob = Self {
            living,
            random: std::rc::Rc::new(RefCell::new(Box::new(SingleThreadedRandomSource::new(0)))),
            goal_selector: GoalSelector::new(),
            target_selector: GoalSelector::new(),
            navigation,
            look_control: LookControl::default(),
            move_control: MoveControl::default(),
            jump_control: JumpControl::default(),
            body_rotation_control: BodyRotationControl::default(),
            flying_move_control: None,
            sensing: Sensing::new(),
            pathfinding_malus: RefCell::new(HashMap::new()),
            ambient_sound_time: 0,
            xp_reward: 0,
            can_pick_up_loot: false,
            target: None,
            drop_chances: crate::world::entity::drop_chances::DropChances::default(),
            loot_table: None,
            loot_table_seed: 0,
            leash_data: None,
            home_position: BlockPos::ZERO,
            home_radius: -1,
            no_action_time: 0,
            spawn_anim: false,
            charging_crossbow: false,
        };
        mob.living.entity.is_mob = true;
        mob
    }

    /// `setChargingCrossbow`/`isChargingCrossbow`
    /// (`CrossbowAttackMob`).
    pub fn set_charging_crossbow(&mut self, charging: bool) {
        self.charging_crossbow = charging;
    }

    /// `isChargingCrossbow`.
    pub fn is_charging_crossbow(&self) -> bool {
        self.charging_crossbow
    }

    /// `createNavigation` — дефолт `GroundPathNavigation`
    /// (переопределяется наследниками: Ghast → Flying и т.д.).
    /// `maxVisitedNodes` = `floor(getMaxPathLength()*16)`,
    /// `getMaxPathLength` = `max(followRange, 16)`.
    pub fn create_navigation(
        living: &LivingEntity,
        level: Option<Arc<dyn LevelReader>>,
    ) -> Option<Rc<RefCell<dyn crate::world::entity::ai::navigation::MobNavigation>>> {
        level.map(|_l| {
            let follow_range = living
                .get_attribute_value_by_id_ro(
                    &crate::entity::ai::attributes::attributes::follow_range(),
                )
                .max(16.0);
            let max_visited = crate::util::mth::floor(follow_range * 16.0) as i32;
            Rc::new(RefCell::new(
                crate::world::entity::ai::navigation::path_navigation::PathNavigation::new(
                    crate::world::entity::ai::navigation::path_navigation::NavKind::Ground,
                    max_visited,
                ),
            )) as Rc<RefCell<dyn crate::world::entity::ai::navigation::MobNavigation>>
        })
    }

    fn create_navigation_for(
        living: &LivingEntity,
        level: Option<Arc<dyn LevelReader>>,
    ) -> Option<Rc<RefCell<dyn crate::world::entity::ai::navigation::MobNavigation>>> {
        Self::create_navigation(living, level)
    }

    /// `getNavigation` — если управляемая сущность — моб, берём
    /// её navigation (Java `getControlledVehicle() instanceof Mob`).
    /// У нас passengers — заглушки; возвращаем свою.
    pub fn get_navigation(&self) -> Option<Rc<RefCell<dyn crate::world::entity::ai::navigation::MobNavigation>>> {
        self.navigation.clone()
    }

    /// `navigationIsDone`.
    pub fn navigation_is_done(&self) -> bool {
        self.navigation
            .as_ref()
            .map(|n| n.borrow().is_done())
            .unwrap_or(true)
    }

    /// `navigationEvaluator` — Rc-хэндл evaluator'а для
    /// `getPathTypeAt` (см. `MoveControl::isWalkable`).
    pub fn navigation_evaluator_mut(
        &self,
    ) -> Option<Rc<RefCell<dyn crate::world::level::pathfinder::node_evaluator::NodeEvaluator>>> {
        self.navigation.as_ref().map(|n| n.borrow().evaluator_handle())
    }

    /// `isSunBurnTick`/`isOnFire`-делегаты при необходимости.

    /// `getMaxHeadXRot` = 40, `getMaxHeadYRot` = 75,
    /// `getHeadRotSpeed` = 10 (Java — int, у нас f32 для
    /// `rotateTowards`).
    pub fn get_max_head_x_rot(&self) -> f32 {
        40.0
    }
    pub fn get_max_head_y_rot(&self) -> f32 {
        75.0
    }
    pub fn get_head_rot_speed(&self) -> f32 {
        10.0
    }

    /// `firstPassengerIsMob`.
    pub fn first_passenger_is_mob(&self) -> bool {
        // `getPassengers().stream().findFirst() instanceof Mob` —
        // passengers пока не хранятся как сущности; возвращаем false
        // до порта vehicle-менеджмента.
        false
    }

    /// `getMainArm`/`isLeftHanded` — делегаты LivingEntity.

    /// `aiStep`-серверная часть (`Mob.aiStep` → `serverAiStep`):
    /// порядок тиков как в Java `Mob.aiStep` (строки ~690-720):
    /// targetSelector.tick → goalSelector.tick → sensing.tick →
    /// navigation.tick → customServerAiStep → controls.tick.
    pub fn server_ai_step(&mut self) {
        // mem::take-паттерн: selectors занимают mob при tick.
        let mut ts = std::mem::take(&mut self.target_selector);
        ts.tick(self);
        self.target_selector = ts;
        let mut gs = std::mem::take(&mut self.goal_selector);
        gs.tick(self);
        self.goal_selector = gs;
        self.sensing.tick();
        if let Some(nav) = self.navigation.clone() {
            // navigation.tick читает mob — передаём &*self.
            // Заимствование nav — Rc, mob не занят.
            let level_for_tick = self.living.entity.level.clone();
            let _ = level_for_tick;
            nav.borrow_mut().tick(self);
        }
        self.custom_server_ai_step();
        // controls: mem::take паттерн для разделения заимствований.
        let mut mc = std::mem::take(&mut self.move_control);
        mc.tick(self);
        self.move_control = mc;
        let mut lc = std::mem::take(&mut self.look_control);
        lc.tick(self);
        self.look_control = lc;
        let mut jc = std::mem::take(&mut self.jump_control);
        jc.tick(self);
        self.jump_control = jc;
        if let Some(fmc) = self.flying_move_control.take() {
            let mut mc = std::mem::take(&mut self.move_control);
            fmc.tick(&mut mc, self);
            self.move_control = mc;
            self.flying_move_control = Some(fmc);
        }
    }

    /// `customServerAiStep` — переопределяется наследниками.
    pub fn custom_server_ai_step(&mut self) {}

    /// `registerGoals` — переопределяется наследниками.
    pub fn register_goals(&mut self) {}

    /// `getPathfindingMalus`-helper для модификации (напр.
    /// AmphibiousNodeEvaluator.prepare/done).
    pub fn pathfinding_malus_map(&self) -> &RefCell<HashMap<PathType, f32>> {
        &self.pathfinding_malus
    }

    /// `isPersistenceRequired` — entity.persistence_required.
    pub fn set_persistence_required(&mut self, v: bool) {
        self.living.entity.persistence_required = v;
    }
    pub fn is_persistence_required(&self) -> bool {
        self.living.entity.persistence_required
    }

    /// `getLeashData`/`setLeashData` (упрощённо — Leashable).
    pub fn leash_data(&self) -> Option<&LeashData> {
        self.leash_data.as_ref()
    }
    pub fn set_leash_data(&mut self, data: Option<LeashData>) {
        self.leash_data = data;
    }

    /// `getHomePosition`/`setHomePosition`/`restrictTo`.
    pub fn restrict_to(&mut self, pos: BlockPos, radius: i32) {
        self.home_position = pos;
        self.home_radius = radius;
    }
    pub fn get_home_position(&self) -> BlockPos {
        self.home_position
    }
    pub fn get_home_restrict_radius(&self) -> i32 {
        self.home_radius
    }
    pub fn has_home_restriction(&self) -> bool {
        self.home_radius >= 0
    }

    /// `isWithinHomeDistance`/`isWithinRestriction`.
    pub fn is_within_restriction(&self, pos: BlockPos) -> bool {
        if !self.has_home_restriction() {
            return true;
        }
        let dx = pos.x - self.home_position.x;
        let dz = pos.z - self.home_position.z;
        dx * dx + dz * dz <= self.home_radius * self.home_radius
    }

    /// `getAmbientSoundInterval` — базовая реализация.
    pub fn get_ambient_sound_interval(&self) -> i32 {
        120
    }

    /// `isNoAi` — DATA_MOB_FLAGS NO_AI (bit 0).
    pub fn is_no_ai(&self) -> bool {
        self.living.entity.shared_flags & 1 != 0
    }
    pub fn set_no_ai(&mut self, no_ai: bool) {
        if no_ai {
            self.living.entity.shared_flags |= 1;
        } else {
            self.living.entity.shared_flags &= !1;
        }
    }

    /// `isLeftHanded`/`setLeftHanded` (bit 1).
    pub fn is_left_handed(&self) -> bool {
        self.living.entity.shared_flags & 2 != 0
    }
    pub fn set_left_handed(&mut self, left: bool) {
        if left {
            self.living.entity.shared_flags |= 2;
        } else {
            self.living.entity.shared_flags &= !2;
        }
    }

    /// `isAggressive`/`setAggressive` (bit 2).
    pub fn is_aggressive(&self) -> bool {
        self.living.entity.shared_flags & 4 != 0
    }
    pub fn set_aggressive(&mut self, aggressive: bool) {
        if aggressive {
            self.living.entity.shared_flags |= 4;
        } else {
            self.living.entity.shared_flags &= !4;
        }
    }

    /// `getTarget`/`setTarget`.
    pub fn get_target(&self) -> Option<&LivingEntity> {
        self.target.as_ref()
    }
    pub fn set_target(&mut self, target: Option<LivingEntity>) {
        self.target = target;
    }

    /// `canPickUpLoot`/`setCanPickUpLoot`.
    pub fn can_pick_up_loot(&self) -> bool {
        self.can_pick_up_loot
    }
    pub fn set_can_pick_up_loot(&mut self, v: bool) {
        self.can_pick_up_loot = v;
    }

    /// `getXpReward`/`setXpReward`.
    pub fn get_xp_reward(&self) -> i32 {
        self.xp_reward
    }
    pub fn set_xp_reward(&mut self, v: i32) {
        self.xp_reward = v;
    }

    /// `lookAt` — поворот к сущности (как в Java Mob.lookAt).
    pub fn look_at(&mut self, entity: &Entity, max_y_rot_increase: f32, max_x_rot_increase: f32) {
        let d = entity.get_x() - self.living.entity.get_x();
        let d1 = entity.get_z() - self.living.entity.get_z();
        let d2 = if entity.is_living_entity() {
            entity.get_eye_y() - self.living.entity.get_eye_y()
        } else {
            (entity.bounding_box.min_y + entity.bounding_box.max_y) / 2.0
                - self.living.entity.get_eye_y()
        };
        let square_root = (d * d + d1 * d1).sqrt();
        let f = (crate::util::mth::atan2(d1, d) * 180.0 / std::f64::consts::PI) as f32 - 90.0;
        let f1 = -((crate::util::mth::atan2(d2, square_root) * 180.0 / std::f64::consts::PI) as f32);
        let new_x = self.rotlerp(self.living.entity.get_x_rot(), f1, max_x_rot_increase);
        let new_y = self.rotlerp(self.living.entity.get_y_rot(), f, max_y_rot_increase);
        self.living.entity.set_x_rot(new_x);
        self.living.entity.set_y_rot(new_y);
    }

    /// `rotlerp`.
    fn rotlerp(&self, angle: f32, target_angle: f32, max_increase: f32) -> f32 {
        let f = crate::util::mth::wrap_degrees_f32(target_angle - angle);
        let f = f.clamp(-max_increase, max_increase);
        angle + f
    }

    /// `checkMobSpawnRules`/`checkMobSpawnObstruction` — статические
    /// проверки спавна (используются наследниками).
    /// `hasHome` — задана ли home-точка (`homeRadius >= 0`).
    pub fn has_home(&self) -> bool {
        self.home_radius >= 0
    }

    /// `isWithinHome(pos)` — BlockPos-вариант.
    pub fn is_within_home(&self, pos: BlockPos) -> bool {
        self.home_position.dist_manhattan(pos) <= self.home_radius
    }

    /// `isWithinHome(pos)` — Vec3-вариант (через blockPosition).
    pub fn is_within_home_vec(&self, pos: Vec3) -> bool {
        self.is_within_home(BlockPos::containing(pos.x, pos.y, pos.z))
    }

    /// `hasControllingPassenger` — базово false (пассажиры —
    /// заглушки; Boat/Vehicle переопределят).
    pub fn has_controlling_passenger(&self) -> bool {
        self.living.entity.get_controlling_passenger().is_some()
    }

    /// `getNoActionTime`.
    pub fn get_no_action_time(&self) -> i32 {
        self.no_action_time
    }

    /// `getWalkTargetValue(pos)` — malus path type в точке
    /// (PathfinderMob-метод; у нас на Mob напрямую).
    pub fn get_walk_target_value(&self, pos: BlockPos) -> f64 {
        let path_type = crate::world::entity::ai::util::goal_utils::walk_path_type_static(self, pos);
        self.get_pathfinding_malus(path_type) as f64
    }

    /// `shouldPanic` — база `Mob`: паника при горении/уроне
    /// (наследники переопределяют).
    pub fn should_panic(&self) -> bool {
        self.living.entity.is_on_fire() || self.living.entity.hurt_time() > 0
    }

    /// `getNearestPlayer`-обёртка для LookAtPlayerGoal.
    pub fn find_nearest_player(&self, range: f64) -> Option<crate::world::entity::entity::Entity> {
        let level = self.level()?;
        level.nearest_player(self.living.entity.position(), range)
    }

    /// Делегаты к `living.entity` для pathfinding/AI
    /// (Java-методы Entity, вызываемые из PathNavigation).
    pub fn get_x(&self) -> f64 { self.living.entity.get_x() }
    pub fn get_y(&self) -> f64 { self.living.entity.get_y() }
    pub fn get_z(&self) -> f64 { self.living.entity.get_z() }
    pub fn position(&self) -> Vec3 { self.living.entity.position() }
    pub fn block_position(&self) -> BlockPos { self.living.entity.block_position() }
    pub fn level(&self) -> Option<Arc<dyn LevelReader>> { self.living.entity.level.clone() }
    pub fn get_speed(&self) -> f32 { self.living.get_speed() }
    pub fn get_bb_width(&self) -> f32 { self.living.get_bb_width() }

    pub fn check_spawn_obstruction(
        level: &dyn LevelReader,
        entity_type: &crate::world::entity::entity_type::EntityType,
        pos: BlockPos,
    ) -> bool {
        let _ = (level, entity_type, pos);
        // TODO-полный порт: проверка коллизий spawn bounding box.
        // Пока — консервативно true (спавн не блокируется).
        true
    }

    /// `isFlapping`/`isInSittingPose` — переопределяются.
    pub fn is_flapping(&self) -> bool {
        false
    }
}

impl MobPathing for Mob {
    fn bb_width(&self) -> f32 {
        self.living.get_bb_width()
    }
    fn bb_height(&self) -> f32 {
        self.living.get_bb_height()
    }
    fn get_x(&self) -> f64 {
        self.living.entity.get_x()
    }
    fn get_y(&self) -> f64 {
        self.living.entity.get_y()
    }
    fn get_z(&self) -> f64 {
        self.living.entity.get_z()
    }
    fn get_block_y(&self) -> i32 {
        self.living.get_block_y()
    }
    fn on_ground(&self) -> bool {
        self.living.entity.on_ground
    }
    fn is_in_water(&self) -> bool {
        self.living.is_in_water()
    }
    fn can_stand_on_fluid(
        &self,
        fluid: &crate::world::level::block::state::fluid_state::FluidState,
    ) -> bool {
        self.living.can_stand_on_fluid(fluid)
    }
    fn max_up_step(&self) -> f32 {
        self.living.max_up_step()
    }
    fn get_max_fall_distance(&self) -> f32 {
        self.living.get_max_fall_distance()
    }
    fn bounding_box(&self) -> Aabb {
        self.living.entity.bounding_box.clone()
    }
    fn block_position(&self) -> BlockPos {
        self.living.block_position()
    }
    fn level(&self) -> Option<Arc<dyn LevelReader>> {
        self.living.entity.level.clone()
    }
    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        // `pathfindingMalus.getOrDefault(type, type.getMalus())`.
        self.pathfinding_malus
            .borrow()
            .get(&path_type)
            .copied()
            .unwrap_or_else(|| path_type.get_malus())
    }
    fn set_pathfinding_malus(&self, path_type: PathType, malus: f32) {
        self.pathfinding_malus.borrow_mut().insert(path_type, malus);
    }
    fn next_int_between(&self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        self.random.borrow_mut().next_int_range(min, max + 1)
    }
    fn has_line_of_sight(&self, entity: &Entity) -> bool {
        self.living.has_line_of_sight(entity)
    }
    fn get_speed(&self) -> f32 {
        self.living.get_speed()
    }
    fn position(&self) -> Vec3 {
        self.living.position()
    }
}

impl Mob {
    /// `canAttack(target)` — делегирует `LivingEntity.canAttack`.
    pub fn can_attack(&self, target: &LivingEntity) -> bool {
        self.living.can_attack(target)
    }

    /// `getSensing().hasLineOfSight(entity)` — Sensing требует
    /// `&mut self`, а `has_line_of_sight` берёт `&dyn MobPathing`:
    /// делаем `mem::take`/`put-back` чтобы не конфликтовать с
    /// заимствованием `mob`.
    pub fn has_line_of_sight(&mut self, target: &crate::world::entity::entity::Entity) -> bool {
        let mut sensing = std::mem::take(&mut self.sensing);
        let result = sensing.has_line_of_sight(self, target);
        self.sensing = sensing;
        result
    }

    /// `TamableAnimal.getOwner()`-эквивалент — UUID владельца
    /// для приручаемых мобов (TamableAnimal ещё не портирован;
    /// наследники переопределят). `None` = не приручён.
    pub fn tamable_owner_uuid(&self) -> Option<crate::util::uuid::Uuid> {
        None
    }

    /// `TamableAnimal.isTame()`.
    pub fn tamable_is_tame(&self) -> bool {
        false
    }

    /// `TamableAnimal.isOrderedToSit()`.
    pub fn tamable_is_ordered_to_sit(&self) -> bool {
        false
    }

    /// `TamableAnimal.getOwner()` — LivingEntity-владелец
    /// (резолвится наследником из owner-ссылки).
    pub fn tamable_owner(&self) -> Option<LivingEntity> {
        None
    }

    /// `TamableAnimal.wantsToAttack(attacker, owner)`.
    pub fn wants_to_attack(
        &self,
        _attacker: Option<&LivingEntity>,
        _owner: &LivingEntity,
    ) -> bool {
        false
    }

    /// `Raider.hasActiveRaid()`.
    pub fn has_active_raid(&self) -> bool {
        false
    }

    /// `NeutralMob.forgetCurrentTargetAndRefreshUniversalAnger()`.
    pub fn forget_current_target_and_refresh_universal_anger(&mut self) {
        self.target = None;
    }

    /// `Mob.isWithinMeleeAttackRange(target)` — vanilla:
    /// `ATTACK_RANGE` активного предмета → effective max/min;
    /// иначе `DEFAULT_ATTACK_REACH = sqrt(2.04) - 0.6`.
    /// Проверка: расширенный bbox пересекает hitbox цели,
    /// и (minRange <= 0 || bbox(minRange) НЕ пересекает).
    pub fn is_within_melee_attack_range(&self, entity: &LivingEntity) -> bool {
        use crate::resources::identifier::Identifier;
        use crate::world::item::component::attack_range::AttackRange;
        let (d, d1) = {
            let active = self.living.get_active_item();
            match active.components.get_by_id::<AttackRange>(
                &Identifier::minecraft("attack_range"),
            ) {
                Some(range) => (
                    range.effective_max_range(&self.living.entity) as f64,
                    range.effective_min_range(&self.living.entity) as f64,
                ),
                None => {
                    const DEFAULT_ATTACK_REACH: f64 = 0.827922_1; // sqrt(2.04)-0.6
                    (DEFAULT_ATTACK_REACH, 0.0)
                }
            }
        };
        let hitbox = entity.entity.bounding_box;
        self.get_attack_bounding_box(d).intersects(&hitbox)
            && (d1 <= 0.0 || !self.get_attack_bounding_box(d1).intersects(&hitbox))
    }

    /// `Mob.getAttackBoundingBox(horizontalExpansion)` —
    /// объединённый bbox с транспортом (vehicle), расширенный
    /// по X/Z.
    pub fn get_attack_bounding_box(&self, horizontal_expansion: f64) -> crate::world::phys::aabb::Aabb {
        let own = self.living.entity.bounding_box;
        let base = if let Some(vehicle) = self.living.get_vehicle() {
            let vb = vehicle.bounding_box;
            crate::world::phys::aabb::Aabb::new(
                own.min_x.min(vb.min_x),
                own.min_y,
                own.min_z.min(vb.min_z),
                own.max_x.max(vb.max_x),
                own.max_y,
                own.max_z.max(vb.max_z),
            )
        } else {
            own
        };
        base.inflate(horizontal_expansion, 0.0, horizontal_expansion)
    }

    /// `Mob.doHurtTarget(target)` — полная vanilla-цепочка:
    /// ATTACK_DAMAGE → weapon damage source → enchant modify →
    /// item attack bonus → hurtServer → knockback → hurtEnemy →
    /// post-attack enchant → setLastHurtMob → attack sound →
    /// lungeForwardMaybe.
    pub fn do_hurt_target(&mut self, target: &mut LivingEntity) -> bool {
        let mut f = self
            .living
            .get_attribute_value_by_id_ro(
                &crate::entity::ai::attributes::attributes::attack_damage(),
            ) as f32;
        let weapon_item = self.living.get_weapon_item();
        // `weaponItem.getDamageSource(this, () -> mobAttack(this))`.
        let fallback_sources = crate::world::damagesource::damage_sources::DamageSources::vanilla();
        let mob_entity = self.living.entity.clone();
        let damage_source = weapon_item.get_damage_source(&self.living, || {
            fallback_sources.mob_attack(&mob_entity)
        });
        // `EnchantmentHelper.modifyDamage(level, weapon, target,
        // source, f)` — модификация зачарованиями оружия.
        f = crate::world::item::enchantment::enchantment_helper::modify_damage(
            &weapon_item,
            target,
            &damage_source,
            f,
        );
        f += weapon_item.get_attack_damage_bonus(
            &target.entity,
            f,
            &damage_source,
        );
        let delta_movement = target.entity.get_delta_movement();
        let flag = target.hurt_server(damage_source.clone(), f);
        if flag {
            let knockback = self.living.get_knockback(target, &damage_source);
            self.living.cause_extra_knockback(target, knockback, delta_movement);
            weapon_item.hurt_enemy(target, &self.living);
            crate::world::item::enchantment::enchantment_helper::do_post_attack_effects(
                target,
                &damage_source,
            );
            self.living.set_last_hurt_mob(Some(target.entity.clone()));
            self.living.play_attack_sound();
        }
        self.living.lunge_forward_maybe();
        flag
    }
}
