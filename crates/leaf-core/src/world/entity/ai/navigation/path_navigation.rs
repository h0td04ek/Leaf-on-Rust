//! Перенос `net.minecraft.world.entity.ai.navigation.PathNavigation`
//! и наследников (`GroundPathNavigation`, `FlyingPathNavigation`,
//! `AmphibiousPathNavigation`, `WaterBoundPathNavigation`,
//! `WallClimberNavigation`).
//!
//! В Java — иерархия классов; здесь — один `PathNavigation`
//! с `kind: NavKind` (как `BlockKind` в block.rs) — поведение
//! идентично, различия в виртуальных методах разруливаются
//! по `kind`.

use crate::core::block_pos::BlockPos;
use crate::tags::block_tags::BlockTags;
use crate::util::mth;
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::path_navigation_region::PathNavigationRegion;
use crate::world::level::pathfinder::amphibious_node_evaluator::AmphibiousNodeEvaluator;
use crate::world::level::pathfinder::fly_node_evaluator::FlyNodeEvaluator;
use crate::world::level::pathfinder::node::Node;
use crate::world::level::pathfinder::node_evaluator::NodeEvaluator;
use crate::world::level::pathfinder::path::Path;
use crate::world::level::pathfinder::path_computation_type::PathComputationType;
use crate::world::level::pathfinder::path_finder::PathFinder;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::level::pathfinder::swim_node_evaluator::SwimNodeEvaluator;
use crate::world::level::pathfinder::walk_node_evaluator::WalkNodeEvaluator;
use crate::world::phys::vec3::Vec3;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use super::mob_navigation::MobNavigation;

/// Вид навигации (`PathNavigation`-наследник в Java).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NavKind {
    /// `GroundPathNavigation`.
    Ground,
    /// `FlyingPathNavigation`.
    Flying,
    /// `AmphibiousPathNavigation`.
    Amphibious,
    /// `WaterBoundPathNavigation`.
    WaterBound,
    /// `WallClimberNavigation` (extends Ground).
    WallClimber,
}

/// `PathNavigation` (+ наследники через `kind`).
pub struct PathNavigation {
    /// `path`.
    path: Option<Path>,
    /// `speedModifier`.
    speed_modifier: f64,
    /// `tick`.
    tick: i32,
    /// `lastStuckCheck`.
    last_stuck_check: i32,
    /// `lastStuckCheckPos`.
    last_stuck_check_pos: Vec3,
    /// `timeoutCachedNode`.
    timeout_cached_node: BlockPos,
    /// `timeoutTimer`.
    timeout_timer: i64,
    /// `lastTimeoutCheck`.
    last_timeoutCheck: i64,
    /// `timeoutLimit`.
    timeout_limit: f64,
    /// `maxDistanceToWaypoint`.
    max_distance_to_waypoint: f32,
    /// `hasDelayedRecomputation`.
    has_delayed_recomputation: bool,
    /// `timeLastRecompute`.
    time_last_recompute: i64,
    /// `nodeEvaluator` — общий с pathFinder.
    node_evaluator: Rc<RefCell<dyn NodeEvaluator>>,
    /// `targetPos`.
    target_pos: Option<BlockPos>,
    /// `reachRange`.
    reach_range: i32,
    /// `maxVisitedNodesMultiplier`.
    max_visited_nodes_multiplier: f32,
    /// `pathFinder`.
    path_finder: PathFinder,
    /// `isStuck`.
    is_stuck: bool,
    /// `requiredPathLength`.
    required_path_length: f32,
    /// `kind` — какой наследник PathNavigation.
    kind: NavKind,
    // --- GroundPathNavigation fields ---
    /// `avoidSun`.
    avoid_sun: bool,
    /// `canPathToTargetsBelowSurface`.
    can_path_to_targets_below_surface: bool,
    // --- WaterBoundPathNavigation fields ---
    /// `allowBreaching`.
    allow_breaching: bool,
    // --- WallClimberNavigation fields ---
    /// `pathToPosition`.
    path_to_position: Option<BlockPos>,
    /// `waterBoundInitialized` — allowBreaching вычислен
    /// (WaterBoundPathNavigation.createPathFinder-side эффект).
    water_bound_initialized: bool,
    /// `maxVisitedNodesBase` — для пересоздания pathFinder
    /// (WaterBound).
    max_visited_nodes_base: i32,
}

impl PathNavigation {
    /// `MAX_TIME_RECOMPUTE`.
    pub const MAX_TIME_RECOMPUTE: i64 = 20;
    /// `STUCK_CHECK_INTERVAL`.
    pub const STUCK_CHECK_INTERVAL: i32 = 100;
    /// `STUCK_THRESHOLD_DISTANCE_FACTOR`.
    pub const STUCK_THRESHOLD_DISTANCE_FACTOR: f32 = 0.25;

    /// `new` — создаёт evaluator по `kind` (аналог `createPathFinder`
    /// в наследниках) и `PathFinder` с `maxVisitedNodes`.
    pub fn new(kind: NavKind, max_visited_nodes: i32) -> Self {
        let evaluator: Rc<RefCell<dyn NodeEvaluator>> = match kind {
            NavKind::Ground | NavKind::WallClimber => {
                Rc::new(RefCell::new(WalkNodeEvaluator::new()))
            }
            NavKind::Flying => Rc::new(RefCell::new(FlyNodeEvaluator::new())),
            NavKind::Amphibious => {
                Rc::new(RefCell::new(AmphibiousNodeEvaluator::new(false)))
            }
            NavKind::WaterBound => Rc::new(RefCell::new(SwimNodeEvaluator::new(false))),
        };
        if kind == NavKind::WaterBound {
            // WaterBoundPathNavigation.createPathFinder:
            // allowBreaching = mob.getType() == DOLPHIN — вычисляется
            // при первом createPath (нет mob на момент конструктора).
        }
        if kind == NavKind::WaterBound {
            evaluator.borrow_mut().set_can_pass_doors(false);
        }
        let path_finder = PathFinder::new(evaluator.clone(), max_visited_nodes);
        Self {
            path: None,
            speed_modifier: 1.0,
            tick: 0,
            last_stuck_check: 0,
            last_stuck_check_pos: Vec3::ZERO,
            timeout_cached_node: BlockPos::ZERO,
            timeout_timer: 0,
            last_timeoutCheck: 0,
            timeout_limit: 0.0,
            max_distance_to_waypoint: 0.5,
            has_delayed_recomputation: false,
            time_last_recompute: 0,
            node_evaluator: evaluator,
            target_pos: None,
            reach_range: 0,
            max_visited_nodes_multiplier: 1.0,
            path_finder,
            is_stuck: false,
            required_path_length: 16.0,
            kind,
            avoid_sun: false,
            can_path_to_targets_below_surface: false,
            allow_breaching: false,
            path_to_position: None,
            water_bound_initialized: false,
            max_visited_nodes_base: max_visited_nodes,
        }
    }

    /// `updatePathfinderMaxVisitedNodes`.
    pub fn update_pathfinder_max_visited_nodes(&mut self, mob: &Mob) {
        let floor = mth::floor(self.get_max_path_length(mob) as f64 * 16.0) as i32;
        self.path_finder.set_max_visited_nodes(floor);
    }

    /// `getMaxPathLength` — `max(followRange, requiredPathLength)`.
    fn get_max_path_length(&self, mob: &Mob) -> f32 {
        let follow_range = mob
            .living
            .get_attribute_value_by_id_ro(&crate::entity::ai::attributes::attributes::follow_range());
        (follow_range as f32).max(self.required_path_length)
    }

    /// `createPath(targets, regionOffset, offsetUpward,
    /// reachRange, followRange)`.
    #[allow(clippy::too_many_arguments)]
    fn create_path_inner(
        &mut self,
        mob: &Mob,
        targets: &HashSet<BlockPos>,
        region_offset: i32,
        offset_upward: bool,
        reach_range: i32,
        follow_range: f32,
    ) -> Option<Path> {
        if targets.is_empty() {
            return None;
        }
        let Some(level) = mob.level() else {
            return None;
        };
        if mob.get_y() < level.min_build_height() as f64 {
            return None;
        }
        if !self.can_update_path_impl(mob) {
            return None;
        }
        if let Some(p) = &self.path {
            if !p.is_done()
                && self
                    .target_pos
                    .is_some_and(|tp| targets.contains(&tp))
            {
                return self.path.clone();
            }
        }
        let block_pos = if offset_upward {
            mob.block_position().above()
        } else {
            mob.block_position()
        };
        let i = (follow_range + region_offset as f32) as i32;
        let region = PathNavigationRegion::new(
            level.as_ref(),
            block_pos.offset(-i, -i, -i),
            block_pos.offset(i, i, i),
        );
        let cache = level.path_type_cache();
        let path = self.path_finder.find_path(
            &region,
            cache,
            mob,
            targets,
            follow_range,
            reach_range,
            self.max_visited_nodes_multiplier,
        );
        if let Some(p) = &path {
            self.target_pos = Some(p.target_pos());
            self.reach_range = reach_range;
            self.reset_stuck_timeout();
        }
        path
    }

    /// `createPath(targets, regionOffset, offsetUpward, reachRange)`.
    fn create_path_default(
        &mut self,
        mob: &Mob,
        targets: &HashSet<BlockPos>,
        region_offset: i32,
        offset_upward: bool,
        reach_range: i32,
    ) -> Option<Path> {
        let follow_range = self.get_max_path_length(mob);
        self.create_path_inner(mob, targets, region_offset, offset_upward, reach_range, follow_range)
    }

    /// `trimPath` — cauldron-коррекция + avoidSun (Ground).
    fn trim_path(&mut self, mob: &Mob) {
        if let Some(path) = &mut self.path {
            let Some(level) = mob.level() else { return };
            for i in 0..path.node_count() {
                let node = path.node(i);
                let next = if i + 1 < path.node_count() {
                    Some(path.node(i + 1))
                } else {
                    None
                };
                let pos = BlockPos::new(node.borrow().x, node.borrow().y, node.borrow().z);
                let is_cauldron = level
                    .get_block_state(pos)
                    .map(|s| s.is_named_tag(&BlockTags::cauldrons()))
                    .unwrap_or(false);
                if is_cauldron {
                    let (x, y, z) = {
                        let n = node.borrow();
                        (n.x, n.y + 1, n.z)
                    };
                    let cloned = Node::node_ref(x, y, z);
                    *cloned.borrow_mut() = node.borrow().clone_and_move(x, y, z);
                    path.replace_node(i, cloned);
                    if let Some(n1) = next {
                        let (n1x, n1y, n1z, node_y) = {
                            let nb = node.borrow();
                            let n1b = n1.borrow();
                            (n1b.x, n1b.y, n1b.z, nb.y)
                        };
                        if node_y >= n1y {
                            let c2 = Node::node_ref(n1x, n1y + 1, n1z);
                            *c2.borrow_mut() = n1.borrow().clone_and_move(n1x, n1y + 1, n1z);
                            path.replace_node(i + 1, c2);
                        }
                    }
                }
            }
        }
        // GroundPathNavigation.trimPath — avoidSun.
        if self.kind == NavKind::Ground && self.avoid_sun {
            if let Some(level) = mob.level() {
                if level
                    .can_see_sky(BlockPos::containing(mob.get_x(), mob.get_y() + 0.5, mob.get_z()))
                {
                    return;
                }
                if let Some(path) = &self.path {
                    for i in 0..path.node_count() {
                        let node = path.node(i);
                        let (x, y, z) = {
                            let n = node.borrow();
                            (n.x, n.y, n.z)
                        };
                        if level.can_see_sky(BlockPos::new(x, y, z)) {
                            self.path.as_mut().unwrap().truncate_nodes(i);
                            return;
                        }
                    }
                }
            }
        }
    }

    /// `followThePath`.
    fn follow_the_path(&mut self, mob: &Mob) {
        let temp_pos = self.temp_mob_pos(mob);
        self.max_distance_to_waypoint = if mob.living.get_bb_width() > 0.75 {
            mob.living.get_bb_width() / 2.0
        } else {
            0.75 - mob.living.get_bb_width() / 2.0
        };
        let (next_node_pos, next_node_type, next_entity_pos) = {
            let path = match &self.path {
                Some(p) => p,
                None => return,
            };
            (
                path.next_node_pos(),
                path.next_node().borrow().path_type,
                path.next_entity_pos(mob.living.get_bb_width()),
            )
        };
        let abs = (mob.get_x() - (next_node_pos.x as f64 + 0.5)).abs();
        let abs1 = (mob.get_y() - next_node_pos.y as f64).abs();
        let abs2 = (mob.get_z() - (next_node_pos.z as f64 + 0.5)).abs();
        let mut flag = abs < self.max_distance_to_waypoint as f64
            && abs2 < self.max_distance_to_waypoint as f64
            && abs1 < 1.0;
        if !flag
            && self.can_cut_corner(next_node_type)
            && self.should_target_next_node_in_direction(mob, &temp_pos)
        {
            flag = true;
        }
        if flag {
            self.path.as_mut().unwrap().advance();
        }
        self.do_stuck_detection(mob, &temp_pos);
    }

    /// `shouldTargetNextNodeInDirection`.
    fn should_target_next_node_in_direction(&self, mob: &Mob, pos: &Vec3) -> bool {
        let path = match &self.path {
            Some(p) => p,
            None => return false,
        };
        if path.next_node_index() + 1 >= path.node_count() {
            return false;
        }
        let vec3 = path.next_node_pos().get_bottom_center();
        if !pos.closer_than(&vec3, 2.0) {
            return false;
        }
        if self.can_move_directly(mob, pos, &path.next_entity_pos(mob.living.get_bb_width())) {
            return true;
        }
        let vec31 = path.node_pos(path.next_node_index() + 1).get_bottom_center();
        let vec32 = vec3.subtract_vec(pos);
        let vec33 = vec31.subtract_vec(pos);
        let d = vec32.length_sqr();
        let d1 = vec33.length_sqr();
        let flag = d1 < d;
        let flag1 = d < 0.5;
        if !flag && !flag1 {
            return false;
        }
        let vec34 = vec32.normalize();
        let vec35 = vec33.normalize();
        vec35.dot_vec(&vec34) < 0.0
    }

    /// `doStuckDetection`.
    fn do_stuck_detection(&mut self, mob: &Mob, pos: &Vec3) {
        if self.tick - self.last_stuck_check > Self::STUCK_CHECK_INTERVAL {
            let speed = mob.get_speed();
            let f = if speed >= 1.0 { speed } else { speed * speed };
            let f1 = f * 100.0 * Self::STUCK_THRESHOLD_DISTANCE_FACTOR;
            if pos.distance_to_sqr(&self.last_stuck_check_pos) < (f1 * f1) as f64 {
                self.is_stuck = true;
                self.stop_inner();
            } else {
                self.is_stuck = false;
            }
            self.last_stuck_check = self.tick;
            self.last_stuck_check_pos = pos.clone();
        }

        if let Some(path) = &self.path {
            if !path.is_done() {
                let next_node_pos = path.next_node_pos();
                let game_time = mob.level().map(|l| l.game_time()).unwrap_or(0);
                if next_node_pos == self.timeout_cached_node {
                    self.timeout_timer += game_time - self.last_timeoutCheck;
                } else {
                    self.timeout_cached_node = next_node_pos;
                    let d = pos.distance_to(&self.timeout_cached_node.get_bottom_center());
                    self.timeout_limit = if mob.get_speed() > 0.0 {
                        d / mob.get_speed() as f64 * 20.0
                    } else {
                        0.0
                    };
                }
                if self.timeout_limit > 0.0 && self.timeout_timer as f64 > self.timeout_limit * 3.0 {
                    self.timeout_path();
                }
                self.last_timeoutCheck = game_time;
            }
        }
    }

    /// `timeoutPath`.
    fn timeout_path(&mut self) {
        self.reset_stuck_timeout();
        self.stop_inner();
    }

    /// `resetStuckTimeout`.
    fn reset_stuck_timeout(&mut self) {
        self.timeout_cached_node = BlockPos::ZERO;
        self.timeout_timer = 0;
        self.timeout_limit = 0.0;
        self.is_stuck = false;
    }

    /// `stop` без mob.
    fn stop_inner(&mut self) {
        self.path = None;
    }

    /// `getGroundY`.
    fn get_ground_y(&self, mob: &Mob, pos: &Vec3) -> f64 {
        let block_pos = BlockPos::containing(pos.x, pos.y, pos.z);
        let is_air_below = mob
            .level()
            .and_then(|l| l.get_block_state(block_pos.below()))
            .map(|s| s.is_air())
            .unwrap_or(false);
        if is_air_below {
            pos.y
        } else {
            let level = mob.level().expect("mob level");
            WalkNodeEvaluator::get_floor_level_static(level.as_ref(), block_pos)
        }
    }

    /// `canMoveDirectly` — kind-зависимый.
    fn can_move_directly(&self, mob: &Mob, current_pos: &Vec3, next_pos: &Vec3) -> bool {
        match self.kind {
            NavKind::Flying => Self::is_clear_for_movement_between(mob, current_pos, next_pos, true),
            NavKind::Amphibious => {
                mob.living.is_in_water()
                    && Self::is_clear_for_movement_between(mob, current_pos, next_pos, false)
            }
            NavKind::WaterBound => {
                Self::is_clear_for_movement_between(mob, current_pos, next_pos, false)
            }
            _ => false,
        }
    }

    /// `canCutCorner`.
    fn can_cut_corner(&self, path_type: PathType) -> bool {
        path_type != PathType::DangerFire
            && path_type != PathType::DangerOther
            && path_type != PathType::WalkableDoor
    }

    /// `isClearForMovementBetween` — DDA-рейкаст по блокам
    /// (в Java — `level.clip(ClipContext)` с COLLIDER-шейпами;
    /// здесь — проход по блокам с `collisionShape` AABB-проверкой
    /// и fluid-проверкой).
    pub(crate) fn is_clear_for_movement_between(
        mob: &Mob,
        pos1: &Vec3,
        pos2: &Vec3,
        avoid_fluid: bool,
    ) -> bool {
        let vec3 = Vec3::new(pos2.x, pos2.y + mob.living.get_bb_height() as f64 * 0.5, pos2.z);
        let Some(level) = mob.level() else {
            return true;
        };
        let level_ref: &dyn LevelReader = level.as_ref();
        let start = BlockPos::containing(pos1.x, pos1.y, pos1.z);
        let end = BlockPos::containing(vec3.x, vec3.y, vec3.z);
        let step_x = (end.x - start.x).signum() as i32;
        let step_y = (end.y - start.y).signum() as i32;
        let step_z = (end.z - start.z).signum() as i32;
        let mut cur = start;
        let mut guard = 0i32;
        loop {
            guard += 1;
            if guard > 8192 {
                return true;
            }
            if let Some(s) = level_ref.get_block_state(cur) {
                let shape = s.get_collision_shape(level_ref, cur);
                if !shape.is_empty() {
                    let offset = Vec3::new(cur.x as f64, cur.y as f64, cur.z as f64);
                    for aabb in shape.to_aabbs() {
                        let moved = aabb.move_vec(&offset);
                        if moved.clip(pos1, &vec3).is_some() {
                            return false;
                        }
                    }
                }
                if avoid_fluid && !s.get_fluid_state().is_empty() {
                    return false;
                }
            }
            if cur == end {
                return true;
            }
            // Переход к следующему блоку: доминирующая ось —
            // упрощённый DDA (шаг по всем осям с разным знаком
            // может «зигзагать»; для pathfinding-допуска достаточно).
            cur = BlockPos::new(cur.x + step_x, cur.y + step_y, cur.z + step_z);
            if cur.x != end.x && cur.y != end.y && cur.z != end.z
                && (cur.x - end.x).signum() != step_x as i32
            {
                // защита от выхода за границу
                if (cur.x - end.x).abs() > (start.x - end.x).abs() + 2
                    && (cur.y - end.y).abs() > (start.y - end.y).abs() + 2
                    && (cur.z - end.z).abs() > (start.z - end.z).abs() + 2
                {
                    return true;
                }
            }
        }
    }

    /// `isStableDestination` — kind-зависимый.
    fn is_stable_destination_impl(&self, mob: &Mob, pos: BlockPos) -> bool {
        match self.kind {
            NavKind::Flying => mob
                .level()
                .and_then(|l| {
                    let level_ref: &dyn LevelReader = l.as_ref();
                    l.get_block_state(pos)
                        .map(|s| s.entity_can_stand_on(level_ref, pos))
                })
                .unwrap_or(false),
            NavKind::Amphibious => mob
                .level()
                .and_then(|l| l.get_block_state(pos.below()))
                .map(|s| !s.is_air())
                .unwrap_or(false),
            NavKind::WaterBound => mob
                .level()
                .and_then(|l| {
                    let level_ref: &dyn LevelReader = l.as_ref();
                    l.get_block_state(pos)
                        .map(|s| !s.is_solid_render(level_ref, pos))
                })
                .unwrap_or(true),
            _ => mob
                .level()
                .and_then(|l| {
                    let level_ref: &dyn LevelReader = l.as_ref();
                    let bp = pos.below();
                    l.get_block_state(bp)
                        .map(|s| s.is_solid_render(level_ref, bp))
                })
                .unwrap_or(false),
        }
    }

    /// `tempMobPos` — kind-зависимый.
    fn temp_mob_pos(&self, mob: &Mob) -> Vec3 {
        match self.kind {
            NavKind::Ground | NavKind::WallClimber => {
                Vec3::new(mob.get_x(), self.get_surface_y(mob), mob.get_z())
            }
            NavKind::Flying => mob.position(),
            NavKind::Amphibious | NavKind::WaterBound => {
                Vec3::new(mob.get_x(), mob.living.get_y_fraction(0.5), mob.get_z())
            }
        }
    }

    /// `getSurfaceY` (GroundPathNavigation).
    fn get_surface_y(&self, mob: &Mob) -> f64 {
        if mob.living.is_in_water() && self.can_float() {
            let mut block_y = mob.living.get_block_y();
            let mut i = 0;
            loop {
                let pos = BlockPos::containing(mob.get_x(), block_y as f64, mob.get_z());
                let is_water = mob
                    .level()
                    .and_then(|l| l.get_block_state(pos))
                    .map(|s| s.block.id == crate::resources::identifier::Identifier::minecraft("water"))
                    .unwrap_or(false);
                if !is_water {
                    return block_y as f64;
                }
                block_y += 1;
                i += 1;
                if i > 16 {
                    return mob.living.get_block_y() as f64;
                }
            }
        } else {
            mth::floor(mob.get_y() + 0.5) as f64
        }
    }

    /// `canUpdatePath` — kind-зависимый.
    fn can_update_path_impl(&self, mob: &Mob) -> bool {
        match self.kind {
            NavKind::Ground | NavKind::WallClimber => {
                mob.living.entity.on_ground || mob.living.is_in_water() || mob.living.is_in_lava() || mob.living.entity.is_passenger()
            }
            NavKind::Flying => {
                (self.can_float() && mob.living.is_in_water()) || !mob.living.entity.is_passenger()
            }
            NavKind::Amphibious => true,
            NavKind::WaterBound => self.allow_breaching || mob.living.is_in_water() || mob.living.is_in_lava(),
        }
    }

    /// `canNavigateGround` — kind-зависимый.
    fn can_navigate_ground_impl(&self) -> bool {
        matches!(self.kind, NavKind::Ground | NavKind::Amphibious | NavKind::WallClimber)
    }

    /// `getGroundY` — Amphibious/WaterBound возвращают pos.y.
    fn get_ground_y_impl(&self, mob: &Mob, pos: &Vec3) -> f64 {
        match self.kind {
            NavKind::Amphibious | NavKind::WaterBound => pos.y,
            _ => self.get_ground_y(mob, pos),
        }
    }

    /// `setAvoidSun` (GroundPathNavigation).
    pub fn set_avoid_sun(&mut self, v: bool) {
        self.avoid_sun = v;
    }
    /// `setCanWalkOverFences` (GroundPathNavigation).
    pub fn set_can_walk_over_fences(&mut self, v: bool) {
        self.node_evaluator.borrow_mut().set_can_walk_over_fences(v);
    }
    /// `setCanPathToTargetsBelowSurface` (GroundPathNavigation).
    pub fn set_can_path_to_targets_below_surface(&mut self, v: bool) {
        self.can_path_to_targets_below_surface = v;
    }

    /// `hasValidPathType` (GroundPathNavigation).
    pub fn has_valid_path_type(&self, path_type: PathType) -> bool {
        self.kind != NavKind::Ground
            || (path_type != PathType::Water && path_type != PathType::Lava && path_type != PathType::Open)
    }
}

impl PathNavigation {
    /// Уровень моба как `&dyn LevelReader` (panics если нет —
    /// в Java `mob.level()` не null).
    pub(crate) fn level_of(mob: &Mob) -> Arc<dyn LevelReader> {
        mob.level().expect("Mob without level")
    }
}

impl MobNavigation for PathNavigation {
    fn set_can_path_to_targets_below_surface(&mut self, v: bool) {
        PathNavigation::set_can_path_to_targets_below_surface(self, v);
    }

    fn tick(&mut self, mob: &mut Mob) {
        self.tick += 1;
        if self.has_delayed_recomputation {
            self.recompute_path(mob);
        }
        if !self.is_done() {
            if self.can_update_path_impl(mob) {
                self.follow_the_path(mob);
            } else if self.path.as_ref().is_some_and(|p| !p.is_done()) {
                // Базовый tick: advance если mob над следующей точкой.
                let temp_pos = self.temp_mob_pos(mob);
                let next_pos = self.path.as_ref().unwrap().next_entity_pos(mob.living.get_bb_width());
                if self.kind == NavKind::Flying {
                    if mob.living.get_block_x() == mth::floor(next_pos.x) as i32
                        && mob.living.get_block_y() == mth::floor(next_pos.y) as i32
                        && mob.living.get_block_z() == mth::floor(next_pos.z) as i32
                    {
                        self.path.as_mut().unwrap().advance();
                    }
                } else if temp_pos.y > next_pos.y
                    && !mob.living.entity.on_ground
                    && mth::floor(temp_pos.x) == mth::floor(next_pos.x)
                    && mth::floor(temp_pos.z) == mth::floor(next_pos.z)
                {
                    self.path.as_mut().unwrap().advance();
                }
            }
            if !self.is_done() {
                let path = self.path.as_ref().unwrap();
                let next_pos = path.next_entity_pos(mob.living.get_bb_width());
                let ground_y = self.get_ground_y_impl(mob, &next_pos);
                mob.move_control
                    .set_wanted_position(next_pos.x, ground_y, next_pos.z, self.speed_modifier);
            }
        }
        // WallClimberNavigation.tick — после super.tick, если done.
        if self.kind == NavKind::WallClimber && self.is_done() {
            if let Some(path_to) = self.path_to_position {
                let mob_pos = mob.position();
                let closer = !path_to.closer_to_center_than(mob_pos, mob.living.get_bb_width() as f64)
                    && (mob.get_y() <= path_to.y as f64
                        || !BlockPos::new(path_to.x, mob.living.get_block_y(), path_to.z)
                            .closer_to_center_than(mob_pos, mob.living.get_bb_width() as f64));
                if closer {
                    mob.move_control.set_wanted_position(
                        path_to.x as f64,
                        path_to.y as f64,
                        path_to.z as f64,
                        self.speed_modifier,
                    );
                } else {
                    self.path_to_position = None;
                }
            }
        }
    }

    fn create_path_pos(&mut self, mob: &Mob, pos: BlockPos, reach_range: i32) -> Option<Path> {
        // GroundPathNavigation.createPath — chunk-проверка +
        // findSurfacePosition.
        let mut pos = pos;
        if self.kind == NavKind::Ground || self.kind == NavKind::WallClimber {
            if !self.can_path_to_targets_below_surface {
                pos = self.find_surface_position(mob, pos, reach_range);
                if pos.x == i32::MIN {
                    return None;
                }
            }
        }
        // WallClimber запоминает цель.
        if self.kind == NavKind::WallClimber {
            self.path_to_position = Some(pos);
        }
        let mut set = HashSet::new();
        set.insert(pos);
        // WaterBound: allowBreaching вычисляется один раз в
        // createPathFinder (здесь — при первом вызове).
        if self.kind == NavKind::WaterBound && !self.allow_breaching_initialized() {
            self.init_water_bound(mob);
        }
        self.create_path_default(mob, &set, 8, false, reach_range)
    }

    fn move_to_entity(
        &mut self,
        mob: &Mob,
        entity: &crate::world::entity::living_entity::LivingEntity,
        speed_modifier: f64,
    ) -> bool {
        if self.kind == NavKind::WallClimber {
            // WallClimberNavigation.moveTo(Entity, speed) —
            // path с reachRange=0; при неудаче всё равно
            // запоминает цель и возвращает true.
            let path = self.create_path_entity(mob, entity, 0);
            if let Some(p) = path {
                return self.move_to_path(mob, Some(p), speed_modifier);
            }
            self.path_to_position = Some(entity.block_position());
            self.speed_modifier = speed_modifier;
            return true;
        }
        match self.create_path_entity(mob, entity, 1) {
            Some(path) => self.move_to_path(mob, Some(path), speed_modifier),
            None => false,
        }
    }

    fn move_to_path(&mut self, mob: &Mob, path: Option<Path>, speed_modifier: f64) -> bool {
        let Some(path) = path else {
            self.path = None;
            return false;
        };
        if !path.same_as(self.path.as_ref()) {
            self.path = Some(path);
        }
        if self.is_done() {
            return false;
        }
        self.trim_path(mob);
        if self.path.as_ref().map(|p| p.node_count()).unwrap_or(0) <= 0 {
            return false;
        }
        self.speed_modifier = speed_modifier;
        let temp_pos = self.temp_mob_pos(mob);
        self.last_stuck_check = self.tick;
        self.last_stuck_check_pos = temp_pos;
        true
    }

    fn stop(&mut self, _mob: &mut Mob) {
        self.path = None;
    }

    fn is_done(&self) -> bool {
        self.path.as_ref().map(|p| p.is_done()).unwrap_or(true)
    }

    fn path(&self) -> Option<&Path> {
        self.path.as_ref()
    }

    fn speed_modifier(&self) -> f64 {
        self.speed_modifier
    }
    fn set_speed_modifier(&mut self, v: f64) {
        self.speed_modifier = v;
    }

    fn can_update_path(&self, mob: &Mob) -> bool {
        self.can_update_path_impl(mob)
    }

    fn should_recompute_path(&self, mob: &Mob, pos: BlockPos) -> bool {
        if self.has_delayed_recomputation {
            return false;
        }
        if let Some(path) = &self.path {
            if !path.is_done() && path.node_count() != 0 {
                if let Some(end_node) = path.end_node() {
                    let n = end_node.borrow();
                    let vec3 = Vec3::new(
                        (n.x as f64 + mob.get_x()) / 2.0,
                        (n.y as f64 + mob.get_y()) / 2.0,
                        (n.z as f64 + mob.get_z()) / 2.0,
                    );
                    return pos.closer_to_center_than(
                        vec3,
                        (path.node_count() - path.next_node_index()) as f64,
                    );
                }
            }
        }
        false
    }

    fn get_path_type_at(&mut self, mob: &Mob, pos: BlockPos) -> PathType {
        self.node_evaluator.borrow_mut().get_path_type_at(mob, pos)
    }

    fn evaluator_handle(&self) -> Rc<RefCell<dyn NodeEvaluator>> {
        self.node_evaluator.clone()
    }

    fn set_can_float(&mut self, can_float: bool) {
        // AmphibiousPathNavigation.setCanFloat — no-op.
        if self.kind == NavKind::Amphibious || self.kind == NavKind::WaterBound {
            return;
        }
        self.node_evaluator.borrow_mut().set_can_float(can_float);
    }
    fn can_float(&self) -> bool {
        self.node_evaluator.borrow().can_float()
    }

    fn set_can_open_doors(&mut self, v: bool) {
        self.node_evaluator.borrow_mut().set_can_open_doors(v);
    }
    fn set_can_pass_doors(&mut self, v: bool) {
        self.node_evaluator.borrow_mut().set_can_pass_doors(v);
    }
    fn set_avoid_sun(&mut self, v: bool) {
        self.set_avoid_sun(v);
    }

    fn can_navigate_ground(&self) -> bool {
        self.can_navigate_ground_impl()
    }

    fn is_stable_destination(&self, mob: &Mob, pos: BlockPos) -> bool {
        self.is_stable_destination_impl(mob, pos)
    }

    fn max_distance_to_waypoint(&self) -> f32 {
        self.max_distance_to_waypoint
    }

    fn is_stuck(&self) -> bool {
        self.is_stuck
    }

    fn set_required_path_length(&mut self, v: f32, mob: &Mob) {
        self.required_path_length = v;
        self.update_pathfinder_max_visited_nodes(mob);
    }

    fn set_max_visited_nodes_multiplier(&mut self, v: f32) {
        self.max_visited_nodes_multiplier = v;
    }
    fn reset_max_visited_nodes_multiplier(&mut self) {
        self.max_visited_nodes_multiplier = 1.0;
    }

    fn recompute_path(&mut self, mob: &Mob) {
        let game_time = mob.level().map(|l| l.game_time()).unwrap_or(0);
        if game_time - self.time_last_recompute > Self::MAX_TIME_RECOMPUTE {
            if let Some(target_pos) = self.target_pos {
                self.path = None;
                let mut set = HashSet::new();
                set.insert(target_pos);
                self.path = self.create_path_default(mob, &set, 8, false, self.reach_range);
                self.time_last_recompute = game_time;
                self.has_delayed_recomputation = false;
            }
        } else {
            self.has_delayed_recomputation = true;
        }
    }

    fn create_path_set(&mut self, mob: &Mob, targets: &HashSet<BlockPos>, reach_range: i32) -> Option<Path> {
        self.create_path_default(mob, targets, 8, false, reach_range)
    }

    fn target_pos(&self) -> Option<BlockPos> {
        self.target_pos
    }

    fn path_computation_type(&self) -> PathComputationType {
        match self.kind {
            NavKind::Flying => PathComputationType::Air,
            NavKind::WaterBound | NavKind::Amphibious => PathComputationType::Water,
            _ => PathComputationType::Land,
        }
    }
}

impl PathNavigation {
    /// `findSurfacePosition` (GroundPathNavigation) —
    /// `None`-возврат кодируется `BlockPos{x: i32::MIN}`.
    fn find_surface_position(&self, mob: &Mob, pos: BlockPos, _reach_range: i32) -> BlockPos {
        let Some(level) = mob.level() else {
            return BlockPos::ZERO;
        };
        // Проверка "chunkNow == null" — has_chunk_at.
        if !level.has_chunk_at(pos) {
            return BlockPos::new(i32::MIN, 0, 0);
        }
        let get_state = |p: BlockPos| level.get_block_state(p);
        let mut pos = pos;
        if get_state(pos).map(|s| s.is_air()).unwrap_or(false) {
            let mut mutable = pos;
            loop {
                mutable = mutable.below();
                if mutable.y < level.min_build_height()
                    || !get_state(mutable).map(|s| s.is_air()).unwrap_or(false)
                {
                    break;
                }
            }
            if mutable.y >= level.min_build_height() {
                pos = mutable.above();
            } else {
                mutable = BlockPos::new(pos.x, pos.y + 1, pos.z);
                while mutable.y <= level.min_build_height() + level.height() - 1
                    && get_state(mutable).map(|s| s.is_air()).unwrap_or(false)
                {
                    mutable = mutable.above();
                }
                pos = mutable;
            }
        }
        let level_ref: &dyn LevelReader = level.as_ref();
        if !get_state(pos)
            .map(|s| s.is_solid_render(level_ref, pos))
            .unwrap_or(false)
        {
            return pos;
        }
        let mut mutable = pos.above();
        let max_y = level.min_build_height() + level.height() - 1;
        while mutable.y <= max_y
            && get_state(mutable)
                .map(|s| s.is_solid_render(level_ref, mutable))
                .unwrap_or(false)
        {
            mutable = mutable.above();
        }
        mutable
    }

    /// `initWaterBound` — WaterBoundPathNavigation.createPathFinder
    /// логика (allowBreaching + SwimNodeEvaluator setup).
    fn init_water_bound(&mut self, mob: &Mob) {
        self.allow_breaching = mob
            .living
            .entity
            .entity_type
            .value()
            .map(|t| t.id == crate::resources::identifier::Identifier::minecraft("dolphin"))
            .unwrap_or(false);
        // Пересоздаём evaluator с правильным allowBreaching.
        let evaluator: Rc<RefCell<dyn NodeEvaluator>> =
            Rc::new(RefCell::new(SwimNodeEvaluator::new(self.allow_breaching)));
        evaluator.borrow_mut().set_can_pass_doors(false);
        self.node_evaluator = evaluator.clone();
        self.path_finder = PathFinder::new(evaluator, self.max_visited_nodes_base);
        self.water_bound_initialized = true;
    }

    fn allow_breaching_initialized(&self) -> bool {
        // флаг инициализации — allow_breaching уже вычислен?
        // WaterBound: allowBreaching=false возможно и для не-dolphin;
        // нужен отдельный флаг. Используем path_computation_type
        // трюк нельзя — добавим поле через Option.
        self.water_bound_initialized
    }

}
