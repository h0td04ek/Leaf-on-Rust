//! Перенос `net.minecraft.world.entity.ai.goal` — movement/look
//! цели: FloatGoal, PanicGoal, RandomStrollGoal,
//! WaterAvoidingRandomStrollGoal, WaterAvoidingRandomFlyingGoal,
//! RandomSwimmingGoal, RandomLookAroundGoal, LookAtPlayerGoal,
//! JumpGoal, MoveTowardsTargetGoal.

use crate::core::block_pos::BlockPos;
use crate::tags::damage_type_tags;
use crate::tags::fluid_tags::FluidTags;
use crate::tags::tag_key::TagKey;
use crate::world::damagesource::damage_type::DamageType;
use crate::world::entity::ai::goal::goal::{Goal, GoalFlag, GoalFlagSet};
use crate::world::entity::ai::util::{air_and_water_random_pos, behavior_utils, default_random_pos, hover_random_pos, land_random_pos};
use crate::world::entity::entity::Entity;
use crate::world::entity::mob::Mob;
use crate::world::phys::vec3::Vec3;

/// `FloatGoal` — моб всплывает (JUMP-флаг; в конструкторе
/// Java вызывает `navigation.setCanFloat(true)` — у нас
/// выставляем при первом `can_use`/`start`, идемпотентно).
pub struct FloatGoal {
    can_float_set: bool,
}

impl FloatGoal {
    /// `new(mob)`.
    pub fn new() -> Self {
        Self { can_float_set: false }
    }
}

impl Default for FloatGoal {
    fn default() -> Self { Self::new() }
}

impl Goal for FloatGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        self.ensure_can_float(mob);
        (mob.living.is_in_water()
            && mob.living.entity.get_fluid_height(true) > mob.living.entity.get_fluid_jump_threshold())
            || mob.living.is_in_lava()
    }
    fn requires_update_every_tick(&self) -> bool { true }
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Jump])
    }
    fn tick(&mut self, mob: &mut Mob) {
        if mob.random.borrow_mut().next_float() < 0.8 {
            mob.jump_control.jump();
        }
    }
}

impl FloatGoal {
    fn ensure_can_float(&mut self, mob: &mut Mob) {
        if !self.can_float_set {
            if let Some(nav) = mob.navigation.clone() {
                nav.borrow_mut().set_can_float(true);
            }
            self.can_float_set = true;
        }
    }
}

/// `PanicGoal` — паника от damage-type тегов.
pub struct PanicGoal {
    /// `speedModifier`.
    pub speed_modifier: f64,
    /// `posX/Y/Z`.
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
    /// `isRunning` (Java — отдельный флаг; у WrappedGoal есть
    /// свой isRunning, но PanicGoal держит свой для isRunning()).
    pub is_running: bool,
    /// `panicCausingDamageTypes` — TagKey<DamageType>.
    pub panic_causing_damage_types: TagKey<DamageType>,
}

impl PanicGoal {
    /// `new(mob, speedModifier)` — `DamageTypeTags.PANIC_CAUSES`.
    pub fn new(speed_modifier: f64) -> Self {
        Self {
            speed_modifier,
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
            is_running: false,
            panic_causing_damage_types: damage_type_tags::keys::panic_causes(),
        }
    }

    /// Java `shouldPanic()`.
    pub fn should_panic(&self, mob: &Mob) -> bool {
        mob.living
            .last_damage_source
            .as_ref()
            .map(|s| s.is_tag(&self.panic_causing_damage_types))
            .unwrap_or(false)
    }

    /// Java `findRandomPosition()` — `DefaultRandomPos.getPos(5,4)`.
    fn find_random_position(&mut self, mob: &Mob) -> bool {
        match default_random_pos::get_pos(mob, 5, 4) {
            Some(pos) => {
                self.pos_x = pos.x;
                self.pos_y = pos.y;
                self.pos_z = pos.z;
                true
            }
            None => false,
        }
    }

    /// Java `lookForWater(level, entity, range)`.
    fn look_for_water(&self, mob: &Mob, range: i32) -> Option<BlockPos> {
        let level = mob.level()?;
        let entity_pos = mob.living.entity.block_position();
        // pos должен быть "неполным" (не solid) — упрощённо:
        // ищем ближайшую воду в радиусе через скан.
        let mut best: Option<BlockPos> = None;
        let mut best_d = i64::MAX;
        for dx in -range..=range {
            for dz in -range..=range {
                for dy in -1..=1 {
                    let p = entity_pos.offset(dx, dy, dz);
                    if let Some(fs) = level.get_fluid_state(p) {
                        if fs.is_in_named_tag(&FluidTags::water()) || fs.is_water() {
                            let d = (p.x - entity_pos.x).pow(2) as i64
                                + (p.y - entity_pos.y).pow(2) as i64
                                + (p.z - entity_pos.z).pow(2) as i64;
                            if d < best_d {
                                best_d = d;
                                best = Some(p);
                            }
                        }
                    }
                }
            }
        }
        best
    }
}

impl Goal for PanicGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if !self.should_panic(mob) {
            return false;
        }
        if mob.living.is_on_fire() {
            if let Some(bp) = self.look_for_water(mob, 5) {
                self.pos_x = bp.x as f64 + 0.5;
                self.pos_y = bp.y as f64;
                self.pos_z = bp.z as f64 + 0.5;
                return true;
            }
        }
        self.find_random_position(mob)
    }
    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        mob.navigation.as_ref().map(|n| !n.borrow().is_done()).unwrap_or(false)
    }
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Move])
    }
    fn start(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.navigation.clone() {
            nav.borrow_mut().move_to_pos(mob, self.pos_x, self.pos_y, self.pos_z, self.speed_modifier);
        }
        self.is_running = true;
    }
    fn stop(&mut self, _mob: &mut Mob) {
        self.is_running = false;
    }
}

/// `RandomStrollGoal` — случайные прогулки.
pub struct RandomStrollGoal {
    /// `speedModifier`.
    pub speed_modifier: f64,
    /// `wantedX/Y/Z`.
    pub wanted_x: f64,
    pub wanted_y: f64,
    pub wanted_z: f64,
    /// `interval`.
    pub interval: i32,
    /// `forceTrigger`.
    pub force_trigger: bool,
    /// `checkNoActionTime`.
    pub check_no_action_time: bool,
}

impl RandomStrollGoal {
    /// `new(mob, speedModifier)` — interval=120, checkNoActionTime=true.
    pub fn new(speed_modifier: f64) -> Self {
        Self::with_interval(speed_modifier, 120)
    }
    /// `new(mob, speedModifier, interval)`.
    pub fn with_interval(speed_modifier: f64, interval: i32) -> Self {
        Self {
            speed_modifier,
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
            interval,
            force_trigger: false,
            check_no_action_time: true,
        }
    }
    /// `new(mob, speedModifier, interval, checkNoActionTime)`.
    pub fn full(speed_modifier: f64, interval: i32, check_no_action_time: bool) -> Self {
        let mut g = Self::with_interval(speed_modifier, interval);
        g.check_no_action_time = check_no_action_time;
        g
    }

    /// Java `getPosition()` — переопределяется наследниками
    /// (Swimming/WaterAvoiding/Flying).
    pub fn get_position(&self, mob: &Mob) -> Option<Vec3> {
        default_random_pos::get_pos(mob, 10, 7)
    }

    /// `trigger()`.
    pub fn trigger(&mut self) {
        self.force_trigger = true;
    }
    /// `setInterval`.
    pub fn set_interval(&mut self, new_chance: i32) {
        self.interval = new_chance;
    }
}

impl Goal for RandomStrollGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.has_controlling_passenger() {
            return false;
        }
        if !self.force_trigger {
            if self.check_no_action_time && mob.get_no_action_time() >= 100 {
                return false;
            }
            let delay = crate::util::mth::positive_ceil_div(self.interval, 2);
            if mob.random.borrow_mut().next_int_bound(delay) != 0 {
                return false;
            }
        }
        let Some(position) = self.get_position(mob) else { return false };
        self.wanted_x = position.x;
        self.wanted_y = position.y;
        self.wanted_z = position.z;
        self.force_trigger = false;
        true
    }
    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        !mob.navigation.as_ref().map(|n| n.borrow().is_done()).unwrap_or(true)
            && !mob.has_controlling_passenger()
    }
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Move])
    }
    fn start(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.navigation.clone() {
            nav.borrow_mut().move_to_pos(mob, self.wanted_x, self.wanted_y, self.wanted_z, self.speed_modifier);
        }
    }
    fn stop(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.navigation.clone() {
            nav.borrow_mut().stop(mob);
        }
    }
}

/// `RandomSwimmingGoal` — `RandomStrollGoal` с
/// `BehaviorUtils.getRandomSwimmablePos(10, 7)`.
pub struct RandomSwimmingGoal {
    pub inner: RandomStrollGoal,
}

impl RandomSwimmingGoal {
    /// `new(mob, speedModifier, interval)`.
    pub fn new(speed_modifier: f64, interval: i32) -> Self {
        Self { inner: RandomStrollGoal::with_interval(speed_modifier, interval) }
    }
}

impl Goal for RandomSwimmingGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        // getPosition-переопределение: temporarily swap logic —
        // повторяем canUse RandomStrollGoal с swimmable pos.
        if mob.has_controlling_passenger() {
            return false;
        }
        if !self.inner.force_trigger {
            if self.inner.check_no_action_time && mob.get_no_action_time() >= 100 {
                return false;
            }
            let delay = crate::util::mth::positive_ceil_div(self.inner.interval, 2);
            if mob.random.borrow_mut().next_int_bound(delay) != 0 {
                return false;
            }
        }
        let Some(position) = behavior_utils::get_random_swimmable_pos(mob, 10, 7) else {
            return false;
        };
        self.inner.wanted_x = position.x;
        self.inner.wanted_y = position.y;
        self.inner.wanted_z = position.z;
        self.inner.force_trigger = false;
        true
    }
    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool { self.inner.can_continue_to_use(mob) }
    fn get_flags(&self) -> GoalFlagSet { self.inner.get_flags() }
    fn start(&mut self, mob: &mut Mob) { self.inner.start(mob) }
    fn stop(&mut self, mob: &mut Mob) { self.inner.stop(mob) }
}

/// `WaterAvoidingRandomStrollGoal`.
pub struct WaterAvoidingRandomStrollGoal {
    pub inner: RandomStrollGoal,
    /// `probability`.
    pub probability: f32,
}

impl WaterAvoidingRandomStrollGoal {
    /// `new(mob, speedModifier)` — probability=0.001.
    pub fn new(speed_modifier: f64) -> Self {
        Self::with_probability(speed_modifier, 0.001)
    }
    /// `new(mob, speedModifier, probability)`.
    pub fn with_probability(speed_modifier: f64, probability: f32) -> Self {
        Self { inner: RandomStrollGoal::new(speed_modifier), probability }
    }

    fn get_position_water_avoiding(&self, mob: &Mob) -> Option<Vec3> {
        if mob.living.is_in_water() {
            land_random_pos::get_pos(mob, 15, 7).or_else(|| self.inner.get_position(mob))
        } else if mob.random.borrow_mut().next_float() >= self.probability {
            land_random_pos::get_pos(mob, 10, 7)
        } else {
            self.inner.get_position(mob)
        }
    }
}

impl Goal for WaterAvoidingRandomStrollGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.has_controlling_passenger() {
            return false;
        }
        if !self.inner.force_trigger {
            if self.inner.check_no_action_time && mob.get_no_action_time() >= 100 {
                return false;
            }
            let delay = crate::util::mth::positive_ceil_div(self.inner.interval, 2);
            if mob.random.borrow_mut().next_int_bound(delay) != 0 {
                return false;
            }
        }
        let Some(position) = self.get_position_water_avoiding(mob) else { return false };
        self.inner.wanted_x = position.x;
        self.inner.wanted_y = position.y;
        self.inner.wanted_z = position.z;
        self.inner.force_trigger = false;
        true
    }
    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool { self.inner.can_continue_to_use(mob) }
    fn get_flags(&self) -> GoalFlagSet { self.inner.get_flags() }
    fn start(&mut self, mob: &mut Mob) { self.inner.start(mob) }
    fn stop(&mut self, mob: &mut Mob) { self.inner.stop(mob) }
}

/// `WaterAvoidingRandomFlyingGoal`.
pub struct WaterAvoidingRandomFlyingGoal {
    pub inner: RandomStrollGoal,
}

impl WaterAvoidingRandomFlyingGoal {
    /// `new(mob, speedModifier)`.
    pub fn new(speed_modifier: f64) -> Self {
        Self { inner: RandomStrollGoal::new(speed_modifier) }
    }
}

impl Goal for WaterAvoidingRandomFlyingGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.has_controlling_passenger() {
            return false;
        }
        if !self.inner.force_trigger {
            if self.inner.check_no_action_time && mob.get_no_action_time() >= 100 {
                return false;
            }
            let delay = crate::util::mth::positive_ceil_div(self.inner.interval, 2);
            if mob.random.borrow_mut().next_int_bound(delay) != 0 {
                return false;
            }
        }
        let view = mob.living.entity.get_view_vector(0.0);
        let pos = hover_random_pos::get_pos(mob, 8, 7, view.x, view.z, std::f32::consts::FRAC_PI_2 as f64, 3, 1)
            .or_else(|| air_and_water_random_pos::get_pos(mob, 8, 4, -2, view.x, view.z, std::f32::consts::FRAC_PI_2 as f64));
        let Some(position) = pos else { return false };
        self.inner.wanted_x = position.x;
        self.inner.wanted_y = position.y;
        self.inner.wanted_z = position.z;
        self.inner.force_trigger = false;
        true
    }
    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool { self.inner.can_continue_to_use(mob) }
    fn get_flags(&self) -> GoalFlagSet { self.inner.get_flags() }
    fn start(&mut self, mob: &mut Mob) { self.inner.start(mob) }
    fn stop(&mut self, mob: &mut Mob) { self.inner.stop(mob) }
}

/// `RandomLookAroundGoal`.
pub struct RandomLookAroundGoal {
    pub rel_x: f64,
    pub rel_z: f64,
    pub look_time: i32,
}

impl RandomLookAroundGoal {
    /// `new(mob)`.
    pub fn new() -> Self {
        Self { rel_x: 0.0, rel_z: 0.0, look_time: 0 }
    }
}

impl Default for RandomLookAroundGoal {
    fn default() -> Self { Self::new() }
}

impl Goal for RandomLookAroundGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        mob.random.borrow_mut().next_float() < 0.02
    }
    fn can_continue_to_use(&mut self, _mob: &mut Mob) -> bool {
        self.look_time >= 0
    }
    fn requires_update_every_tick(&self) -> bool { true }
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Move, GoalFlag::Look])
    }
    fn start(&mut self, mob: &mut Mob) {
        let angle = std::f64::consts::TAU * mob.random.borrow_mut().next_double();
        self.rel_x = angle.cos();
        self.rel_z = angle.sin();
        self.look_time = 20 + mob.random.borrow_mut().next_int_bound(20);
    }
    fn tick(&mut self, mob: &mut Mob) {
        self.look_time -= 1;
        let x = mob.get_x() + self.rel_x;
        let y = mob.living.entity.get_eye_y();
        let z = mob.get_z() + self.rel_z;
        let (dyaw, dpitch) = (mob.get_head_rot_speed(), mob.get_max_head_x_rot());
        mob.look_control.set_look_at_deltas(x, y, z, dyaw, dpitch);
    }
}

/// `LookAtPlayerGoal`.
pub struct LookAtPlayerGoal {
    /// `lookAt` — Entity (клон; None = нет цели).
    pub look_at: Option<Entity>,
    /// `lookDistance`.
    pub look_distance: f32,
    /// `lookTime`.
    pub look_time: i32,
    /// `probability`.
    pub probability: f32,
    /// `onlyHorizontal`.
    pub only_horizontal: bool,
    /// `lookAtTypeIsPlayer` — `lookAtType == Player.class`.
    pub look_at_type_is_player: bool,
}

impl LookAtPlayerGoal {
    /// `new(mob, lookAtType(Player), lookDistance)`.
    pub fn new_look_at_player(mob_look_distance: f32) -> Self {
        Self::full(mob_look_distance, 0.02, false, true)
    }
    /// `new(mob, lookAtType, lookDistance)`.
    pub fn new(look_distance: f32) -> Self {
        Self::full(look_distance, 0.02, false, false)
    }
    /// `new(mob, lookAtType, lookDistance, probability)`.
    pub fn with_probability(look_distance: f32, probability: f32) -> Self {
        Self::full(look_distance, probability, false, false)
    }
    /// Полный конструктор.
    pub fn full(look_distance: f32, probability: f32, only_horizontal: bool, is_player: bool) -> Self {
        Self {
            look_at: None,
            look_distance,
            look_time: 0,
            probability,
            only_horizontal,
            look_at_type_is_player: is_player,
        }
    }
}

impl Goal for LookAtPlayerGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.random.borrow_mut().next_float() >= self.probability {
            return false;
        }
        if let Some(target) = mob.get_target().cloned() {
            self.look_at = Some(target.entity.clone());
        }
        let eye = mob.living.entity.get_eye_position();
        let found = if self.look_at_type_is_player {
            mob.level().and_then(|l| l.nearest_player(eye, self.look_distance as f64))
        } else {
            mob.level().and_then(|l| l.nearest_entity(eye, self.look_distance as f64))
        };
        let prev = self.look_at.take();
        self.look_at = found.or(prev);
        self.look_at.is_some()
    }
    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(look_at) = &self.look_at else { return false };
        look_at.is_alive()
            && mob.living.entity.distance_to_sqr(look_at) <= (self.look_distance * self.look_distance) as f64
            && self.look_time > 0
    }
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Look])
    }
    fn start(&mut self, mob: &mut Mob) {
        self.look_time = 40 + mob.random.borrow_mut().next_int_bound(40);
    }
    fn stop(&mut self, _mob: &mut Mob) {
        self.look_at = None;
    }
    fn tick(&mut self, mob: &mut Mob) {
        if let Some(look_at) = &self.look_at {
            if look_at.is_alive() {
                let dy = if self.only_horizontal {
                    mob.living.entity.get_eye_y() as f64
                } else {
                    look_at.get_eye_y() as f64
                };
                let x = look_at.get_x();
                let z = look_at.get_z();
                let (dyaw, dpitch) = (mob.get_head_rot_speed(), mob.get_max_head_x_rot());
                mob.look_control.set_look_at_deltas(x, dy, z, dyaw, dpitch);
                self.look_time -= 1;
            }
        }
    }
}

/// `JumpGoal` — случайные прыжки (frog).
pub struct JumpGoal {
    /// `jumpChance`.
    pub jump_chance: f32,
    /// `interval`.
    pub interval: i32,
}

impl JumpGoal {
    /// `new(mob, jumpChance, interval)`.
    pub fn new(jump_chance: f32, interval: i32) -> Self {
        Self { jump_chance, interval }
    }
}

impl Goal for JumpGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        mob.living.entity.on_ground()
            && mob.random.borrow_mut().next_int_bound(crate::util::mth::positive_ceil_div(self.interval, 2)) == 0
            && mob.random.borrow_mut().next_float() < self.jump_chance
    }
    fn can_continue_to_use(&mut self, _mob: &mut Mob) -> bool { false }
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Jump])
    }
    fn start(&mut self, mob: &mut Mob) {
        mob.jump_control.jump();
    }
    fn stop(&mut self, _mob: &mut Mob) {}
}

/// `MoveTowardsTargetGoal`.
pub struct MoveTowardsTargetGoal {
    /// `speedModifier`.
    pub speed_modifier: f64,
    /// `within`.
    pub within: f32,
    /// `wantedX/Y/Z`.
    pub wanted_x: f64,
    pub wanted_y: f64,
    pub wanted_z: f64,
}

impl MoveTowardsTargetGoal {
    /// `new(mob, speedModifier, within)`.
    pub fn new(speed_modifier: f64, within: f32) -> Self {
        Self { speed_modifier, within, wanted_x: 0.0, wanted_y: 0.0, wanted_z: 0.0 }
    }
}

impl Goal for MoveTowardsTargetGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let Some(target) = mob.get_target().cloned() else { return false };
        if target.entity.distance_to_sqr(&mob.living.entity) > (self.within * self.within) as f64 {
            return false;
        }
        let Some(pos) = default_random_pos::get_pos_towards(
            mob,
            16,
            7,
            target.entity.position(),
            std::f32::consts::FRAC_PI_2 as f64,
        ) else {
            return false;
        };
        self.wanted_x = pos.x;
        self.wanted_y = pos.y;
        self.wanted_z = pos.z;
        true
    }
    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        mob.navigation.as_ref().map(|n| !n.borrow().is_done()).unwrap_or(false)
    }
    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Move])
    }
    fn start(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.navigation.clone() {
            nav.borrow_mut().move_to_pos(mob, self.wanted_x, self.wanted_y, self.wanted_z, self.speed_modifier);
        }
    }
    fn stop(&mut self, mob: &mut Mob) {
        if let Some(nav) = mob.navigation.clone() {
            nav.borrow_mut().stop(mob);
        }
    }
}
