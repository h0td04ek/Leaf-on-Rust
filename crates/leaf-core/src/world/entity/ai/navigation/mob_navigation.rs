//! `MobNavigation` — интерфейс `PathNavigation` (в Java —
//! абстрактный класс; у нас трейт + базовая реализация
//! `PathNavigation`).

use crate::core::block_pos::BlockPos;
use crate::world::entity::mob::Mob;
use crate::world::level::pathfinder::node_evaluator::NodeEvaluator;
use crate::world::level::pathfinder::path::Path;
use crate::world::level::pathfinder::path_computation_type::PathComputationType;
use crate::world::level::pathfinder::path_type::PathType;
use std::cell::RefCell;
use std::rc::Rc;

/// `PathNavigation` — интерфейс навигации.
///
/// Методы принимают `&Mob`/`&mut Mob` вместо хранения ссылки
/// (в Rust нельзя хранить `&mut Mob` в поле моба).
pub trait MobNavigation {
    /// `tick`.
    fn tick(&mut self, mob: &mut Mob);

    /// `createPath(pos, reachRange)`.
    fn create_path_pos(&mut self, mob: &Mob, pos: BlockPos, reach_range: i32) -> Option<Path>;

    /// `createPath(entity, reachRange)` — entity у нас `LivingEntity`.
    fn create_path_entity(&mut self, mob: &Mob, entity: &crate::world::entity::living_entity::LivingEntity, reach_range: i32) -> Option<Path> {
        self.create_path_pos(mob, entity.block_position(), reach_range)
    }

    /// `moveTo(path, speedModifier)`.
    fn move_to_path(&mut self, mob: &Mob, path: Option<Path>, speed_modifier: f64) -> bool;

    /// `moveTo(x, y, z, speedModifier)`.
    fn move_to_pos(&mut self, mob: &Mob, x: f64, y: f64, z: f64, speed_modifier: f64) -> bool {
        let path = self.create_path_pos(mob, BlockPos::containing(x, y, z), 1);
        self.move_to_path(mob, path, speed_modifier)
    }

    /// `moveTo(entity, speedModifier)`.
    fn move_to_entity(&mut self, mob: &Mob, entity: &crate::world::entity::living_entity::LivingEntity, speed_modifier: f64) -> bool {
        match self.create_path_entity(mob, entity, 1) {
            Some(path) => self.move_to_path(mob, Some(path), speed_modifier),
            None => false,
        }
    }

    /// `stop`.
    fn stop(&mut self, mob: &mut Mob);

    /// `isDone`.
    fn is_done(&self) -> bool;

    /// `isInProgress`.
    fn is_in_progress(&self) -> bool {
        !self.is_done()
    }

    /// `getPath`.
    fn path(&self) -> Option<&Path>;

    /// `setSpeedModifier`/`getSpeedModifier`-эквивалент —
    /// `speedModifier` поле.
    fn speed_modifier(&self) -> f64;
    fn set_speed_modifier(&mut self, v: f64);

    /// `canUpdatePath` — защищённый в Java; здесь часть API.
    fn can_update_path(&self, mob: &Mob) -> bool;

    /// `shouldRecomputePath`.
    fn should_recompute_path(&self, mob: &Mob, pos: BlockPos) -> bool;

    /// `getPathTypeAt(mob, pos)` — через nodeEvaluator
    /// (`MoveControl.isWalkable` использует).
    fn get_path_type_at(&mut self, mob: &Mob, pos: BlockPos) -> PathType;

    /// `getNodeEvaluator` — Rc-хэндл (общий с pathFinder).
    fn evaluator_handle(&self) -> Rc<RefCell<dyn NodeEvaluator>>;

    /// `setCanFloat`/`canFloat`.
    fn set_can_float(&mut self, can_float: bool);
    fn can_float(&self) -> bool;

    /// `setCanOpenDoors`/`setCanPassDoors`.
    fn set_can_open_doors(&mut self, v: bool);
    fn set_can_pass_doors(&mut self, v: bool);

    /// `setAvoidSun` — `GroundPathNavigation`-специфично
    /// (`RestrictSunGoal`); default no-op для остальных типов.
    fn set_avoid_sun(&mut self, _v: bool) {}

    /// `canNavigateGround`.
    fn can_navigate_ground(&self) -> bool;

    /// `isStableDestination`.
    fn is_stable_destination(&self, mob: &Mob, pos: BlockPos) -> bool;

    /// `getMaxDistanceToWaypoint`.
    fn max_distance_to_waypoint(&self) -> f32;

    /// `isStuck`.
    fn is_stuck(&self) -> bool;

    /// `setRequiredPathLength`/`updatePathfinderMaxVisitedNodes`/
    /// `resetMaxVisitedNodesMultiplier`/`setMaxVisitedNodesMultiplier`.
    fn set_required_path_length(&mut self, v: f32, mob: &Mob);
    fn set_max_visited_nodes_multiplier(&mut self, v: f32);
    fn reset_max_visited_nodes_multiplier(&mut self);

    /// `recomputePath`.
    fn recompute_path(&mut self, mob: &Mob);

    /// `createPath(Stream/Set)` — для целей из множества.
    fn create_path_set(&mut self, mob: &Mob, targets: &std::collections::HashSet<BlockPos>, reach_range: i32) -> Option<Path> {
        let _ = (mob, targets, reach_range);
        None
    }

    /// `getTargetPos`.
    fn target_pos(&self) -> Option<BlockPos>;

    /// `pathType` (`PathComputationType` навигации).
    fn path_computation_type(&self) -> PathComputationType;
}
