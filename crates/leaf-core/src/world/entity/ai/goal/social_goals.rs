//! Перенос социальных/домашних AI-целей
//! `net.minecraft.world.entity.ai.goal`:
//! FollowOwnerGoal, SitWhenOrderedToGoal, RandomStandGoal.

use crate::world::entity::ai::goal::goal::{Goal, GoalFlag, GoalFlagSet};
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::{Mob, MobPathing};
use crate::world::level::pathfinder::path_type::PathType;
use std::sync::Arc;

/// `FollowOwnerGoal` — следование за владельцем (приручённые мобы).
pub struct FollowOwnerGoal {
    speed_modifier: f64,
    owner: Option<LivingEntity>,
    time_to_recalc_path: i32,
    stop_distance: f32,
    start_distance: f32,
    old_water_cost: f32,
}

impl FollowOwnerGoal {
    /// `new(tamable, speedModifier, startDistance, stopDistance)`.
    /// Проверка типа навигации (Ground/Flying) — как в Java.
    pub fn new(
        _mob: &Mob,
        speed_modifier: f64,
        start_distance: f32,
        stop_distance: f32,
    ) -> Self {
        Self {
            speed_modifier,
            owner: None,
            time_to_recalc_path: 0,
            stop_distance,
            start_distance,
            old_water_cost: 0.0,
        }
    }
}

impl Goal for FollowOwnerGoal {
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Move, GoalFlag::Look])
    }

    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let Some(owner) = mob.tamable_owner() else {
            return false;
        };
        if mob.unable_to_move_to_owner() {
            return false;
        }
        if mob.living.entity.distance_to_sqr(&owner.entity)
            < (self.start_distance * self.start_distance) as f64
        {
            return false;
        }
        self.owner = Some(owner);
        true
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(owner) = &self.owner else {
            return false;
        };
        !mob.navigation_is_done()
            && !mob.unable_to_move_to_owner()
            && mob.living.entity.distance_to_sqr(&owner.entity)
                > (self.stop_distance * self.stop_distance) as f64
    }

    fn start(&mut self, mob: &mut Mob) {
        self.time_to_recalc_path = 0;
        self.old_water_cost = mob.get_pathfinding_malus(PathType::Water);
        mob.set_pathfinding_malus(PathType::Water, 0.0);
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.owner = None;
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().stop(mob);
        }
        mob.set_pathfinding_malus(PathType::Water, self.old_water_cost);
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(owner) = self.owner.clone() else { return };
        let should_try_teleport = mob.should_try_teleport_to_owner();
        if !should_try_teleport && mob.living.entity.distance_to_sqr(&owner.entity) <= 256.0 {
            // Paper-лимит дистанции обзора питомца (16 блоков).
            mob.look_control.set_look_at_entity_deltas(
                &owner.entity,
                10.0,
                mob.get_max_head_x_rot(),
            );
        }

        self.time_to_recalc_path -= 1;
        if self.time_to_recalc_path <= 0 {
            self.time_to_recalc_path = Self::reduced_tick_delay_static(10);
            if should_try_teleport {
                mob.try_to_teleport_to_owner();
            } else if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().move_to_entity(mob, &owner, self.speed_modifier);
            }
        }
    }
}

/// `SitWhenOrderedToGoal` — сидение по команде (TamableAnimal).
pub struct SitWhenOrderedToGoal;

impl Goal for SitWhenOrderedToGoal {
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Jump, GoalFlag::Move])
    }

    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let is_ordered_to_sit = mob.tamable_is_ordered_to_sit();
        if !is_ordered_to_sit && !mob.tamable_is_tame() {
            return false;
        }
        let e = &mob.living.entity;
        if e.is_in_water() || !e.on_ground() {
            return false;
        }
        // `owner == null || owner.level() != mob.level() || cond && isOrderedToSit`.
        let Some(owner) = mob.tamable_owner() else {
            return true;
        };
        let same_level = match (&owner.entity.level, &mob.living.entity.level) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if !same_level {
            return true;
        }
        let d = mob.living.entity.distance_to_sqr(&owner.entity);
        (d >= 144.0 || owner.get_last_hurt_by_mob().is_none()) && is_ordered_to_sit
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        mob.tamable_is_ordered_to_sit()
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().stop(mob);
        }
        mob.set_in_sitting_pose(true);
    }

    fn stop(&mut self, mob: &mut Mob) {
        mob.set_in_sitting_pose(false);
    }
}

/// `RandomStandGoal` — случайное вставание лошади.
pub struct RandomStandGoal {
    next_stand: i32,
}

impl RandomStandGoal {
    /// `new(horse)` — `resetStandInterval` в конструкторе.
    pub fn new(mob: &Mob) -> Self {
        Self {
            next_stand: -mob.horse_ambient_stand_interval(),
        }
    }
}

impl Goal for RandomStandGoal {
    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn can_use(&mut self, mob: &mut Mob) -> bool {
        self.next_stand += 1;
        if self.next_stand > 0
            && mob.random.borrow_mut().next_int_bound(1000) < self.next_stand
        {
            self.next_stand = -mob.horse_ambient_stand_interval();
            !mob.living.entity.is_immobile()
                && mob.random.borrow_mut().next_int_bound(10) == 0
        } else {
            false
        }
    }

    fn can_continue_to_use(&mut self, _mob: &mut Mob) -> bool {
        false
    }

    fn start(&mut self, mob: &mut Mob) {
        mob.horse_stand_if_possible();
        if let Some(sound) = mob.horse_ambient_stand_sound() {
            mob.living.entity.play_sound(Some(&sound), 0.4, 1.0);
        }
    }
}

/// `PARTNER_TARGETING` для BreedGoal — `forNonCombat().range(8).ignoreLineOfSight()`.
fn breed_partner_targeting() -> crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions {
    let mut t = crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions::for_non_combat();
    t.range(8.0);
    t.ignore_line_of_sight();
    t
}

/// `BreedGoal` — поиск партнёра и размножение (Animal).
pub struct BreedGoal {
    speed_modifier: f64,
    partner: Option<LivingEntity>,
    love_time: i32,
    targeting: crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions,
}

impl BreedGoal {
    /// `new(animal, speedModifier)` — partnerClass = класс животного
    /// (в Rust: тот же entity_type — проверяется в `can_mate`).
    pub fn new(_mob: &Mob, speed_modifier: f64) -> Self {
        Self {
            speed_modifier,
            partner: None,
            love_time: 0,
            targeting: breed_partner_targeting(),
        }
    }

    /// `getFreePartner()` — ближайший подходящий партнёр в 8 блоках.
    fn get_free_partner(&mut self, mob: &mut Mob) -> Option<LivingEntity> {
        let level = mob.living.entity.level.clone()?;
        let bb = mob.living.entity.get_bounding_box().inflate(8.0, 8.0, 8.0);
        let mut best: Option<LivingEntity> = None;
        let mut best_d = f64::MAX;
        for e in level.entities_in_box(&bb) {
            if e.uuid == mob.living.entity.uuid || !e.is_living {
                continue;
            }
            let partner = LivingEntity::from_entity(e);
            // `canMate && !isPanicking && distanceToSqr < d` (+ TargetingConditions).
            if !mob.living.entity.can_mate(&partner.entity) {
                continue;
            }
            if !partner.is_alive() {
                continue;
            }
            let d = mob.living.entity.distance_to_sqr(&partner.entity);
            if d >= best_d {
                continue;
            }
            let ok = self.targeting.test(&level, Some(&mut *mob), &partner);
            if !ok {
                continue;
            }
            best = Some(partner);
            best_d = d;
        }
        best
    }

    /// `breed()` — `spawnChildFromBreeding`.
    fn breed(&mut self, mob: &mut Mob) {
        let (Some(partner), Some(level)) = (self.partner.clone(), mob.living.entity.level.clone())
        else {
            return;
        };
        mob.spawn_child_from_breeding(&level, partner.entity.uuid);
    }
}

impl Goal for BreedGoal {
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Move, GoalFlag::Look])
    }

    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if !mob.is_in_love() {
            return false;
        }
        self.partner = self.get_free_partner(mob);
        self.partner.is_some()
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(p) = &self.partner else {
            return false;
        };
        p.is_alive() && p.entity.is_in_love() && self.love_time < 60
    }

    fn stop(&mut self, _mob: &mut Mob) {
        self.partner = None;
        self.love_time = 0;
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(partner) = self.partner.clone() else { return };
        mob.look_control.set_look_at_entity_deltas(
            &partner.entity,
            10.0,
            mob.get_max_head_x_rot(),
        );
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().move_to_entity(mob, &partner, self.speed_modifier);
        }
        self.love_time += 1;
        if self.love_time >= self.adjusted_tick_delay(60)
            && mob.living.entity.distance_to_sqr(&partner.entity) < 9.0
        {
            self.breed(mob);
        }
    }
}

/// `FollowParentGoal` — детёныш следует за родителем (Animal).
pub struct FollowParentGoal {
    speed_modifier: f64,
    parent: Option<LivingEntity>,
    time_to_recalc_path: i32,
}

impl FollowParentGoal {
    /// `new(animal, speedModifier)`.
    pub fn new(_mob: &Mob, speed_modifier: f64) -> Self {
        Self {
            speed_modifier,
            parent: None,
            time_to_recalc_path: 0,
        }
    }
}

impl Goal for FollowParentGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.get_age() >= 0 {
            return false;
        }
        let Some(level) = mob.living.entity.level.clone() else {
            return false;
        };
        let bb = mob
            .living
            .entity
            .get_bounding_box()
            .inflate(8.0, 4.0, 8.0);
        let mut best: Option<LivingEntity> = None;
        let mut best_d = f64::MAX;
        for e in level.entities_in_box(&bb) {
            // `getEntitiesOfClass(animal.getClass())` — тот же entity_type.
            if e.entity_type != mob.living.entity.entity_type {
                continue;
            }
            if e.get_age() < 0 {
                continue;
            }
            let d = mob.living.entity.distance_to_sqr(&e);
            if d <= best_d {
                best_d = d;
                best = Some(LivingEntity::from_entity(e));
            }
        }
        let Some(parent) = best else {
            return false;
        };
        // `d < 9.0` (DONT_FOLLOW_IF_CLOSER_THAN²) → false.
        if best_d < 9.0 {
            return false;
        }
        self.parent = Some(parent);
        true
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        if mob.get_age() >= 0 {
            return false;
        }
        let Some(parent) = &self.parent else {
            return false;
        };
        if !parent.is_alive() {
            return false;
        }
        let d = mob.living.entity.distance_to_sqr(&parent.entity);
        !(d < 9.0) && !(d > 256.0)
    }

    fn start(&mut self, _mob: &mut Mob) {
        self.time_to_recalc_path = 0;
    }

    fn stop(&mut self, _mob: &mut Mob) {
        self.parent = None;
    }

    fn tick(&mut self, mob: &mut Mob) {
        self.time_to_recalc_path -= 1;
        if self.time_to_recalc_path <= 0 {
            self.time_to_recalc_path = self.adjusted_tick_delay(10);
            let parent = self.parent.clone();
            if let (Some(p), Some(nav)) = (parent, mob.get_navigation()) {
                nav.borrow_mut().move_to_entity(mob, &p, self.speed_modifier);
            }
        }
    }
}
