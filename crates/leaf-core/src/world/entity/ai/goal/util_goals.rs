//! Перенос utility-целей `net.minecraft.world.entity.ai.goal`:
//! LeapAtTargetGoal, SwellGoal, UseItemGoal, InteractGoal,
//! MoveTowardsRestrictionGoal, TryFindWaterGoal, AvoidEntityGoal,
//! RestrictSunGoal, FleeSunGoal.

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::entity::equipment_slot::EquipmentSlot;
use crate::sounds::sound_event::SoundEvent;
use crate::tags::fluid_tags::FluidTags;
use crate::util::mth;
use crate::world::entity::ai::goal::combat::no_creative_or_spectator;
use crate::world::entity::ai::goal::goal::{Goal, GoalFlag, GoalFlagSet};
use crate::world::entity::ai::goal::movement_look::LookAtPlayerGoal;
use crate::world::entity::ai::util::default_random_pos;
use crate::world::entity::ai::util::goal_utils;
use crate::world::entity::ai::util::land_random_pos;
use crate::world::entity::entity::Entity;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::{Mob, MobPathing};
use crate::world::item::item_stack::ItemStack;
use crate::world::phys::vec3::Vec3;
use crate::world::level::pathfinder::path::Path;
use std::sync::Arc;

/// `LeapAtTargetGoal` — прыжок к цели (волки, лисы, пауки).
pub struct LeapAtTargetGoal {
    yd: f32,
    target: Option<LivingEntity>,
}

impl LeapAtTargetGoal {
    /// `new(mob, yd)`.
    pub fn new(_mob: &Mob, yd: f32) -> Self {
        Self { yd, target: None }
    }
}

impl Goal for LeapAtTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.has_controlling_passenger() {
            return false;
        }
        let Some(target) = mob.get_target().cloned() else {
            return false;
        };
        let d = mob.living.entity.distance_to_sqr(&target.entity);
        if !(4.0..=16.0).contains(&d) || !mob.living.entity.on_ground() {
            return false;
        }
        if mob.random.borrow_mut().next_int_bound(Self::reduced_tick_delay_static(5)) != 0 {
            return false;
        }
        self.target = Some(target);
        true
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        !mob.living.entity.on_ground()
    }

    fn start(&mut self, mob: &mut Mob) {
        let Some(target) = &self.target else { return };
        let delta_movement = mob.living.entity.get_delta_movement();
        let mut vec3 = Vec3::new(
            target.entity.get_x() - mob.living.entity.get_x(),
            0.0,
            target.entity.get_z() - mob.living.entity.get_z(),
        );
        if vec3.length_sqr() > 1.0e-7 {
            vec3 = vec3
                .normalize()
                .scale(0.4)
                .add_vec(&delta_movement.scale(0.2));
        }
        mob.living
            .entity
            .set_delta_movement(vec3.x, self.yd as f64, vec3.z);
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Jump, GoalFlag::Move].into_iter().collect()
    }
}

/// `SwellGoal` — крипер: вздутие перед взрывом.
pub struct SwellGoal {
    target: Option<LivingEntity>,
}

impl SwellGoal {
    /// `new(creeper)`.
    pub fn new(_mob: &Mob) -> Self {
        Self { target: None }
    }
}

impl Goal for SwellGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let target = mob.get_target().cloned();
        mob.get_swell_dir() > 0
            || target
                .as_ref()
                .map(|t| mob.living.entity.distance_to_sqr(&t.entity) < 9.0)
                .unwrap_or(false)
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().stop(mob);
        }
        self.target = mob.get_target().cloned();
    }

    fn stop(&mut self, _mob: &mut Mob) {
        self.target = None;
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(target) = &self.target else {
            mob.set_swell_dir(-1);
            return;
        };
        let too_far = mob.living.entity.distance_to_sqr(&target.entity) > 49.0;
        if too_far || !no_creative_or_spectator(Some(target)) {
            mob.set_swell_dir(-1);
        } else if !mob.has_line_of_sight(&target.entity) {
            mob.set_swell_dir(-1);
        } else {
            mob.set_swell_dir(1);
            // Purpur `creeperEncircleTarget` (Leaf): обход цели
            // по орбите во время вздутия.
            let encircle = mob
                .level()
                .map(|l| l.purpur_creeper_encircle_target())
                .unwrap_or(false);
            if encircle {
                let relative = mob
                    .living
                    .entity
                    .position()
                    .subtract_vec(&target.entity.position())
                    .y_rot(std::f32::consts::FRAC_PI_3)
                    .normalize()
                    .multiply(2.0, 2.0, 2.0);
                let destination = target.entity.position().add_vec(&relative);
                if let Some(nav) = mob.get_navigation() {
                    nav.borrow_mut().move_to_pos(
                        mob,
                        destination.x,
                        destination.y,
                        destination.z,
                        1.0,
                    );
                }
            }
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move].into_iter().collect()
    }
}

/// `UseItemGoal` — использование предмета (книга, еда и т.п.).
pub struct UseItemGoal {
    item: ItemStack,
    can_use_selector: Box<dyn Fn(&Mob) -> bool>,
    finish_using_sound: Option<Holder<SoundEvent>>,
}

impl UseItemGoal {
    /// `new(mob, item, finishUsingSound, canUseSelector)`.
    pub fn new(
        mob: &Mob,
        item: ItemStack,
        finish_using_sound: Option<Holder<SoundEvent>>,
        can_use_selector: impl Fn(&Mob) -> bool + 'static,
    ) -> Self {
        Self {
            item,
            can_use_selector: Box::new(can_use_selector),
            finish_using_sound,
        }
    }
}

impl Goal for UseItemGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        (self.can_use_selector)(mob)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        mob.living.is_using_item()
    }

    fn start(&mut self, mob: &mut Mob) {
        mob.living
            .set_item_slot(EquipmentSlot::Mainhand, self.item.clone());
        let arm = mob.living.get_main_arm();
        mob.living.start_using_item(arm);
    }

    fn stop(&mut self, mob: &mut Mob) {
        mob.living
            .set_item_slot(EquipmentSlot::Mainhand, ItemStack::empty());
        if let Some(sound) = &self.finish_using_sound {
            let pitch = mob.random.borrow_mut().next_float() * 0.2 + 0.9;
            mob.living.entity.play_sound(Some(sound), 1.0, pitch);
        }
    }
}

/// `InteractGoal` — LookAtPlayerGoal + MOVE-флаг
/// (взаимодействие с игроком/мобом).
pub struct InteractGoal {
    inner: LookAtPlayerGoal,
}

impl InteractGoal {
    /// `new(mob, lookAtType, lookDistance)` — `is_player_type`
    /// соответствует `lookAtType == Player.class`.
    pub fn new(_mob: &Mob, is_player_type: bool, look_distance: f32) -> Self {
        Self {
            inner: LookAtPlayerGoal::full(look_distance, 0.02, false, is_player_type),
        }
    }

    /// `new(mob, lookAtType, lookDistance, probability)`.
    pub fn with_probability(
        _mob: &Mob,
        is_player_type: bool,
        look_distance: f32,
        probability: f32,
    ) -> Self {
        Self {
            inner: LookAtPlayerGoal::full(look_distance, probability, false, is_player_type),
        }
    }
}

impl Goal for InteractGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        self.inner.can_use(mob)
    }
    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.inner.can_continue_to_use(mob)
    }
    fn start(&mut self, mob: &mut Mob) {
        self.inner.start(mob);
    }
    fn stop(&mut self, mob: &mut Mob) {
        self.inner.stop(mob);
    }
    fn tick(&mut self, mob: &mut Mob) {
        self.inner.tick(mob);
    }
    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Look, GoalFlag::Move].into_iter().collect()
    }
}

/// `MoveTowardsRestrictionGoal` — возврат в зону ограничения
/// (restrained area / home).
pub struct MoveTowardsRestrictionGoal {
    wanted_x: f64,
    wanted_y: f64,
    wanted_z: f64,
    speed_modifier: f64,
}

impl MoveTowardsRestrictionGoal {
    /// `new(mob, speedModifier)`.
    pub fn new(_mob: &Mob, speed_modifier: f64) -> Self {
        Self {
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
            speed_modifier,
        }
    }
}

impl Goal for MoveTowardsRestrictionGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.is_within_home_pos() {
            return false;
        }
        let home = mob.get_home_position();
        let center = crate::world::phys::vec3::at_bottom_center_of(&home.to_vec3i());
        let Some(pos) = default_random_pos::get_pos_towards(
            mob,
            16,
            7,
            center,
            std::f64::consts::FRAC_PI_2,
        ) else {
            return false;
        };
        self.wanted_x = pos.x;
        self.wanted_y = pos.y;
        self.wanted_z = pos.z;
        true
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        !mob.navigation_is_done()
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().move_to_pos(
                mob,
                self.wanted_x,
                self.wanted_y,
                self.wanted_z,
                self.speed_modifier,
            );
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move].into_iter().collect()
    }
}

/// `TryFindWaterGoal` — поиск воды при нахождении на суше.
pub struct TryFindWaterGoal;

impl TryFindWaterGoal {
    /// `new(mob)`.
    pub fn new(_mob: &Mob) -> Self {
        Self
    }
}

impl Goal for TryFindWaterGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if !mob.living.entity.on_ground() {
            return false;
        }
        let pos = mob.living.entity.block_position();
        let Some(level) = mob.level() else { return false };
        let in_water = level
            .get_fluid_state(pos)
            .map(|f| f.is_in_named_tag(&FluidTags::water()))
            .unwrap_or(false);
        !in_water
    }

    fn start(&mut self, mob: &mut Mob) {
        let e = &mob.living.entity;
        let min = BlockPos::new(
            mth::floor(e.get_x() - 2.0),
            mth::floor(e.get_y() - 2.0),
            mth::floor(e.get_z() - 2.0),
        );
        let max = BlockPos::new(
            mth::floor(e.get_x() + 2.0),
            e.get_block_y(),
            mth::floor(e.get_z() + 2.0),
        );
        let Some(level) = mob.level() else { return };
        for pos in BlockPos::between_closed(&min, &max) {
            let in_water = level
                .get_fluid_state(pos)
                .map(|f| f.is_in_named_tag(&FluidTags::water()))
                .unwrap_or(false);
            if in_water {
                mob.move_control
                    .set_wanted_position(pos.x as f64, pos.y as f64, pos.z as f64, 1.0);
                return;
            }
        }
    }
}

/// `AvoidEntityGoal` — избегание сущности указанного класса
/// (хищники убегают от игрока и т.п.).
///
/// `avoid_class` — предикат по `Entity` (аналог `Class<T>`),
/// `avoid_predicate`/`predicate_on_avoid_entity` —
/// `Predicate<LivingEntity>` из Java-конструкторов.
pub struct AvoidEntityGoal {
    walk_speed_modifier: f64,
    sprint_speed_modifier: f64,
    to_avoid: Option<LivingEntity>,
    path: Option<Path>,
    max_dist: f32,
    avoid_class: Box<dyn Fn(&Entity) -> bool>,
    avoid_predicate: Arc<dyn Fn(&LivingEntity) -> bool + Send + Sync>,
    predicate_on_avoid_entity: Arc<dyn Fn(&LivingEntity) -> bool + Send + Sync>,
}

impl AvoidEntityGoal {
    /// `new(mob, avoidClass, avoidPredicate, maxDist,
    /// walkSpeedModifier, sprintSpeedModifier,
    /// predicateOnAvoidEntity)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mob: &Mob,
        avoid_class: impl Fn(&Entity) -> bool + 'static,
        avoid_predicate: impl Fn(&LivingEntity) -> bool + Send + Sync + 'static,
        max_dist: f32,
        walk_speed_modifier: f64,
        sprint_speed_modifier: f64,
        predicate_on_avoid_entity: impl Fn(&LivingEntity) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            walk_speed_modifier,
            sprint_speed_modifier,
            to_avoid: None,
            path: None,
            max_dist,
            avoid_class: Box::new(avoid_class),
            avoid_predicate: Arc::new(avoid_predicate),
            predicate_on_avoid_entity: Arc::new(predicate_on_avoid_entity),
        }
    }

    /// `new(mob, avoidClass, maxDist, walkSpeed, sprintSpeed)` —
    /// дефолтные предикаты (`entity -> true` / NO_CREATIVE_OR_SPECTATOR).
    pub fn new_simple(
        mob: &Mob,
        avoid_class: impl Fn(&Entity) -> bool + 'static,
        max_dist: f32,
        walk_speed_modifier: f64,
        sprint_speed_modifier: f64,
    ) -> Self {
        Self::new(
            mob,
            avoid_class,
            |_| true,
            max_dist,
            walk_speed_modifier,
            sprint_speed_modifier,
            |le| no_creative_or_spectator(Some(le)),
        )
    }

    /// `getServerLevel(mob).getNearestEntity(...)` — ближайшая
    /// подходящая живая сущность в AABB (maxDist, 3, maxDist),
    /// проходящая `TargetingConditions.forCombat().range(maxDist)`.
    fn nearest_to_avoid(&mut self, mob: &mut Mob) -> Option<LivingEntity> {
        let level = mob.level()?;
        let box_ = mob.living.entity.bounding_box.inflate(
            self.max_dist as f64,
            3.0,
            self.max_dist as f64,
        );
        let candidates: Vec<Entity> = level
            .entities_in_box(&box_)
            .into_iter()
            .filter(|e| {
                e.is_living_entity()
                    && (self.avoid_class)(e)
                    && (self.avoid_predicate)(&LivingEntity::from_entity(e.clone()))
            })
            .collect();
        let mut conditions = crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions::for_combat();
        let on_avoid = self.predicate_on_avoid_entity.clone();
        conditions.range(self.max_dist as f64).selector(Arc::new(
            move |le: &LivingEntity, _level: &dyn crate::world::level::level_reader::LevelReader| {
                (on_avoid)(le)
            },
        ));
        let mut best: Option<LivingEntity> = None;
        let mut best_d = f64::MAX;
        let (mx, my, mz) = (mob.living.entity.get_x(), mob.living.entity.get_y(), mob.living.entity.get_z());
        for e in candidates {
            let le = LivingEntity::from_entity(e);
            if !le.entity.is_alive() {
                continue;
            }
            if !conditions.test(&level, Some(&mut *mob), &le) {
                continue;
            }
            let d = le.entity.distance_to_sqr_pos(mx, my, mz);
            if d < best_d {
                best_d = d;
                best = Some(le);
            }
        }
        best
    }
}

impl Goal for AvoidEntityGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let Some(to_avoid) = self.nearest_to_avoid(mob) else {
            self.to_avoid = None;
            return false;
        };
        let Some(pos_away) = land_random_pos::get_pos_away(
            mob,
            16,
            7,
            to_avoid.entity.position(),
        ) else {
            self.to_avoid = None;
            return false;
        };
        let closer_to_mob = to_avoid
            .entity
            .distance_to_sqr_pos(pos_away.x, pos_away.y, pos_away.z)
            < to_avoid.entity.distance_to_sqr(&mob.living.entity);
        if closer_to_mob {
            self.to_avoid = None;
            return false;
        }
        let path = mob.get_navigation().and_then(|nav| {
            nav.borrow_mut().create_path_pos(
                mob,
                BlockPos::containing(pos_away.x, pos_away.y, pos_away.z),
                0,
            )
        });
        match path {
            Some(p) => {
                self.path = Some(p);
                self.to_avoid = Some(to_avoid);
                true
            }
            None => {
                self.to_avoid = None;
                false
            }
        }
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        !mob.navigation_is_done()
    }

    fn start(&mut self, mob: &mut Mob) {
        if let (Some(nav), Some(path)) = (mob.get_navigation(), self.path.take()) {
            nav.borrow_mut().move_to_path(mob, Some(path), self.walk_speed_modifier);
        }
    }

    fn stop(&mut self, _mob: &mut Mob) {
        self.to_avoid = None;
    }

    fn tick(&mut self, mob: &mut Mob) {
        let (Some(to_avoid), Some(nav)) = (&self.to_avoid, mob.get_navigation()) else {
            return;
        };
        let d = mob.living.entity.distance_to_sqr(&to_avoid.entity);
        let speed = if d < 49.0 {
            self.sprint_speed_modifier
        } else {
            self.walk_speed_modifier
        };
        nav.borrow_mut().set_speed_modifier(speed);
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move].into_iter().collect()
    }
}

/// `RestrictSunGoal` — ходьба только в тени (зомби и т.п.).
pub struct RestrictSunGoal;

impl RestrictSunGoal {
    /// `new(mob)`.
    pub fn new(_mob: &Mob) -> Self {
        Self
    }
}

impl Goal for RestrictSunGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let Some(level) = mob.level() else { return false };
        let head = mob
            .living
            .get_item_by_slot(EquipmentSlot::Head)
            .clone();
        level.is_bright_outside()
            && head.is_empty()
            && goal_utils::has_ground_path_navigation(mob)
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().set_avoid_sun(true);
        }
    }

    fn stop(&mut self, mob: &mut Mob) {
        if goal_utils::has_ground_path_navigation(mob) {
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().set_avoid_sun(false);
            }
        }
    }
}

/// `FleeSunGoal` — бегство от солнца (выбор позиции в тени).
pub struct FleeSunGoal {
    wanted_x: f64,
    wanted_y: f64,
    wanted_z: f64,
    speed_modifier: f64,
}

impl FleeSunGoal {
    /// `new(mob, speedModifier)`.
    pub fn new(_mob: &Mob, speed_modifier: f64) -> Self {
        Self {
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
            speed_modifier,
        }
    }

    /// `getHidePos()` — до 10 попыток найти позицию без неба
    /// с отрицательным walk-target-value.
    fn get_hide_pos(&self, mob: &mut Mob) -> Option<Vec3> {
        let level = mob.level()?;
        let block_pos = mob.living.entity.block_position();
        for _ in 0..10 {
            let mut r = mob.random.borrow_mut();
            let bp = BlockPos::new(
                block_pos.x + r.next_int_range(-10, 10),
                block_pos.y + r.next_int_range(-3, 3),
                block_pos.z + r.next_int_range(-10, 10),
            );
            drop(r);
            if !level.can_see_sky(bp) && mob.get_walk_target_value(bp) < 0.0 {
                return Some(crate::world::phys::vec3::at_bottom_center_of(
                    &bp.to_vec3i(),
                ));
            }
        }
        None
    }
}

impl Goal for FleeSunGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let Some(level) = mob.level() else { return false };
        if mob.get_target().is_some()
            || !level.is_bright_outside()
            || !mob.living.entity.is_on_fire()
            || !level.can_see_sky(mob.living.entity.block_position())
        {
            return false;
        }
        let head = mob
            .living
            .get_item_by_slot(EquipmentSlot::Head)
            .clone();
        if !head.is_empty() {
            return false;
        }
        let Some(hide) = self.get_hide_pos(mob) else {
            return false;
        };
        self.wanted_x = hide.x;
        self.wanted_y = hide.y;
        self.wanted_z = hide.z;
        true
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        !mob.navigation_is_done()
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().move_to_pos(
                mob,
                self.wanted_x,
                self.wanted_y,
                self.wanted_z,
                self.speed_modifier,
            );
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move].into_iter().collect()
    }
}

/// `BreathAirGoal` — поиск воздуха и всплытие (утонувшие мобы).
pub struct BreathAirGoal;

impl BreathAirGoal {
    /// `givesAir(level, pos)` — нет жидкости (или bubble_column)
    /// и блок проходим для LAND-pathfinding.
    fn gives_air(
        level: &std::sync::Arc<dyn crate::world::level::level_reader::LevelReader>,
        pos: BlockPos,
    ) -> bool {
        let Some(state) = level.get_block_state(pos) else {
            return false;
        };
        let fluid_empty = level
            .get_fluid_state(pos)
            .map(|f| f.is_empty())
            .unwrap_or(true);
        let bubble = state.block.id.path == "bubble_column";
        if !fluid_empty && !bubble {
            return false;
        }
        state.is_pathfindable(
            level.as_ref(),
            pos,
            crate::world::level::pathfinder::path_computation_type::PathComputationType::Land,
        )
    }

    /// `findAirPosition` — спиральный поиск дающего воздух блока
    /// в Box(x±1, y..y+8, z±1), fallback — y+8.
    fn find_air_position(&mut self, mob: &mut Mob) {
        let Some(level) = mob.living.entity.level.clone() else {
            return;
        };
        let e = &mob.living.entity;
        let min_x = mth::floor(e.pos.x - 1.0) as i32;
        let min_y = e.get_block_y();
        let min_z = mth::floor(e.pos.z - 1.0) as i32;
        let max_x = mth::floor(e.pos.x + 1.0) as i32;
        let max_y = mth::floor(e.pos.y + 8.0) as i32;
        let max_z = mth::floor(e.pos.z + 1.0) as i32;
        let mut found: Option<BlockPos> = None;
        'outer: for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let pos = BlockPos::new(x, y, z);
                    if Self::gives_air(&level, pos) {
                        found = Some(pos);
                        break 'outer;
                    }
                }
            }
        }
        let target = found.unwrap_or_else(|| {
            BlockPos::containing(e.pos.x, e.pos.y + 8.0, e.pos.z)
        });
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().move_to_pos(
                mob,
                target.x as f64,
                target.y as f64 + 1.0,
                target.z as f64,
                1.0,
            );
        }
    }
}

impl Goal for BreathAirGoal {
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Move, GoalFlag::Look])
    }

    fn can_use(&mut self, mob: &mut Mob) -> bool {
        mob.living.entity.get_air_supply() < 140
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.can_use(mob)
    }

    fn is_interruptable(&self) -> bool {
        false
    }

    fn start(&mut self, mob: &mut Mob) {
        self.find_air_position(mob);
    }

    fn tick(&mut self, mob: &mut Mob) {
        self.find_air_position(mob);
        let input = Vec3::new(
            mob.living.xxa as f64,
            mob.living.yya as f64,
            mob.living.zza as f64,
        );
        mob.living.move_relative_friction(0.02, input);
        let delta = mob.living.entity.get_delta_movement();
        mob.living.entity.r#move(delta);
    }
}

/// `ClimbOnTopOfPowderSnowGoal` — прыжок из рыхлого снега наверх
/// (POWDER_SNOW_WALKABLE_MOBS: polar_bear, strider).
pub struct ClimbOnTopOfPowderSnowGoal;

impl ClimbOnTopOfPowderSnowGoal {
    /// `getType().is(POWDER_SNOW_WALKABLE_MOBS)` — через entity_type
    /// holder (Direct) и tag-реестр.
    fn is_walkable_mob(mob: &Mob) -> bool {
        let Some(t) = (match &mob.living.entity.entity_type {
            crate::core::holder::Holder::Direct(t) => Some(t),
            _ => None,
        }) else {
            return false;
        };
        crate::tags::entity_type_tags::is_entity_type_in_tag(
            t,
            &crate::tags::entity_type_tags::keys::powder_snow_walkable_mobs(),
        )
    }
}

impl Goal for ClimbOnTopOfPowderSnowGoal {
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Jump])
    }

    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let e = &mob.living.entity;
        let flag = e.was_in_powder_snow || e.is_in_powder_snow();
        if !(flag && Self::is_walkable_mob(mob)) {
            return false;
        }
        let Some(level) = e.level.clone() else {
            return false;
        };
        let block_pos = e.block_position().above();
        let Some(state) = level.get_block_state(block_pos) else {
            return false;
        };
        state.block.id.path == "powder_snow"
            || state.collision_shape_unleveled().is_empty()
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        mob.jump_control.jump();
    }
}

/// `DolphinJumpGoal extends JumpGoal` — выпрыгивание дельфина
/// из воды (проверка свободного пути + звуки/питч).
pub struct DolphinJumpGoal {
    interval: i32,
    breached: bool,
}

/// `STEPS_TO_CHECK` — дистанции проверки пути.
const DOLPHIN_STEPS_TO_CHECK: [i32; 6] = [0, 1, 4, 5, 6, 7];

impl DolphinJumpGoal {
    /// `new(dolphin, interval)` — `reducedTickDelay(interval)`.
    pub fn new(_mob: &Mob, interval: i32) -> Self {
        Self {
            interval: Self::reduced_tick_delay_static(interval),
            breached: false,
        }
    }

    /// `waterIsClear` — жидкость WATER и нет блокирующего блока.
    fn water_is_clear(
        level: &std::sync::Arc<dyn crate::world::level::level_reader::LevelReader>,
        pos: BlockPos,
        dx: i32,
        dz: i32,
        scale: i32,
    ) -> bool {
        let p = BlockPos::new(
            pos.x + dx * scale,
            pos.y,
            pos.z + dz * scale,
        );
        let water = level
            .get_fluid_state(p)
            .map(|f| {
                f.is_in_named_tag(&crate::tags::fluid_tags::FluidTags::water())
            })
            .unwrap_or(false);
        let clear = level
            .get_block_state(p)
            .map(|s| !s.blocks_motion())
            .unwrap_or(false);
        water && clear
    }

    /// `surfaceIsClear` — воздух на +1 и +2 над водой.
    fn surface_is_clear(
        level: &std::sync::Arc<dyn crate::world::level::level_reader::LevelReader>,
        pos: BlockPos,
        dx: i32,
        dz: i32,
        scale: i32,
    ) -> bool {
        let p1 = BlockPos::new(pos.x + dx * scale, pos.y + 1, pos.z + dz * scale);
        let p2 = BlockPos::new(pos.x + dx * scale, pos.y + 2, pos.z + dz * scale);
        level
            .get_block_state(p1)
            .map(|s| s.is_air())
            .unwrap_or(false)
            && level
                .get_block_state(p2)
                .map(|s| s.is_air())
                .unwrap_or(false)
    }
}

impl Goal for DolphinJumpGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.random.borrow_mut().next_int_bound(self.interval) != 0 {
            return false;
        }
        let Some(level) = mob.living.entity.level.clone() else {
            return false;
        };
        let dir = mob.living.entity.get_motion_direction();
        let (dx, dz) = (dir.get_step_x(), dir.get_step_z());
        let pos = mob.living.block_position();
        for &i in &DOLPHIN_STEPS_TO_CHECK {
            if !Self::water_is_clear(&level, pos, dx, dz, i)
                || !Self::surface_is_clear(&level, pos, dx, dz, i)
            {
                return false;
            }
        }
        true
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let e = &mob.living.entity;
        let d = e.get_delta_movement().y;
        let cond = !(d * d < 0.03)
            || e.get_x_rot() == 0.0
            || e.get_x_rot().abs() >= 10.0
            || !e.is_in_water();
        cond && !e.on_ground()
    }

    fn is_interruptable(&self) -> bool {
        false
    }

    fn start(&mut self, mob: &mut Mob) {
        let dir = mob.living.entity.get_motion_direction();
        let cur = mob.living.entity.get_delta_movement();
        mob.living.entity.set_delta_movement(
            cur.x + dir.get_step_x() as f64 * 0.6,
            cur.y + 0.7,
            cur.z + dir.get_step_z() as f64 * 0.6,
        );
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().stop(mob);
        }
    }

    fn stop(&mut self, mob: &mut Mob) {
        mob.living.entity.set_x_rot(0.0);
    }

    fn tick(&mut self, mob: &mut Mob) {
        let was = self.breached;
        if !was {
            let level = mob.living.entity.level.clone();
            let pos = mob.living.block_position();
            self.breached = level
                .and_then(|l| l.get_fluid_state(pos))
                .map(|f| {
                    f.is_in_named_tag(&crate::tags::fluid_tags::FluidTags::water())
                })
                .unwrap_or(false);
        }
        if self.breached && !was {
            let sound = crate::sounds::sound_events::dolphin_jump();
            mob.living.entity.play_sound(Some(&sound), 1.0, 1.0);
        }
        let delta = mob.living.entity.get_delta_movement();
        if delta.y * delta.y < 0.03 && mob.living.entity.get_x_rot() != 0.0 {
            let xrot = crate::util::mth::rot_lerp_f32(0.2, mob.living.entity.get_x_rot(), 0.0);
            mob.living.entity.set_x_rot(xrot);
        } else if delta.length() > 1.0e-5 {
            let d = (delta.x * delta.x + delta.z * delta.z).sqrt();
            let d1 = (-delta.y).atan2(d) * 180.0 / std::f64::consts::PI;
            mob.living.entity.set_x_rot(d1 as f32);
        }
    }
}

/// `EntityEvent.EAT_GRASS` = 10.
const ENTITY_EVENT_EAT_GRASS: u8 = 10;
/// `LevelEvent.PARTICLES_DESTROY_BLOCK` = 2001.
const LEVEL_EVENT_PARTICLES_DESTROY_BLOCK: i32 = 2001;

/// `EatBlockGoal` — поедание травы/растений (овцы и др.).
pub struct EatBlockGoal {
    eat_animation_tick: i32,
}

impl EatBlockGoal {
    /// `IS_EDIBLE` — `BlockTags.EDIBLE_FOR_SHEEP`.
    fn is_edible(state: &crate::world::level::block::state::block_state::BlockState) -> bool {
        state.is_block_tag(&crate::tags::block_tags::BlockTags::edible_for_sheep())
    }
}

impl Goal for EatBlockGoal {
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Move, GoalFlag::Look, GoalFlag::Jump])
    }

    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let baby = mob.living.entity.is_baby();
        let delay = if baby { 50 } else { 1000 };
        if mob.random.borrow_mut().next_int_bound(self.adjusted_tick_delay(delay)) != 0 {
            return false;
        }
        let Some(level) = mob.living.entity.level.clone() else {
            return false;
        };
        let pos = mob.living.block_position();
        let edible = level
            .get_block_state(pos)
            .map(|s| Self::is_edible(&s))
            .unwrap_or(false);
        edible
            || level
                .get_block_state(pos.below())
                .map(|s| s.block.id.path == "grass_block")
                .unwrap_or(false)
    }

    fn can_continue_to_use(&mut self, _mob: &mut Mob) -> bool {
        self.eat_animation_tick > 0
    }

    fn start(&mut self, mob: &mut Mob) {
        self.eat_animation_tick = self.adjusted_tick_delay(40);
        let id = mob.living.entity.id;
        if let Some(level) = mob.living.entity.level.clone() {
            level.broadcast_entity_event(id, ENTITY_EVENT_EAT_GRASS);
        }
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().stop(mob);
        }
    }

    fn stop(&mut self, _mob: &mut Mob) {
        self.eat_animation_tick = 0;
    }

    fn tick(&mut self, mob: &mut Mob) {
        self.eat_animation_tick = (self.eat_animation_tick - 1).max(0);
        if self.eat_animation_tick != self.adjusted_tick_delay(4) {
            return;
        }
        let Some(level) = mob.living.entity.level.clone() else {
            return;
        };
        let pos = mob.living.block_position();
        let edible = level
            .get_block_state(pos)
            .map(|s| Self::is_edible(&s))
            .unwrap_or(false);
        if edible {
            if level.mob_griefing() {
                level.destroy_block(pos, false);
            }
            mob.ate();
        } else {
            let below = pos.below();
            let is_grass = level
                .get_block_state(below)
                .map(|s| s.block.id.path == "grass_block")
                .unwrap_or(false);
            if is_grass {
                if level.mob_griefing() {
                    // `PARTICLES_DESTROY_BLOCK` + `setBlock(DIRT)`.
                    let state_id = level
                        .get_block_state(below)
                        .map(|s| {
                            crate::world::level::block::state::block_state_registry::block_state_id_map()
                                .get_id(&s)
                        })
                        .unwrap_or(0);
                    level.level_event(LEVEL_EVENT_PARTICLES_DESTROY_BLOCK, below, state_id);
                    if let Some(dirt_block) = crate::world::level::block::block_by_identifier(
                        &crate::resources::identifier::Identifier::minecraft("dirt"),
                    ) {
                        let dirt = crate::world::level::block::state::block_state::BlockState::new(
                            dirt_block,
                            crate::world::level::block::state::state_holder::StateHolder::default(),
                        );
                        level.set_block_state(below, &dirt);
                    }
                }
                mob.ate();
            }
        }
    }
}

/// `FollowMobGoal` — следование за другим мобом (кроме своего класса).
pub struct FollowMobGoal {
    speed_modifier: f64,
    following: Option<LivingEntity>,
    time_to_recalc_path: i32,
    stop_distance: f32,
    old_water_cost: f32,
    area_size: f32,
}

impl FollowMobGoal {
    /// `new(mob, speedModifier, stopDistance, areaSize)`.
    pub fn new(_mob: &Mob, speed_modifier: f64, stop_distance: f32, area_size: f32) -> Self {
        Self {
            speed_modifier,
            following: None,
            time_to_recalc_path: 0,
            stop_distance,
            old_water_cost: 0.0,
            area_size,
        }
    }
}

impl Goal for FollowMobGoal {
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Move, GoalFlag::Look])
    }

    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let Some(level) = mob.living.entity.level.clone() else {
            return false;
        };
        let bb = mob
            .living
            .entity
            .get_bounding_box()
            .inflate_all(self.area_size as f64);
        // `getEntitiesOfClass(Mob, bb, mob.getClass() != other.getClass())`.
        let mut candidates: Vec<crate::world::entity::entity::Entity> = Vec::new();
        let own_type = mob.living.entity.entity_type.clone();
        level.for_each_mob_in_box(&bb, &mut |m| {
            if m.living.entity.entity_type != own_type {
                candidates.push(m.living.entity.clone());
            }
        });
        // Первый видимый кандидат.
        for e in candidates {
            if !e.is_invisible() {
                self.following = Some(LivingEntity::from_entity(e));
                return true;
            }
        }
        false
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(f) = &self.following else {
            return false;
        };
        !mob.navigation_is_done()
            && mob.living.entity.distance_to_sqr(&f.entity)
                > (self.stop_distance * self.stop_distance) as f64
    }

    fn start(&mut self, mob: &mut Mob) {
        self.time_to_recalc_path = 0;
        self.old_water_cost = mob.get_pathfinding_malus(crate::world::level::pathfinder::path_type::PathType::Water);
        mob.set_pathfinding_malus(crate::world::level::pathfinder::path_type::PathType::Water, 0.0);
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.following = None;
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().stop(mob);
        }
        mob.set_pathfinding_malus(crate::world::level::pathfinder::path_type::PathType::Water, self.old_water_cost);
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(f) = self.following.clone() else {
            return;
        };
        if mob.is_leashed() {
            return;
        }
        mob.look_control.set_look_at_entity_deltas(
            &f.entity,
            10.0,
            mob.get_max_head_x_rot(),
        );
        self.time_to_recalc_path -= 1;
        if self.time_to_recalc_path > 0 {
            return;
        }
        self.time_to_recalc_path = self.adjusted_tick_delay(10);
        let d = mob.living.entity.pos.x - f.entity.pos.x;
        let d1 = mob.living.entity.pos.y - f.entity.pos.y;
        let d2 = mob.living.entity.pos.z - f.entity.pos.z;
        let d3 = d * d + d1 * d1 + d2 * d2;
        if d3 > (self.stop_distance * self.stop_distance) as f64 {
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().move_to_entity(mob, &f, self.speed_modifier);
            }
        } else if let Some(nav) = mob.get_navigation() {
            // `navigation.stop()` + отход от цели, если lookControl
            // следящей сущности указывает на моба (view не имеет
            // lookControl — упрощение: отход при d3 <= stopDistance).
            nav.borrow_mut().stop(mob);
            if d3 <= self.stop_distance as f64 {
                let d4 = f.entity.pos.x - mob.living.entity.pos.x;
                let d5 = f.entity.pos.z - mob.living.entity.pos.z;
                nav.borrow_mut().move_to_pos(
                    mob,
                    mob.living.entity.pos.x - d4,
                    mob.living.entity.pos.y,
                    mob.living.entity.pos.z - d5,
                    self.speed_modifier,
                );
            }
        }
    }
}

/// `OcelotAttackGoal` — цилотатакующий: преследует `mob.target`,
/// дистанционный множитель скорости (0.8/1.33/0.6 по d3),
/// `doHurtTarget` каждые 20 тиков в melee-радиусе.
#[derive(Default)]
pub struct OcelotAttackGoal {
    /// `attackTime`.
    pub attack_time: i32,
}

impl OcelotAttackGoal {
    /// `new(mob)` — флаги MOVE+LOOK (в `setFlags`, сохраняем).
    pub fn new() -> Self {
        Self { attack_time: 0 }
    }
}

impl Goal for OcelotAttackGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        mob.get_target().is_some()
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(target) = mob.get_target().cloned() else {
            return false;
        };
        if !target.is_alive()
            || mob.living.entity.distance_to_sqr(&target.entity) > 225.0
        {
            return false;
        }
        let nav_done = mob
            .get_navigation()
            .as_ref()
            .map(|n| n.borrow().is_done())
            .unwrap_or(true);
        if nav_done {
            return self.can_use(mob);
        }
        true
    }

    fn start(&mut self, mob: &mut Mob) {
        self.attack_time = 0;
    }

    fn stop(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().stop(mob);
        }
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move, GoalFlag::Look].into_iter().collect()
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(target) = mob.get_target().cloned() else {
            return;
        };
        mob.look_control
            .set_look_at_entity_deltas(&target.entity, 30.0, 30.0);
        let width = mob.get_bb_width() as f64;
        let d = width * 2.0 * width * 2.0;
        let d1 = mob
            .living
            .entity
            .distance_to_sqr_pos(
                target.entity.get_x(),
                target.entity.get_y(),
                target.entity.get_z(),
            );
        let speed = if d1 > d && d1 < 16.0 {
            1.33
        } else if d1 < 225.0 {
            0.6
        } else {
            0.8
        };
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().move_to_entity(mob, &target, speed);
        }
        self.attack_time = (self.attack_time - 1).max(0);
        if d1 <= d && self.attack_time <= 0 {
            self.attack_time = 20;
            let mut t = target.clone();
            mob.do_hurt_target(&mut t);
        }
    }
}

/// `OfferFlowerGoal` — железный голем протягивает мак (poppy)
/// ближайшему кандидату `#candidate_for_iron_golem_gift`
/// каждые ~8000 тиков; на stop дарит цветок медному голему
/// (`#accepts_iron_golem_gift`, слот антенны).
pub struct OfferFlowerGoal {
    /// `entity` — кандидат на подарок.
    pub entity: Option<LivingEntity>,
    /// `tick` — оставшиеся тики жеста.
    pub tick: i32,
}

impl OfferFlowerGoal {
    /// `new(golem)`.
    pub fn new() -> Self {
        Self { entity: None, tick: 0 }
    }
}

impl Default for OfferFlowerGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for OfferFlowerGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let Some(level) = mob.level() else {
            return false;
        };
        if !level.is_bright_outside() {
            return false;
        }
        if mob.random.borrow_mut().next_int_bound(8000) != 0 {
            return false;
        }
        let box_ = mob
            .living
            .entity
            .get_bounding_box()
            .inflate(6.0, 2.0, 6.0);
        let mut conditions =
            crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions::for_non_combat();
        conditions.range(6.0);
        let own_uuid = mob.living.entity.uuid;
        let found = level
            .entities_in_box(&box_)
            .into_iter()
            .filter(|e| {
                e.is_living_entity()
                    && e.entity_type
                        .value()
                        .map(|t| {
                            crate::tags::entity_type_tags::entity_type_is(
                                t,
                                crate::tags::entity_type_tags::keys::candidate_for_iron_golem_gift(),
                            )
                        })
                        .unwrap_or(false)
            })
            .map(|e| LivingEntity::from_entity(e))
            .find(|le| {
                le.entity.uuid != own_uuid && conditions.test(&level, Some(mob), le)
            });
        self.entity = found;
        self.entity.is_some()
    }

    fn can_continue_to_use(&mut self, _mob: &mut Mob) -> bool {
        self.tick > 0
    }

    fn start(&mut self, mob: &mut Mob) {
        self.tick = self.adjusted_tick_delay(400);
        mob.golem_offer_flower(true);
    }

    fn stop(&mut self, mob: &mut Mob) {
        mob.golem_offer_flower(false);
        let gift = if self.tick == 0 {
            self.entity.as_ref().and_then(|entity| {
                let accepts = entity
                    .entity
                    .entity_type
                    .value()
                    .map(|t| {
                        crate::tags::entity_type_tags::entity_type_is(
                            t,
                            crate::tags::entity_type_tags::keys::accepts_iron_golem_gift(),
                        )
                    })
                    .unwrap_or(false);
                if accepts
                    && mob.copper_golem_antenna_empty()
                    && mob
                        .living
                        .entity
                        .get_bounding_box()
                        .intersects(&entity.entity.get_bounding_box())
                {
                    Some(())
                } else {
                    None
                }
            })
        } else {
            None
        };
        if gift.is_some() {
            mob.copper_golem_set_antenna_flower();
        }
        self.entity = None;
    }

    fn tick(&mut self, mob: &mut Mob) {
        if let Some(entity) = &self.entity {
            mob.look_control
                .set_look_at_entity_deltas(&entity.entity, 30.0, 30.0);
        }
        self.tick -= 1;
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move, GoalFlag::Look].into_iter().collect()
    }
}

/// `BegGoal` — волк выпрашивает еду/кость у ближайшего игрока
/// (LOOK-флаг, lookTime = 40+rand(40), визуал `setIsInterested`).
pub struct BegGoal {
    /// `player` — цель попрошайничества (Entity-view).
    pub player: Option<Entity>,
    /// `lookDistance`.
    pub look_distance: f32,
    /// `lookTime`.
    pub look_time: i32,
    /// `begTargeting` — `forNonCombat().range(lookDistance)`.
    pub beg_targeting:
        crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions,
}

impl BegGoal {
    /// `new(wolf, lookDistance)`.
    pub fn new(look_distance: f32) -> Self {
        let mut beg_targeting =
            crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions::for_non_combat();
        beg_targeting.range(look_distance as f64);
        Self { player: None, look_distance, look_time: 0, beg_targeting }
    }

    /// `playerHoldingInteresting(player)` — кость или еда волка
    /// в любой руке.
    fn player_holding_interesting(&self, mob: &Mob, player: &LivingEntity) -> bool {
        let bone =
            crate::resources::identifier::Identifier::with_default_namespace("bone").unwrap();
        let interesting = |stack: &crate::world::item::item_stack::ItemStack| {
            stack.is(&bone) || mob.wolf_is_food(stack)
        };
        interesting(player.get_main_hand_item())
            || interesting(player.get_offhand_item())
    }
}

impl Goal for BegGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let Some(level) = mob.level() else {
            return false;
        };
        let pos = mob.position();
        let Some(player_e) = level.nearest_player(pos, self.look_distance as f64) else {
            return false;
        };
        let player = LivingEntity::from_entity(player_e.clone());
        if !player_e.is_player()
            || !self.beg_targeting.test(&level, Some(mob), &player)
        {
            return false;
        }
        if !self.player_holding_interesting(mob, &player) {
            return false;
        }
        self.player = Some(player_e);
        true
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(player) = self.player.clone() else {
            return false;
        };
        let le = LivingEntity::from_entity(player);
        if !le.is_alive() {
            return false;
        }
        let d = mob.living.entity.distance_to_sqr(&le.entity);
        if d > (self.look_distance as f64) * (self.look_distance as f64) {
            return false;
        }
        if self.look_time <= 0 {
            return false;
        }
        self.player_holding_interesting(mob, &le)
    }

    fn start(&mut self, mob: &mut Mob) {
        mob.wolf_set_is_interested(true);
        self.look_time = self.adjusted_tick_delay(
            40 + mob.random.borrow_mut().next_int_bound(40),
        );
    }

    fn stop(&mut self, mob: &mut Mob) {
        mob.wolf_set_is_interested(false);
        self.player = None;
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(player) = self.player.clone() else {
            return;
        };
        mob.look_control.set_look_at_deltas(
            player.get_x(),
            player.get_eye_y(),
            player.get_z(),
            10.0,
            mob.get_max_head_x_rot(),
        );
        self.look_time -= 1;
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Look].into_iter().collect()
    }
}

/// `LookAtTradingPlayerGoal` — житель смотрит на торговца
/// (`villager.isTrading()` → `getTradingPlayer()`, dist 8).
pub struct LookAtTradingPlayerGoal {
    /// `lookAt` — Entity торговца.
    pub look_at: Option<Entity>,
    /// `lookDistance` = 8.0.
    pub look_distance: f32,
    /// `lookTime`.
    pub look_time: i32,
    /// `probability` = 0.02 (из `LookAtPlayerGoal(mob, Player, 8)`).
    pub probability: f32,
    /// `onlyHorizontal` = false.
    pub only_horizontal: bool,
}

impl LookAtTradingPlayerGoal {
    /// `new(villager)` — `super(villager, Player.class, 8.0F)`.
    pub fn new() -> Self {
        Self { look_at: None, look_distance: 8.0, look_time: 0, probability: 0.02, only_horizontal: false }
    }
}

impl Default for LookAtTradingPlayerGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for LookAtTradingPlayerGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if !mob.villager_is_trading() {
            return false;
        }
        self.look_at = mob.villager_trading_player();
        self.look_at.is_some()
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(look_at) = &self.look_at else {
            return false;
        };
        if look_at.is_removed() {
            return false;
        }
        let d = mob.living.entity.distance_to_sqr(look_at);
        if d > (self.look_distance as f64) * (self.look_distance as f64) {
            return false;
        }
        if mob.random.borrow_mut().next_float() >= self.probability {
            return false;
        }
        self.look_time = 40 + mob.random.borrow_mut().next_int_bound(40);
        true
    }

    fn stop(&mut self, _mob: &mut Mob) {
        self.look_at = None;
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(look_at) = self.look_at.clone() else {
            return;
        };
        self.look_time -= 1;
        mob.look_control.set_look_at_deltas(
            look_at.get_x(),
            look_at.get_eye_y(),
            look_at.get_z(),
            10.0,
            mob.get_max_head_x_rot(),
        );
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Look].into_iter().collect()
    }
}

/// `FollowFlockLeaderGoal` — школьная рыба следует за вожаком:
/// `nextStartTick` 200+rand(200)%20, при активации стая
/// регистрируется через `addFollowers`, путь к вожаку каждые
/// 10 тиков (`pathToLeader`).
pub struct FollowFlockLeaderGoal {
    /// `timeToRecalcPath`.
    pub time_to_recalc_path: i32,
    /// `nextStartTick`.
    pub next_start_tick: i32,
}

impl FollowFlockLeaderGoal {
    /// `new(mob)`.
    pub fn new() -> Self {
        Self { time_to_recalc_path: 0, next_start_tick: 0 }
    }

    /// `nextStartTick(taskOwner)` — `reducedTickDelay(200 + rand(200)%20)`.
    fn next_start_tick_of(&self, mob: &Mob) -> i32 {
        Self::reduced_tick_delay_static(
            200 + mob.random.borrow_mut().next_int_bound(200) % 20,
        )
    }
}

impl Default for FollowFlockLeaderGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for FollowFlockLeaderGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.fish_has_followers() {
            return false;
        }
        if mob.fish_is_follower() {
            return true;
        }
        if self.next_start_tick > 0 {
            self.next_start_tick -= 1;
            return false;
        }
        self.next_start_tick = self.next_start_tick_of(mob);
        let Some(level) = mob.level() else {
            return false;
        };
        let box_ = mob.living.entity.get_bounding_box().inflate(8.0, 8.0, 8.0);
        let own_uuid = mob.living.entity.uuid;
        let mut followers: Vec<LivingEntity> = Vec::new();
        for e in level.entities_in_box(&box_) {
            if !e.is_living_entity() {
                continue;
            }
            let le = LivingEntity::from_entity(e);
            if le.entity.uuid == own_uuid {
                continue;
            }
            let can_be_followed = le.mob_can_be_followed();
            let is_follower = le.mob_is_follower();
            if !can_be_followed && !is_follower {
                continue;
            }
            if !is_follower {
                followers.push(le);
            }
        }
        mob.fish_add_followers(followers);
        mob.fish_is_follower()
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        mob.fish_is_follower() && mob.fish_in_range_of_leader()
    }

    fn start(&mut self, _mob: &mut Mob) {
        self.time_to_recalc_path = 0;
    }

    fn stop(&mut self, mob: &mut Mob) {
        mob.fish_stop_following();
    }

    fn tick(&mut self, mob: &mut Mob) {
        self.time_to_recalc_path -= 1;
        if self.time_to_recalc_path <= 0 {
            self.time_to_recalc_path = self.adjusted_tick_delay(10);
            mob.fish_path_to_leader();
        }
    }
}

/// `EntityEvent.TAMING_FAILED` (net.minecraft.world.entity.EntityEvent).
pub const ENTITY_EVENT_TAMING_FAILED: u8 = 6;

/// `RunAroundLikeCrazyGoal` — дикий (не прирученный) конь в
/// повозке бегает кругами: moveTo случайной точке, каждые
/// ~50 тиков ejectPassengers + makeMad + TAMING_FAILED(6).
pub struct RunAroundLikeCrazyGoal {
    /// `speedModifier`.
    pub speed_modifier: f64,
    /// `posX/Y/Z`.
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
}

impl RunAroundLikeCrazyGoal {
    /// `new(horse, speedModifier)` — MOVE-флаг.
    pub fn new(speed_modifier: f64) -> Self {
        Self { speed_modifier, pos_x: 0.0, pos_y: 0.0, pos_z: 0.0 }
    }
}

impl Goal for RunAroundLikeCrazyGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.horse_is_mob_controlled() || mob.horse_is_tamed() || !mob.living.entity.is_vehicle() {
            return false;
        }
        let Some(pos) =
            crate::world::entity::ai::util::default_random_pos::get_pos(mob, 5, 4)
        else {
            return false;
        };
        self.pos_x = pos.x;
        self.pos_y = pos.y;
        self.pos_z = pos.z;
        true
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().move_to_pos(
                mob,
                self.pos_x,
                self.pos_y,
                self.pos_z,
                self.speed_modifier,
            );
        }
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        !mob.horse_is_tamed()
            && mob
                .get_navigation()
                .as_ref()
                .map(|n| !n.borrow().is_done())
                .unwrap_or(false)
            && mob.living.entity.is_vehicle()
    }

    fn tick(&mut self, mob: &mut Mob) {
        if mob.horse_is_tamed() {
            return;
        }
        if mob.random.borrow_mut().next_int_bound(self.adjusted_tick_delay(50)) != 0 {
            return;
        }
        let Some(first_passenger) = mob.living.entity.get_first_passenger().cloned() else {
            return;
        };
        if first_passenger.is_player() {
            let temper = mob.horse_get_temper();
            let max_temper = mob.horse_get_max_temper();
            // CraftBukkit EntityTameEvent / purpur alwaysTameInCreative
            // — на уровне event-bus'а; здесь vanilla-логика:
            // creative → мгновенное природжение, иначе rand(maxTemper)<temper.
            let tame = LivingEntity::from_entity(first_passenger.clone()).player_has_infinite_materials()
                || (max_temper > 0
                    && mob.random.borrow_mut().next_int_bound(max_temper) < temper);
            if tame {
                mob.horse_tame_with_name(&first_passenger);
                return;
            }
            mob.horse_modify_temper(5);
        }
        mob.eject_passengers();
        mob.horse_make_mad();
        if let Some(level) = mob.level() {
            level.broadcast_entity_event(mob.living.entity.id, ENTITY_EVENT_TAMING_FAILED);
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move].into_iter().collect()
    }
}
