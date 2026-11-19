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
