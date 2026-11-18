//! Перенос `net.minecraft.world.entity.ai.goal.target` —
//! TargetGoal (базовое состояние), HurtByTargetGoal,
//! NearestAttackableTargetGoal.

use crate::core::holder::Holder;
use crate::resources::identifier::Identifier;
use crate::util::uuid::Uuid;
use crate::world::entity::ai::goal::goal::{Goal, GoalFlag, GoalFlagSet};
use crate::world::entity::ai::navigation::mob_navigation::MobNavigation;
use crate::world::entity::ai::targeting::targeting_conditions::{Selector, TargetingConditions};
use crate::world::entity::entity_type::EntityType;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::entity::mob_category::MobCategory;
use crate::world::phys::aabb::Aabb;

/// Разновидность `Class<?>` из Java-конструкторов
/// (`toIgnoreDamage`/`toIgnoreAlert`): точный entity-type id
/// или категория (`Mob.class`/`Animal.class`-уровень через
/// `MobCategory`, т.к. наследования классов в Rust нет).
#[derive(Clone, Debug)]
pub enum IgnoredClass {
    /// Конкретный `EntityType` (id).
    Type(Identifier),
    /// Целая `MobCategory` (аналог `Mob.class.isAssignableFrom`).
    Category(MobCategory),
}

impl IgnoredClass {
    /// `clazz.isAssignableFrom(entity.getClass())`.
    pub fn matches(&self, entity: &LivingEntity) -> bool {
        match self {
            IgnoredClass::Type(id) => holder_type_id(&entity.entity.entity_type)
                .map(|eid| &eid == id)
                .unwrap_or(false),
            IgnoredClass::Category(cat) => {
                holder_type_category(&entity.entity.entity_type) == Some(*cat)
            }
        }
    }
}

use crate::world::entity::entity_type::{holder_type_category, holder_type_id};

/// `TargetGoal` — разделяемое состояние базовой target-цели.
/// В Java — abstract class; здесь — компонент, который
/// конкретные цели держат полем и дёргают через helpers.
#[derive(Default)]
pub struct TargetGoalState {
    /// `mustSee`.
    pub must_see: bool,
    /// `mustReach`.
    pub must_reach: bool,
    /// `reachCache` (0=EMPTY,1=CAN_REACH,2=CANT_REACH).
    pub reach_cache: i32,
    /// `reachCacheTime`.
    pub reach_cache_time: i32,
    /// `unseenTicks`.
    pub unseen_ticks: i32,
    /// `targetMob`.
    pub target_mob: Option<LivingEntity>,
    /// `unseenMemoryTicks` (default 60).
    pub unseen_memory_ticks: i32,
}

impl TargetGoalState {
    /// `TargetGoal(mob, mustSee)` / `(mob, mustSee, mustReach)`.
    pub fn new(must_see: bool, must_reach: bool) -> Self {
        Self {
            must_see,
            must_reach,
            reach_cache: 0,
            reach_cache_time: 0,
            unseen_ticks: 0,
            target_mob: None,
            unseen_memory_ticks: 60,
        }
    }

    /// `setUnseenMemoryTicks`.
    pub fn set_unseen_memory_ticks(&mut self, ticks: i32) {
        self.unseen_memory_ticks = ticks;
    }

    /// `getFollowDistance()` — `FOLLOW_RANGE` атрибут.
    pub fn get_follow_distance(mob: &Mob) -> f64 {
        mob.living
            .get_attribute_value_by_id_ro(&crate::entity::ai::attributes::attributes::follow_range())
    }

    /// `canContinueToUse()`.
    pub fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let target = mob
            .get_target()
            .cloned()
            .or_else(|| self.target_mob.clone());
        let Some(target) = target else { return false };
        if !mob.living.can_attack(&target) {
            return false;
        }
        // team != null && team1 == team → false
        if let (Some(t1), Some(t2)) = (mob.living.get_team(), target.get_team()) {
            if t1.name == t2.name {
                return false;
            }
        }
        let follow_distance = Self::get_follow_distance(mob);
        if mob.living.entity.distance_to_sqr(&target.entity)
            > follow_distance * follow_distance
        {
            return false;
        }
        if self.must_see {
            if mob.has_line_of_sight(&target.entity) {
                self.unseen_ticks = 0;
            } else {
                self.unseen_ticks += 1;
                if self.unseen_ticks > self.unseen_memory_ticks {
                    return false;
                }
            }
        }
        mob.set_target(Some(target));
        true
    }

    /// `start()`.
    pub fn start(&mut self) {
        self.reach_cache = 0;
        self.reach_cache_time = 0;
        self.unseen_ticks = 0;
    }

    /// `stop()`.
    pub fn stop(&mut self, mob: &mut Mob) {
        mob.set_target(None);
        self.target_mob = None;
    }

    /// `canAttack(potentialTarget, targetPredicate)`.
    pub fn can_attack(
        &mut self,
        mob: &mut Mob,
        potential_target: Option<&LivingEntity>,
        target_predicate: &TargetingConditions,
    ) -> bool {
        let Some(potential_target) = potential_target else { return false };
        let level = mob.living.entity.level.clone();
        let Some(level) = level else { return false };
        // entity == target проверка по uuid.
        if potential_target.entity.uuid == mob.living.entity.uuid {
            return false;
        }
        if !target_predicate.test(&level, Some(&mut *mob), potential_target) {
            return false;
        }
        if !mob.is_within_home(potential_target.entity.block_position()) {
            return false;
        }
        if self.must_reach {
            self.reach_cache_time -= 1;
            if self.reach_cache_time <= 0 {
                self.reach_cache = 0;
            }
            if self.reach_cache == 0 {
                self.reach_cache = if self.can_reach(mob, potential_target) { 1 } else { 2 };
            }
            if self.reach_cache == 2 {
                return false;
            }
        }
        true
    }

    /// `canReach(target)` — путь до цели заканчивается в 1.5 блоках.
    fn can_reach(&mut self, mob: &mut Mob, target: &LivingEntity) -> bool {
        // reducedTickDelay(10 + rand(5)) — tickDelay=0 → identity.
        self.reach_cache_time = 10 + mob.random.borrow_mut().next_int_bound(5);
        let path = mob.navigation.as_ref().and_then(|nav| {
            nav.borrow_mut().create_path_entity(mob, target, 0)
        });
        let Some(path) = path else { return false };
        let Some(end) = path.end_node() else { return false };
        let (ex, ez) = {
            let n = end.borrow();
            (n.x, n.z)
        };
        let dx = ex - target.entity.block_position().x;
        let dz = ez - target.entity.block_position().z;
        (dx * dx + dz * dz) as f64 <= 2.25
    }
}

/// `HurtByTargetGoal` — цель = тот, кто недавно ударил моба.
pub struct HurtByTargetGoal {
    /// `TargetGoal`-состояние.
    pub base: TargetGoalState,
    /// `timestamp`.
    pub timestamp: i32,
    /// `alertSameType`.
    pub alert_same_type: bool,
    /// `toIgnoreDamage` — `Class<?>[]` → `IgnoredClass`.
    pub to_ignore_damage: Vec<IgnoredClass>,
    /// `toIgnoreAlert`.
    pub to_ignore_alert: Vec<IgnoredClass>,
    /// Статический `HURT_BY_TARGETING`:
    /// `forCombat().ignoreLineOfSight().ignoreInvisibilityTesting()`.
    pub hurt_by_targeting: TargetingConditions,
}

impl HurtByTargetGoal {
    /// `new(mob, toIgnoreDamage...)`.
    pub fn new(to_ignore_damage: Vec<IgnoredClass>) -> Self {
        let mut cond = TargetingConditions::for_combat();
        cond.ignore_line_of_sight();
        cond.ignore_invisibility_testing();
        Self {
            base: TargetGoalState::new(true, false),
            timestamp: 0,
            alert_same_type: false,
            to_ignore_damage,
            to_ignore_alert: Vec::new(),
            hurt_by_targeting: cond,
        }
    }

    /// `setAlertOthers(reinforcementTypes)`.
    pub fn set_alert_others(mut self, types: Vec<IgnoredClass>) -> Self {
        self.alert_same_type = true;
        self.to_ignore_alert = types;
        self
    }

    /// `alertOther(mob, target)` — переопределяемая точка
    /// (в Java protected; здесь просто setTarget с reason).
    pub fn alert_other(mob: &mut Mob, target: &LivingEntity) {
        mob.set_target(Some(target.clone()));
    }

    /// `alertOthers()` — оповещение союзников в радиусе.
    fn alert_others(&self, mob: &mut Mob) {
        let follow_distance = TargetGoalState::get_follow_distance(mob);
        let aabb = Aabb::unit_cube_from_lower_corner(&mob.living.entity.position())
            .inflate(follow_distance, 10.0, follow_distance);
        let self_id = mob.living.entity.id;
        let self_owner = mob.tamable_owner_uuid();
        let attacker = mob
            .living
            .get_last_hurt_by_mob()
            .or_else(|| self.base.target_mob.clone());
        let Some(attacker) = attacker else { return };
        let ignore_alert = self.to_ignore_alert.clone();
        let level = mob.living.entity.level.clone();
        let Some(level) = level else { return };
        level.for_each_mob_in_box(&aabb, &mut |other: &mut Mob| {
            if other.living.entity.id == self_id || other.get_target().is_some() {
                return;
            }
            // TamableAnimal-ветка: если этот моб приручён,
            // владельцы должны совпадать.
            if let Some(o1) = self_owner {
                if other.tamable_owner_uuid() != Some(o1) {
                    return;
                }
            }
            if other.living.is_allied_to(&attacker.entity) {
                return;
            }
            if !ignore_alert.is_empty()
                && ignore_alert.iter().any(|c| c.matches(&other.living))
            {
                return;
            }
            if other.living.entity.is_spectator() {
                return;
            }
            Self::alert_other(other, &attacker);
        });
    }
}

impl Goal for HurtByTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let timestamp = mob.living.get_last_hurt_by_mob_timestamp();
        let last_hurt = mob.living.get_last_hurt_by_mob();
        if timestamp == self.timestamp || last_hurt.is_none() {
            return false;
        }
        let Some(last_hurt) = last_hurt else { return false };
        // UNIVERSAL_ANGER gamerule: игрок-цель не является целью.
        if holder_type_id(&last_hurt.entity.entity_type)
            .map(|id| id == Identifier::minecraft("player"))
            .unwrap_or(false)
            && mob
                .level()
                .map(|l| l.universal_anger())
                .unwrap_or(false)
        {
            return false;
        }
        if self
            .to_ignore_damage
            .iter()
            .any(|c| c.matches(&last_hurt))
        {
            return false;
        }
        self.base.can_attack(mob, Some(&last_hurt), &self.hurt_by_targeting)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.base.can_continue_to_use(mob)
    }

    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Target])
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(attacker) = mob.living.get_last_hurt_by_mob() {
            mob.set_target(Some(attacker.clone()));
            self.base.target_mob = mob.get_target().cloned();
            self.timestamp = mob.living.get_last_hurt_by_mob_timestamp();
            self.base.unseen_memory_ticks = 300;
            if self.alert_same_type {
                self.alert_others(mob);
            }
        }
        self.base.start();
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.base.stop(mob);
    }
}

/// `NearestAttackableTargetGoal<T>` — ближайшаяattackable-цель
/// заданного entity-type'а. `target_type` — id entity-type'а
/// (`minecraft:player` — special-case как в Java Player.class).
#[allow(clippy::too_many_arguments)]
pub struct NearestAttackableTargetGoal {
    /// `TargetGoal`-состояние.
    pub base: TargetGoalState,
    /// `targetType` — entity-type id.
    pub target_type: Identifier,
    /// `randomInterval`.
    pub random_interval: i32,
    /// `target`.
    pub target: Option<LivingEntity>,
    /// `targetConditions`.
    pub target_conditions: TargetingConditions,
}

impl NearestAttackableTargetGoal {
    /// `new(mob, targetType, mustSee)`.
    pub fn new(target_type: Identifier, must_see: bool) -> Self {
        Self::full(target_type, 10, must_see, false, None)
    }

    /// `new(mob, targetType, mustSee, selector)`.
    pub fn with_selector(
        target_type: Identifier,
        must_see: bool,
        selector: Selector,
    ) -> Self {
        Self::full(target_type, 10, must_see, false, Some(selector))
    }

    /// `new(mob, targetType, mustSee, mustReach)`.
    pub fn with_reach(target_type: Identifier, must_see: bool, must_reach: bool) -> Self {
        Self::full(target_type, 10, must_see, must_reach, None)
    }

    /// Полный конструктор `new(mob, targetType, interval,
    /// mustSee, mustReach, selector)`.
    pub fn full(
        target_type: Identifier,
        interval: i32,
        must_see: bool,
        must_reach: bool,
        selector: Option<Selector>,
    ) -> Self {
        let mut conditions = TargetingConditions::for_combat();
        conditions.range(0.0);
        if let Some(sel) = selector {
            conditions.selector(sel);
        }
        Self {
            base: TargetGoalState::new(must_see, must_reach),
            target_type,
            random_interval: interval,
            target: None,
            target_conditions: conditions,
        }
    }

    /// `setTarget(target)`.
    pub fn set_target(&mut self, target: Option<LivingEntity>) {
        self.target = target;
    }

    /// `is_player_target()` — `targetType == Player.class`.
    fn is_player_target(&self) -> bool {
        self.target_type == Identifier::minecraft("player")
    }

    /// `getTargetSearchArea(targetDistance)`.
    fn get_target_search_area(&self, mob: &Mob, target_distance: f64) -> Aabb {
        mob.living
            .entity
            .bounding_box
            .inflate(target_distance, target_distance, target_distance)
    }

    /// `findTarget()`.
    fn find_target(&mut self, mob: &mut Mob) {
        let level = mob.living.entity.level.clone();
        let Some(level) = level else {
            self.target = None;
            return;
        };
        let follow_distance = TargetGoalState::get_follow_distance(mob);
        self.target_conditions.range(follow_distance);
        let eye = mob.living.entity.get_eye_position();
        self.target = if self.is_player_target() {
            level
                .nearest_player(eye, follow_distance)
                .map(LivingEntity::from_entity)
        } else {
            level
                .nearest_entity_of_type(eye, follow_distance, &self.target_type)
                .map(LivingEntity::from_entity)
        };
        // entity == target → null (Java: getNearestEntity отсекает self
        // через targeting;保险起见 проверяем uuid).
        if let Some(t) = &self.target {
            if t.entity.uuid == mob.living.entity.uuid {
                self.target = None;
            }
        }
    }
    /// Публичный доступ к `findTarget` для наследников
    /// (`NearestHealableRaiderTargetGoal.canUse` вызывает его напрямую).
    pub fn find_target_pub(&mut self, mob: &mut Mob) {
        self.find_target(mob);
    }
}

impl Goal for NearestAttackableTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if self.random_interval > 0
            && mob.random.borrow_mut().next_int_bound(self.random_interval) != 0
        {
            return false;
        }
        self.find_target(mob);
        self.target.is_some()
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.base.can_continue_to_use(mob)
    }

    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Target])
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(t) = self.target.clone() {
            mob.set_target(Some(t));
        }
        self.base.start();
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.base.stop(mob);
    }
}

/// `OwnerHurtByTargetGoal` — атака того, кто ударил владельца.
pub struct OwnerHurtByTargetGoal {
    /// `TargetGoal`-состояние.
    pub base: TargetGoalState,
    /// `ownerLastHurtBy`.
    pub owner_last_hurt_by: Option<LivingEntity>,
    /// `timestamp`.
    pub timestamp: i32,
}

impl OwnerHurtByTargetGoal {
    /// `new(tameAnimal)`.
    pub fn new() -> Self {
        Self {
            base: TargetGoalState::new(false, false),
            owner_last_hurt_by: None,
            timestamp: 0,
        }
    }
}

impl Default for OwnerHurtByTargetGoal {
    fn default() -> Self { Self::new() }
}

impl Goal for OwnerHurtByTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if !mob.tamable_is_tame() || mob.tamable_is_ordered_to_sit() {
            return false;
        }
        let Some(owner) = mob.tamable_owner() else { return false };
        self.owner_last_hurt_by = owner.get_last_hurt_by_mob();
        let last_ts = owner.get_last_hurt_by_mob_timestamp();
        last_ts != self.timestamp
            && self.base.can_attack(
                mob,
                self.owner_last_hurt_by.as_ref(),
                &TargetingConditions::default_conditions(),
            )
            && mob.wants_to_attack(self.owner_last_hurt_by.as_ref(), &owner)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.base.can_continue_to_use(mob)
    }

    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Target])
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(t) = self.owner_last_hurt_by.clone() {
            mob.set_target(Some(t));
        }
        if let Some(owner) = mob.tamable_owner() {
            self.timestamp = owner.get_last_hurt_by_mob_timestamp();
        }
        self.base.start();
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.base.stop(mob);
    }
}

/// `OwnerHurtTargetGoal` — атака сущности, которая ударила владельца.
pub struct OwnerHurtTargetGoal {
    /// `TargetGoal`-состояние.
    pub base: TargetGoalState,
    /// `ownerLastHurt`.
    pub owner_last_hurt: Option<LivingEntity>,
    /// `timestamp`.
    pub timestamp: i32,
}

impl OwnerHurtTargetGoal {
    /// `new(tameAnimal)`.
    pub fn new() -> Self {
        Self {
            base: TargetGoalState::new(false, false),
            owner_last_hurt: None,
            timestamp: 0,
        }
    }
}

impl Default for OwnerHurtTargetGoal {
    fn default() -> Self { Self::new() }
}

impl Goal for OwnerHurtTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if !mob.tamable_is_tame() || mob.tamable_is_ordered_to_sit() {
            return false;
        }
        let Some(owner) = mob.tamable_owner() else { return false };
        self.owner_last_hurt = owner.get_last_hurt_mob().map(LivingEntity::from_entity);
        let last_ts = owner.get_last_hurt_mob_timestamp();
        last_ts != self.timestamp
            && self.base.can_attack(
                mob,
                self.owner_last_hurt.as_ref(),
                &TargetingConditions::default_conditions(),
            )
            && mob.wants_to_attack(self.owner_last_hurt.as_ref(), &owner)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.base.can_continue_to_use(mob)
    }

    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Target])
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(t) = self.owner_last_hurt.clone() {
            mob.set_target(Some(t));
        }
        if let Some(owner) = mob.tamable_owner() {
            self.timestamp = owner.get_last_hurt_mob_timestamp();
        }
        self.base.start();
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.base.stop(mob);
    }
}

/// `ResetUniversalAngerTargetGoal` — сброс цели при
/// UNIVERSAL_ANGER-геймеруле и ударе игрока (NeutralMob).
pub struct ResetUniversalAngerTargetGoal {
    /// `alertOthersOfSameType`.
    pub alert_others_of_same_type: bool,
    /// `lastHurtByPlayerTimestamp`.
    pub last_hurt_by_player_timestamp: i32,
}

impl ResetUniversalAngerTargetGoal {
    /// `new(mob, alertOthersOfSameType)`.
    pub fn new(alert_others_of_same_type: bool) -> Self {
        Self {
            alert_others_of_same_type,
            last_hurt_by_player_timestamp: 0,
        }
    }

    /// `wasHurtByPlayer()`.
    fn was_hurt_by_player(&self, mob: &Mob) -> bool {
        mob.living
            .get_last_hurt_by_mob()
            .map(|e| {
                holder_type_id(&e.entity.entity_type)
                    .map(|id| id == Identifier::minecraft("player"))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
            && mob.living.get_last_hurt_by_mob_timestamp() > self.last_hurt_by_player_timestamp
    }
}

impl Goal for ResetUniversalAngerTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        mob.level()
            .map(|l| l.universal_anger())
            .unwrap_or(false)
            && self.was_hurt_by_player(mob)
    }

    fn can_continue_to_use(&mut self, _mob: &mut Mob) -> bool {
        false
    }

    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::empty()
    }

    fn start(&mut self, mob: &mut Mob) {
        self.last_hurt_by_player_timestamp = mob.living.get_last_hurt_by_mob_timestamp();
        mob.forget_current_target_and_refresh_universal_anger();
        if self.alert_others_of_same_type {
            let follow = TargetGoalState::get_follow_distance(mob);
            let aabb = Aabb::unit_cube_from_lower_corner(&mob.living.entity.position())
                .inflate(follow, 10.0, follow);
            let level = mob.living.entity.level.clone();
            if let Some(level) = level {
                level.for_each_mob_in_box(&aabb, &mut |other: &mut Mob| {
                    other.forget_current_target_and_refresh_universal_anger();
                });
            }
        }
    }
}

/// `NonTameRandomTargetGoal` — NearestAttackableTargetGoal,
/// не активный пока моб приручён.
pub struct NonTameRandomTargetGoal {
    /// Внутренняя `NearestAttackableTargetGoal`.
    pub inner: NearestAttackableTargetGoal,
}

impl NonTameRandomTargetGoal {
    /// `new(tamableMob, targetType, mustSee, selector)`.
    pub fn new(
        target_type: Identifier,
        must_see: bool,
        selector: Option<Selector>,
    ) -> Self {
        Self {
            inner: NearestAttackableTargetGoal::full(target_type, 10, must_see, false, selector),
        }
    }
}

impl Goal for NonTameRandomTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        !mob.tamable_is_tame() && self.inner.can_use(mob)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        // Java: targetConditions.test(level, mob, target) —
        // conditions всегда non-null (forCombat в конструкторе).
        let level = mob.living.entity.level.clone();
        let (Some(level), Some(target)) = (level, self.inner.target.clone()) else {
            return self.inner.base.can_continue_to_use(mob);
        };
        if target.entity.uuid == mob.living.entity.uuid {
            return false;
        }
        self.inner.target_conditions.test(&level, Some(mob), &target)
    }

    fn get_flags(&self) -> GoalFlagSet {
        self.inner.get_flags()
    }

    fn start(&mut self, mob: &mut Mob) {
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.inner.stop(mob);
    }
}

/// `NearestHealableRaiderTargetGoal` — цель-рейд для лечения
/// (Raider.hasActiveRaid + cooldown 200).
pub struct NearestHealableRaiderTargetGoal {
    /// Внутренняя `NearestAttackableTargetGoal` (interval=500).
    pub inner: NearestAttackableTargetGoal,
    /// `cooldown`.
    pub cooldown: i32,
}

impl NearestHealableRaiderTargetGoal {
    /// `new(raider, targetType, mustSee, selector)`.
    pub fn new(
        target_type: Identifier,
        must_see: bool,
        selector: Option<Selector>,
    ) -> Self {
        Self {
            inner: NearestAttackableTargetGoal::full(target_type, 500, must_see, false, selector),
            cooldown: 0,
        }
    }

    /// `getCooldown()`.
    pub fn get_cooldown(&self) -> i32 {
        self.cooldown
    }

    /// `decrementCooldown()`.
    pub fn decrement_cooldown(&mut self) {
        self.cooldown -= 1;
    }
}

impl Goal for NearestHealableRaiderTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if self.cooldown > 0 || !mob.random.borrow_mut().next_boolean() {
            return false;
        }
        if !mob.has_active_raid() {
            return false;
        }
        // Java canUse: findTarget() напрямую (без randomInterval).
        self.inner.find_target_pub(mob);
        self.inner.target.is_some()
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.inner.base.can_continue_to_use(mob)
    }

    fn get_flags(&self) -> GoalFlagSet {
        self.inner.get_flags()
    }

    fn start(&mut self, mob: &mut Mob) {
        // reducedTickDelay(200) — tickDelay=0 → 200.
        self.cooldown = 200;
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.inner.stop(mob);
    }
}

/// `NearestAttackableWitchTargetGoal` — NearestAttackableTargetGoal
/// с внешним флагом canAttack.
pub struct NearestAttackableWitchTargetGoal {
    /// Внутренняя `NearestAttackableTargetGoal`.
    pub inner: NearestAttackableTargetGoal,
    /// `canAttack`.
    pub can_attack_flag: bool,
}

impl NearestAttackableWitchTargetGoal {
    /// `new(raider, targetType, interval, mustSee, mustReach, selector)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        target_type: Identifier,
        interval: i32,
        must_see: bool,
        must_reach: bool,
        selector: Option<Selector>,
    ) -> Self {
        Self {
            inner: NearestAttackableTargetGoal::full(target_type, interval, must_see, must_reach, selector),
            can_attack_flag: true,
        }
    }

    /// `setCanAttack(canAttack)`.
    pub fn set_can_attack(&mut self, can_attack: bool) {
        self.can_attack_flag = can_attack;
    }
}

impl Goal for NearestAttackableWitchTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        self.can_attack_flag && self.inner.can_use(mob)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.inner.base.can_continue_to_use(mob)
    }

    fn get_flags(&self) -> GoalFlagSet {
        self.inner.get_flags()
    }

    fn start(&mut self, mob: &mut Mob) {
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.inner.stop(mob);
    }
}

/// `DefendVillageTargetGoal` — голем защищает деревню:
/// цель = игрок с репутацией <= -100 у ближайшего Villager.
pub struct DefendVillageTargetGoal {
    /// `potentialTarget`.
    pub potential_target: Option<LivingEntity>,
    /// `attackTargeting` — `forCombat().range(64)`.
    pub attack_targeting: TargetingConditions,
    /// `base` — TargetGoal-состояние (mustSee=false, mustReach=true).
    pub base: TargetGoalState,
}

impl DefendVillageTargetGoal {
    /// `new(golem)` — mustSee=false, mustReach=true.
    pub fn new() -> Self {
        let mut cond = TargetingConditions::for_combat();
        cond.range(64.0);
        Self {
            potential_target: None,
            attack_targeting: cond,
            base: TargetGoalState::new(false, true),
        }
    }

    fn is_villager(e: &LivingEntity) -> bool {
        holder_type_id(&e.entity.entity_type)
            .map(|id| id == Identifier::minecraft("villager"))
            .unwrap_or(false)
    }

    fn is_player(e: &LivingEntity) -> bool {
        holder_type_id(&e.entity.entity_type)
            .map(|id| id == Identifier::minecraft("player"))
            .unwrap_or(false)
    }
}

impl Default for DefendVillageTargetGoal {
    fn default() -> Self { Self::new() }
}

impl Goal for DefendVillageTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let aabb = mob.living.entity.bounding_box.inflate(10.0, 8.0, 10.0);
        let level = mob.living.entity.level.clone();
        let Some(level) = level else {
            self.potential_target = None;
            return false;
        };
        let entities = level.entities_in_box(&aabb);
        let villagers: Vec<LivingEntity> = entities
            .iter()
            .cloned()
            .map(LivingEntity::from_entity)
            .filter(Self::is_villager)
            .collect();
        let players: Vec<LivingEntity> = entities
            .iter()
            .cloned()
            .map(LivingEntity::from_entity)
            .filter(Self::is_player)
            .collect();
        self.potential_target = None;
        for villager in &villagers {
            for player in &players {
                if villager.player_reputation(player) <= -100 {
                    self.potential_target = Some(player.clone());
                }
            }
        }
        match &self.potential_target {
            Some(t) => !(Self::is_player(t)
                && (t.entity.is_spectator() || t.entity.is_creative())),
            None => false,
        }
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        // targetMob = potentialTarget; canAttack не перепроверяется
        // в Java canContinue (только distance/team/LOS/reach).
        self.base.target_mob = self.potential_target.clone();
        self.base.can_continue_to_use(mob)
    }

    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Target])
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(t) = self.potential_target.clone() {
            mob.set_target(Some(t));
        }
        self.base.start();
    }

    fn stop(&mut self, mob: &mut Mob) {
        mob.set_target(None);
        self.potential_target = None;
        self.base.target_mob = None;
    }
}
