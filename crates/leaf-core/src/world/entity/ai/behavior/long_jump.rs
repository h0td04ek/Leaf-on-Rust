//! Перенос long-jump поведений: `LongJumpUtil`,
//! `LongJumpToRandomPos`, `LongJumpToPreferredBlock`,
//! `LongJumpMidJump`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::entity::ai::attributes::attributes as attrs;
use crate::resources::identifier::Identifier;
use crate::sounds::sound_event::SoundEvent;
use crate::util::random::weighted_random::WeightedRandom;
use crate::util::valueproviders::int_provider::IntProvider;
use crate::util::valueproviders::uniform_int::UniformInt;
use crate::world::entity::ai::behavior::behavior::{BehaviorBase, BehaviorControl, BehaviorStatus};
use crate::world::entity::ai::memory::memory::{
    keys, BlockPosTracker, MemoryModuleType, MemoryStatus, PositionTracker,
};
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::{Mob, MobPathing};
use crate::world::entity::pose::Pose;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;

// ===================== LongJumpUtil =====================

/// `LongJumpUtil.calculateJumpVectorForAngle` — баллистика
/// прыжка под углом `angle` градусов к цели `target`.
/// `max_jump_velocity` — макс. скорость (jumpStrength *
/// maxJumpVelocityMultiplier). `require_clear_transition` —
/// проверка свободного пролёта через `isClearTransition`.
pub fn calculate_jump_vector_for_angle(
    level: &Arc<dyn LevelReader>,
    mob: &Mob,
    target: Vec3,
    max_jump_velocity: f32,
    angle: i32,
    require_clear_transition: bool,
) -> Option<Vec3> {
    let vec3 = mob.living.entity.position();
    let vec31 = Vec3::new(target.x - vec3.x, 0.0, target.z - vec3.z)
        .normalize()
        .scale(0.5);
    let vec32 = target.subtract_vec(&vec31);
    let vec33 = vec32.subtract_vec(&vec3);
    let f = angle as f32 * std::f32::consts::PI / 180.0;
    let atan2 = vec33.z.atan2(vec33.x);
    let d = vec33.subtract(0.0, vec33.y, 0.0).length_sqr();
    let square_root = d.sqrt();
    let d1 = vec33.y;
    let gravity = mob.living.get_gravity();
    let sin = (2.0 * f as f64).sin();
    let d2 = (f as f64).cos().powi(2);
    let sin1 = (f as f64).sin();
    let cos = (f as f64).cos();
    let sin2 = atan2.sin();
    let cos1 = atan2.cos();
    let d3 = d * gravity / (square_root * sin - 2.0 * d1 * d2);
    if d3 < 0.0 {
        return None;
    }
    let square_root1 = d3.sqrt();
    if square_root1 > max_jump_velocity as f64 {
        return None;
    }
    let d4 = square_root1 * cos;
    let d5 = square_root1 * sin1;
    if require_clear_transition {
        let i = (square_root / d4).ceil() as i32 * 2;
        let mut d6 = 0.0;
        let mut vec34: Option<Vec3> = None;
        let dimensions = mob.living.entity.get_dimensions_pose(Pose::LongJumping);
        for i1 in 0..(i - 1) {
            d6 += square_root / i as f64;
            let d7 = sin1 / cos * d6 - d6.powi(2) * gravity
                / (2.0 * d3 * cos.powi(2));
            let d8 = d6 * cos1;
            let d9 = d6 * sin2;
            let vec35 = Vec3::new(vec3.x + d8, vec3.y + d7, vec3.z + d9);
            if let Some(prev) = vec34 {
                if !is_clear_transition(level, mob, dimensions, prev, vec35) {
                    return None;
                }
            }
            vec34 = Some(vec35);
            let _ = i1;
        }
    }
    Some(Vec3::new(d4 * cos1, d5, d4 * sin2).scale(0.95))
}

/// `LongJumpUtil.isClearTransition` — все промежуточные
/// bounding-box'ы свободны от коллизий.
fn is_clear_transition(
    level: &Arc<dyn LevelReader>,
    mob: &Mob,
    dimensions: crate::world::entity::entity_type::EntityDimensions,
    start_pos: Vec3,
    end_pos: Vec3,
) -> bool {
    let vec3 = end_pos.subtract_vec(&start_pos);
    let d = dimensions.width.min(dimensions.height) as f64;
    let ceil = (vec3.length() / d).ceil() as i32;
    let vec31 = vec3.normalize();
    let mut vec32 = start_pos;
    for i in 0..ceil {
        vec32 = if i == ceil - 1 {
            end_pos
        } else {
            vec32.add_vec(&vec31.scale(d * 0.9))
        };
        let bb = dimensions.make_bounding_box(&vec32);
        if !level.no_collision(&bb) {
            return false;
        }
    }
    true
}

// ===================== LongJumpToRandomPos =====================

/// `PossibleJump(targetPos, weight)`.
#[derive(Clone, Debug)]
pub struct PossibleJump {
    pub target_pos: BlockPos,
    pub weight: i32,
}

const FIND_JUMP_TRIES: i32 = 20;
const PREPARE_JUMP_DURATION: i64 = 40;
const TIME_OUT_DURATION: i32 = 200;
const ALLOWED_ANGLES: [i32; 4] = [65, 70, 75, 80];

/// `LongJumpToRandomPos` — поиск точки прыжка в радиусе,
/// баллистика, подготовка 40 тиков, прыжок.
pub struct LongJumpToRandomPos {
    pub base: BehaviorBase,
    pub time_between_long_jumps: UniformInt,
    pub max_long_jump_height: i32,
    pub max_long_jump_width: i32,
    pub max_jump_velocity_multiplier: f32,
    pub jump_candidates: Vec<PossibleJump>,
    pub initial_position: Option<Vec3>,
    pub chosen_jump: Option<Vec3>,
    pub find_jump_tries: i32,
    pub prepare_jump_start: i64,
    pub get_jump_sound: Rc<dyn Fn(&Mob) -> Holder<SoundEvent>>,
    pub acceptable_landing_spot: Rc<dyn Fn(&Mob, BlockPos) -> bool>,
}

impl LongJumpToRandomPos {
    /// `defaultAcceptableLandingSpot` — твёрдый блок снизу +
    /// нулевой pathfinding-malus.
    pub fn default_acceptable_landing_spot(level: &Arc<dyn LevelReader>, mob: &Mob, pos: BlockPos) -> bool {
        let block_pos = pos.below();
        let solid = level
            .get_block_state(block_pos)
            .map(|s| s.is_solid_render(level.as_ref(), block_pos))
            .unwrap_or(false);
        let pt = crate::world::entity::ai::util::goal_utils::walk_path_type_static(mob, pos);
        solid && mob.get_pathfinding_malus(pt) == 0.0
    }

    /// `new(timeBetweenLongJumps, maxLongJumpHeight,
    /// maxLongJumpWidth, maxJumpVelocity, getJumpSound)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time_between_long_jumps: UniformInt,
        max_long_jump_height: i32,
        max_long_jump_width: i32,
        max_jump_velocity_multiplier: f32,
        get_jump_sound: Rc<dyn Fn(&Mob) -> Holder<SoundEvent>>,
    ) -> Self {
        Self::with_landing_spot(
            time_between_long_jumps,
            max_long_jump_height,
            max_long_jump_width,
            max_jump_velocity_multiplier,
            get_jump_sound,
            Rc::new(|_mob, _pos| true),
        )
    }

    /// `new(..., acceptableLandingSpot)`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_landing_spot(
        time_between_long_jumps: UniformInt,
        max_long_jump_height: i32,
        max_long_jump_width: i32,
        max_jump_velocity_multiplier: f32,
        get_jump_sound: Rc<dyn Fn(&Mob) -> Holder<SoundEvent>>,
        acceptable_landing_spot: Rc<dyn Fn(&Mob, BlockPos) -> bool>,
    ) -> Self {
        Self {
            base: BehaviorBase::with_duration(
                BehaviorBase::conditions(&[
                    (keys::look_target(), MemoryStatus::Registered),
                    (keys::long_jump_cooldown_ticks(), MemoryStatus::ValueAbsent),
                    (keys::long_jump_mid_jump(), MemoryStatus::ValueAbsent),
                ]),
                TIME_OUT_DURATION,
            ),
            time_between_long_jumps,
            max_long_jump_height,
            max_long_jump_width,
            max_jump_velocity_multiplier,
            jump_candidates: Vec::new(),
            initial_position: None,
            chosen_jump: None,
            find_jump_tries: 0,
            prepare_jump_start: 0,
            get_jump_sound,
            acceptable_landing_spot,
        }
    }

    /// `getJumpCandidate` — weighted random из кандидатов
    /// с удалением.
    pub fn get_jump_candidate(
        &mut self,
        level: &Arc<dyn LevelReader>,
        mob: &Mob,
    ) -> Option<PossibleJump> {
        let total = WeightedRandom::get_total_weight(&self.jump_candidates, |c| c.weight);
        let mut r = mob.random.borrow_mut();
        let picked = WeightedRandom::get_random_item(
            &mut **r,
            &self.jump_candidates,
            total,
            |c| c.weight,
        )
        .cloned();
        let _ = level;
        if let Some(ref p) = picked {
            if let Some(idx) = self.jump_candidates.iter().position(|c| c.target_pos == p.target_pos) {
                self.jump_candidates.remove(idx);
            }
        }
        picked
    }

    /// `isAcceptableLandingPosition` — не текущая колонна +
    /// acceptableLandingSpot.
    pub(crate) fn is_acceptable_landing_position_pub(
        &self,
        _level: &Arc<dyn LevelReader>,
        mob: &Mob,
        pos: BlockPos,
    ) -> bool {
        let block_pos = mob.block_position();
        (block_pos.x != pos.x || block_pos.z != pos.z)
            && (self.acceptable_landing_spot)(mob, pos)
    }

    /// `pickCandidate` — выбирает прыжок: acceptable pos +
    /// optimal vector + нет достижимого пути.
    pub fn pick_candidate(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        while !self.jump_candidates.is_empty() {
            let Some(candidate) = self.get_jump_candidate(level, mob) else { break };
            let block_pos = candidate.target_pos;
            if self.is_acceptable_landing_position_pub(level, mob, block_pos) {
                let vec3 = crate::world::phys::vec3::at_center_of(&block_pos.to_vec3i());
                let vec31 = self.calculate_optimal_jump_vector(level, mob, vec3);
                if let Some(vec31) = vec31 {
                    mob.living
                        .brain_set_value(&keys::look_target(), Rc::new(BlockPosTracker::new(block_pos)) as Rc<dyn PositionTracker>);
                    let path = mob
                        .navigation
                        .as_ref()
                        .and_then(|n| n.borrow_mut().create_path_pos(mob, block_pos, 8));
                    if path.map(|p| !p.can_reach()).unwrap_or(true) {
                        self.chosen_jump = Some(vec31);
                        self.prepare_jump_start = game_time;
                        return;
                    }
                }
            }
        }
    }

    /// `calculateOptimalJumpVector` — перебор углов
    /// [65,70,75,80] в случайном порядке.
    pub fn calculate_optimal_jump_vector(
        &self,
        level: &Arc<dyn LevelReader>,
        mob: &Mob,
        target: Vec3,
    ) -> Option<Vec3> {
        let mut list = ALLOWED_ANGLES.to_vec();
        // Fisher-Yates как Collections.shuffle.
        let mut r = mob.random.borrow_mut();
        for i in (1..list.len()).rev() {
            let j = r.next_int_bound((i + 1) as i32) as usize;
            list.swap(i, j);
        }
        let f = (mob
            .living
            .get_attribute_value_by_id_ro(&attrs::jump_strength())
            * self.max_jump_velocity_multiplier as f64) as f32;
        for i in list {
            if let Some(v) = calculate_jump_vector_for_angle(
                level, mob, target, f, i, true,
            ) {
                return Some(v);
            }
        }
        None
    }
}

impl BehaviorControl for LongJumpToRandomPos {
    fn status(&self) -> BehaviorStatus { self.base.status }
    fn set_status(&mut self, status: BehaviorStatus) { self.base.status = status; }
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
        &self.base.entry_condition
    }
    fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
    fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
    fn set_end_timestamp(&mut self, timestamp: i64) { self.base.end_timestamp = timestamp; }

    fn check_extra_start_conditions(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
        let honey = Identifier::with_default_namespace("honey_block").unwrap();
        let flag = mob.living.entity.is_on_ground()
            && !mob.living.entity.is_in_water()
            && !mob.living.entity.is_in_lava()
            && !level
                .get_block_state(mob.block_position())
                .map(|s| s.block.id == honey)
                .unwrap_or(false);
        if !flag {
            let mut r = mob.random.borrow_mut();
            let sample = IntProvider::sample(&self.time_between_long_jumps, &mut **r);
            mob.living.brain_set_value(&keys::long_jump_cooldown_ticks(), sample / 2);
        }
        flag
    }

    fn can_still_use(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
        let pos = mob.living.entity.position();
        let flag = self.initial_position.map(|p| p == pos).unwrap_or(false)
            && self.find_jump_tries > 0
            && !mob.living.entity.is_in_water()
            && (self.chosen_jump.is_some() || !self.jump_candidates.is_empty());
        if !flag && !mob.living.brain_has(&keys::long_jump_mid_jump()) {
            let mut r = mob.random.borrow_mut();
            let sample = IntProvider::sample(&self.time_between_long_jumps, &mut **r);
            mob.living.brain_set_value(&keys::long_jump_cooldown_ticks(), sample / 2);
            mob.living.brain_erase(&keys::look_target());
        }
        let _ = level;
        flag
    }

    fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        self.chosen_jump = None;
        self.find_jump_tries = FIND_JUMP_TRIES;
        self.initial_position = Some(mob.living.entity.position());
        let block_pos = mob.block_position();
        let min = BlockPos::new(
            block_pos.x - self.max_long_jump_width,
            block_pos.y - self.max_long_jump_height,
            block_pos.z - self.max_long_jump_width,
        );
        let max = BlockPos::new(
            block_pos.x + self.max_long_jump_width,
            block_pos.y + self.max_long_jump_height,
            block_pos.z + self.max_long_jump_width,
        );
        self.jump_candidates = BlockPos::between_closed_coords(
            min.x, min.y, min.z, max.x, max.y, max.z,
        )
        .filter(|bp| *bp != block_pos)
        .map(|bp| PossibleJump {
            target_pos: bp,
            weight: (block_pos.dist_sqr(bp)).ceil() as i32,
        })
        .collect();
    }

    fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        if let Some(chosen) = self.chosen_jump {
            if game_time - self.prepare_jump_start >= PREPARE_JUMP_DURATION {
                mob.living.entity.set_y_rot(mob.living.y_body_rot);
                mob.living.entity.set_discard_friction(true);
                let len = chosen.length();
                let d = len + mob.living.get_jump_boost_power() as f64;
                mob.living.entity.set_delta_movement(
                    chosen.x * d / len,
                    chosen.y * d / len,
                    chosen.z * d / len,
                );
                mob.living.brain_set_value(&keys::long_jump_mid_jump(), true);
                let sound = (self.get_jump_sound)(mob);
                mob.living.entity.play_sound(Some(&sound), 1.0, 1.0);
            }
        } else {
            self.find_jump_tries -= 1;
            self.pick_candidate(level, mob, game_time);
        }
    }

    fn debug_string(&self) -> String { "LongJumpToRandomPos".to_string() }
}

// ===================== LongJumpToPreferredBlock =====================

/// `LongJumpToPreferredBlock` — как LongJumpToRandomPos, но с
/// шансом `preferred_blocks_chance` предпочитает блоки из
/// `preferred_block_tag` (проверка блока под точкой приземления).
pub struct LongJumpToPreferredBlock {
    pub inner: LongJumpToRandomPos,
    pub preferred_block_tag: Identifier,
    pub preferred_blocks_chance: f32,
    pub not_preferred_jump_candidates: Vec<PossibleJump>,
    pub currently_wanting_preferred_ones: bool,
}

impl LongJumpToPreferredBlock {
    #[allow(clippy::too_many_arguments)]
    /// `new(..., preferredBlockTag, preferredBlocksChance,
    /// acceptableLandingSpot)`.
    pub fn new(
        time_between_long_jumps: UniformInt,
        max_long_jump_height: i32,
        max_long_jump_width: i32,
        max_jump_velocity: f32,
        get_jump_sound: Rc<dyn Fn(&Mob) -> Holder<SoundEvent>>,
        preferred_block_tag: Identifier,
        preferred_blocks_chance: f32,
        acceptable_landing_spot: Rc<dyn Fn(&Mob, BlockPos) -> bool>,
    ) -> Self {
        Self {
            inner: LongJumpToRandomPos::with_landing_spot(
                time_between_long_jumps,
                max_long_jump_height,
                max_long_jump_width,
                max_jump_velocity,
                get_jump_sound,
                acceptable_landing_spot,
            ),
            preferred_block_tag,
            preferred_blocks_chance,
            not_preferred_jump_candidates: Vec::new(),
            currently_wanting_preferred_ones: false,
        }
    }

    /// `getJumpCandidate` — при желании preferred: пропускает
    /// кандидатов не на preferred-блоке в backup-список.
    pub fn get_jump_candidate(
        &mut self,
        level: &Arc<dyn LevelReader>,
        mob: &Mob,
    ) -> Option<PossibleJump> {
        if !self.currently_wanting_preferred_ones {
            return self.inner.get_jump_candidate(level, mob);
        }
        while !self.inner.jump_candidates.is_empty() {
            if let Some(pj) = self.inner.get_jump_candidate(level, mob) {
                let below = pj.target_pos.below();
                let is_preferred = level
                    .get_block_state(below)
                    .map(|s| s.is_block_tag(&self.preferred_block_tag))
                    .unwrap_or(false);
                if is_preferred {
                    return Some(pj);
                }
                self.not_preferred_jump_candidates.push(pj);
            }
        }
        if !self.not_preferred_jump_candidates.is_empty() {
            return Some(self.not_preferred_jump_candidates.remove(0));
        }
        None
    }
}

impl BehaviorControl for LongJumpToPreferredBlock {
    fn status(&self) -> BehaviorStatus { self.inner.status() }
    fn set_status(&mut self, status: BehaviorStatus) { self.inner.set_status(status); }
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
        self.inner.entry_conditions()
    }
    fn durations(&self) -> (i32, i32) { self.inner.durations() }
    fn end_timestamp(&self) -> i64 { self.inner.end_timestamp() }
    fn set_end_timestamp(&mut self, timestamp: i64) { self.inner.set_end_timestamp(timestamp); }

    fn check_extra_start_conditions(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
        self.inner.check_extra_start_conditions(level, mob)
    }
    fn can_still_use(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) -> bool {
        self.inner.can_still_use(level, mob, game_time)
    }
    fn start(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        self.inner.start(level, mob, game_time);
        self.not_preferred_jump_candidates.clear();
        let mut r = mob.random.borrow_mut();
        self.currently_wanting_preferred_ones =
            r.next_float() < self.preferred_blocks_chance;
    }
    fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        // tick как у LongJumpToRandomPos, но pickCandidate использует
        // наш getJumpCandidate — дублируем логику с подменой.
        if self.inner.chosen_jump.is_some() {
            if game_time - self.inner.prepare_jump_start >= PREPARE_JUMP_DURATION {
                mob.living.entity.set_y_rot(mob.living.y_body_rot);
                mob.living.entity.set_discard_friction(true);
                let chosen = self.inner.chosen_jump.unwrap();
                let len = chosen.length();
                let d = len + mob.living.get_jump_boost_power() as f64;
                mob.living.entity.set_delta_movement(
                    chosen.x * d / len,
                    chosen.y * d / len,
                    chosen.z * d / len,
                );
                mob.living.brain_set_value(&keys::long_jump_mid_jump(), true);
                let sound = (self.inner.get_jump_sound)(mob);
                mob.living.entity.play_sound(Some(&sound), 1.0, 1.0);
            }
        } else {
            self.inner.find_jump_tries -= 1;
            self.pick_candidate_preferred(level, mob, game_time);
        }
    }
    fn debug_string(&self) -> String { "LongJumpToPreferredBlock".to_string() }
}

impl LongJumpToPreferredBlock {
    fn pick_candidate_preferred(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
        while !self.inner.jump_candidates.is_empty() {
            let Some(candidate) = self.get_jump_candidate(level, mob) else { break };
            let block_pos = candidate.target_pos;
            if self.inner.is_acceptable_landing_position_pub(level, mob, block_pos) {
                let vec3 = crate::world::phys::vec3::at_center_of(&block_pos.to_vec3i());
                let vec31 = self.inner.calculate_optimal_jump_vector(level, mob, vec3);
                if let Some(vec31) = vec31 {
                    mob.living
                        .brain_set_value(&keys::look_target(), Rc::new(BlockPosTracker::new(block_pos)) as Rc<dyn PositionTracker>);
                    let path = mob
                        .navigation
                        .as_ref()
                        .and_then(|n| n.borrow_mut().create_path_pos(mob, block_pos, 8));
                    if path.map(|p| !p.can_reach()).unwrap_or(true) {
                        self.inner.chosen_jump = Some(vec31);
                        self.inner.prepare_jump_start = game_time;
                        return;
                    }
                }
            }
        }
    }
}

// ===================== LongJumpMidJump =====================

/// `LongJumpMidJump` — фаза полёта: LongJumping pose,
/// discardFriction, по приземлении damping+звук+cooldown.
pub struct LongJumpMidJump {
    pub base: BehaviorBase,
    pub time_between_long_jumps: UniformInt,
    pub landing_sound: Holder<SoundEvent>,
}

impl LongJumpMidJump {
    /// `new(timeBetweenLongJumps, landingSound)`.
    pub fn new(time_between_long_jumps: UniformInt, landing_sound: Holder<SoundEvent>) -> Self {
        Self {
            base: BehaviorBase::with_duration(
                BehaviorBase::conditions(&[
                    (keys::look_target(), MemoryStatus::Registered),
                    (keys::long_jump_mid_jump(), MemoryStatus::ValuePresent),
                ]),
                100,
            ),
            time_between_long_jumps,
            landing_sound,
        }
    }
}

impl BehaviorControl for LongJumpMidJump {
    fn status(&self) -> BehaviorStatus { self.base.status }
    fn set_status(&mut self, status: BehaviorStatus) { self.base.status = status; }
    fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
        &self.base.entry_condition
    }
    fn durations(&self) -> (i32, i32) { (self.base.min_duration, self.base.max_duration) }
    fn end_timestamp(&self) -> i64 { self.base.end_timestamp }
    fn set_end_timestamp(&mut self, timestamp: i64) { self.base.end_timestamp = timestamp; }

    fn can_still_use(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) -> bool {
        !mob.living.entity.is_on_ground()
    }

    fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        mob.living.entity.set_discard_friction(true);
        mob.living.entity.set_pose(Pose::LongJumping);
    }

    fn stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
        if mob.living.entity.is_on_ground() {
            let d = mob.living.entity.get_delta_movement().multiply(0.1, 1.0, 0.1);
            mob.living.entity.set_delta_movement(d.x, d.y, d.z);
            mob.living.entity.play_sound(Some(&self.landing_sound), 2.0, 1.0);
            let _ = level;
        }
        mob.living.entity.set_discard_friction(false);
        mob.living.entity.set_pose(Pose::Standing);
        mob.living.brain_erase(&keys::long_jump_mid_jump());
        let mut r = mob.random.borrow_mut();
        let sample = IntProvider::sample(&self.time_between_long_jumps, &mut **r);
        mob.living.brain_set_value(&keys::long_jump_cooldown_ticks(), sample);
    }

    fn debug_string(&self) -> String { "LongJumpMidJump".to_string() }
}
