//! Перенос дверных AI-целей `net.minecraft.world.entity.ai.goal`:
//! DoorInteractGoal (база), OpenDoorGoal, BreakDoorGoal.

use crate::core::block_pos::BlockPos;
use crate::world::entity::ai::goal::goal::{Goal, GoalFlag, GoalFlagSet};
use crate::world::entity::ai::util::goal_utils;
use crate::world::entity::mob::Mob;
use crate::world::level::block::BlockKind;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::property::StateValue;
use crate::world::level::pathfinder::path::Path;
use crate::world::difficulty::Difficulty;
use std::sync::Arc;

/// `LevelEvent.SOUND_ZOMBIE_WOODEN_DOOR` = 1019.
const SOUND_ZOMBIE_WOODEN_DOOR: i32 = 1019;
/// `LevelEvent.SOUND_ZOMBIE_DOOR_CRASH` = 1021.
const SOUND_ZOMBIE_DOOR_CRASH: i32 = 1021;

/// `DoorBlock.isWoodenDoor(level, pos)` — деревянная дверь
/// (не железная). `BlockKind::Door{wooden}` + fallback по id.
fn is_wooden_door(level: &dyn crate::world::level::level_reader::LevelReader, pos: BlockPos) -> bool {
    let Some(state) = level.get_block_state(pos) else {
        return false;
    };
    match state.block.kind {
        BlockKind::Door { wooden } => wooden,
        _ => {
            let p = state.block.id.path.as_str();
            p.ends_with("_door") && p != "iron_door"
        }
    }
}

/// `DoorBlock.OPEN`-значение состояния двери.
fn door_is_open(state: &BlockState) -> bool {
    state
        .values
        .get_by_name("open")
        .map(|v| matches!(v, StateValue::Bool(true)))
        .unwrap_or(false)
}

/// `DoorBlock.setOpen(mob, level, state, pos, open)` —
/// установка `open`-свойства через `set_block_state`.
fn set_door_open(level: &dyn crate::world::level::level_reader::LevelReader, pos: BlockPos, open: bool) {
    let Some(state) = level.get_block_state(pos) else {
        return;
    };
    let mut state = state;
    state.values.set_erased("open", StateValue::Bool(open));
    level.set_block_state(pos, &state);
}

/// `DoorInteractGoal` — абстрактная база: моб останавливается у двери
/// на пути и (по наследнику) открывает/ломает её.
pub struct DoorInteractGoal {
    door_pos: BlockPos,
    has_door: bool,
    passed: bool,
    door_open_dir_x: f32,
    door_open_dir_z: f32,
}

impl DoorInteractGoal {
    /// `new(mob)` — проверка GroundPathNavigation как в Java.
    pub fn new(mob: &Mob) -> Option<Self> {
        if !goal_utils::has_ground_path_navigation(mob) {
            return None;
        }
        Some(Self {
            door_pos: BlockPos::ZERO,
            has_door: false,
            passed: false,
            door_open_dir_x: 0.0,
            door_open_dir_z: 0.0,
        })
    }

    /// `isOpen()`.
    fn is_open(&mut self, mob: &Mob) -> bool {
        if !self.has_door {
            return false;
        }
        let Some(level) = mob.living.entity.level.clone() else {
            self.has_door = false;
            return false;
        };
        let Some(state) = level.get_block_state(self.door_pos) else {
            self.has_door = false;
            return false;
        };
        if !matches!(state.block.kind, BlockKind::Door { .. })
            && !(state.block.id.path.ends_with("_door"))
        {
            self.has_door = false;
            return false;
        }
        door_is_open(&state)
    }

    /// `setOpen(open)`.
    fn set_open(&self, mob: &Mob, open: bool) {
        if !self.has_door {
            return;
        }
        let Some(level) = mob.living.entity.level.clone() else {
            return;
        };
        set_door_open(level.as_ref(), self.door_pos, open);
    }

    /// Общая `canUse` базы DoorInteractGoal.
    fn base_can_use(&mut self, mob: &mut Mob) -> bool {
        if !goal_utils::has_ground_path_navigation(mob) {
            return false;
        }
        if mob.living.entity.horizontal_collision {
            return false;
        }
        let Some(level) = mob.living.entity.level.clone() else {
            return false;
        };
        let Some(path) = Self::current_path(mob) else {
            return false;
        };
        if !path.processed || path.is_done() {
            return false;
        }
        let up_to = (path.next_node_index() + 2).min(path.node_count());
        for i in 0..up_to {
            let node = path.node(i);
            let n = node.borrow();
            self.door_pos = BlockPos::new(n.x, n.y + 1, n.z);
            drop(n);
            if mob
                .living
                .entity
                .distance_to_sqr_pos(self.door_pos.x as f64, mob.living.entity.pos.y, self.door_pos.z as f64)
                <= 2.25
            {
                self.has_door = is_wooden_door(level.as_ref(), self.door_pos);
                if self.has_door {
                    return true;
                }
            }
        }
        // `doorPos = mob.blockPosition().above()`.
        self.door_pos = mob.living.block_position().above();
        self.has_door = is_wooden_door(level.as_ref(), self.door_pos);
        self.has_door
    }

    fn current_path(mob: &Mob) -> Option<Path> {
        mob.get_navigation().and_then(|nav| nav.borrow().path().cloned())
    }

    /// Общий `tick` базы — `passed` когда прошли плоскость двери.
    fn base_tick(&mut self, mob: &Mob) {
        let f = self.door_pos.x as f32 + 0.5 - mob.living.entity.pos.x as f32;
        let f1 = self.door_pos.z as f32 + 0.5 - mob.living.entity.pos.z as f32;
        if self.door_open_dir_x * f + self.door_open_dir_z * f1 < 0.0 {
            self.passed = true;
        }
    }
}

impl Goal for DoorInteractGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        self.base_can_use(mob)
    }

    fn can_continue_to_use(&mut self, _mob: &mut Mob) -> bool {
        !self.passed
    }

    fn start(&mut self, mob: &mut Mob) {
        self.passed = false;
        self.door_open_dir_x =
            (self.door_pos.x as f32 + 0.5) - mob.living.entity.pos.x as f32;
        self.door_open_dir_z =
            (self.door_pos.z as f32 + 0.5) - mob.living.entity.pos.z as f32;
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        self.base_tick(mob);
    }

    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::empty()
    }
}

/// `OpenDoorGoal` — открытие двери по пути.
pub struct OpenDoorGoal {
    inner: Option<DoorInteractGoal>,
    close_door: bool,
    forget_time: i32,
}

impl OpenDoorGoal {
    /// `new(mob, closeDoor)`.
    pub fn new(mob: &Mob, close_door: bool) -> Option<Self> {
        Some(Self {
            inner: DoorInteractGoal::new(mob),
            close_door,
            forget_time: 0,
        })
    }
}

impl Goal for OpenDoorGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        self.inner.as_mut().map(|g| g.can_use(mob)).unwrap_or(false)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.close_door
            && self.forget_time > 0
            && self.inner.as_mut().map(|g| g.can_continue_to_use(mob)).unwrap_or(false)
    }

    fn start(&mut self, mob: &mut Mob) {
        self.forget_time = 20;
        if let Some(g) = &self.inner {
            g.set_open(mob, true);
        }
        if let Some(g) = self.inner.as_mut() {
            g.start(mob);
        }
    }

    fn stop(&mut self, mob: &mut Mob) {
        if let Some(g) = &self.inner {
            g.set_open(mob, false);
        }
        if let Some(g) = self.inner.as_mut() {
            g.stop(mob);
        }
    }

    fn tick(&mut self, mob: &mut Mob) {
        self.forget_time -= 1;
        if let Some(g) = self.inner.as_mut() {
            g.base_tick(mob);
        }
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::empty()
    }
}

/// `BreakDoorGoal` — взлом двери (зомби и др.).
pub struct BreakDoorGoal {
    inner: Option<DoorInteractGoal>,
    break_time: i32,
    last_break_progress: i32,
    door_break_time: i32,
    valid_difficulties: Arc<dyn Fn(Difficulty) -> bool + Send + Sync>,
}

impl BreakDoorGoal {
    /// `new(mob, doorBreakTime, validDifficulties)`.
    pub fn new(
        mob: &Mob,
        door_break_time: i32,
        valid_difficulties: Arc<dyn Fn(Difficulty) -> bool + Send + Sync>,
    ) -> Option<Self> {
        Some(Self {
            inner: DoorInteractGoal::new(mob),
            break_time: 0,
            last_break_progress: -1,
            door_break_time,
            valid_difficulties,
        })
    }

    /// `getDoorBreakTime()`.
    fn get_door_break_time(&self) -> i32 {
        self.door_break_time.max(240)
    }

    fn door_pos(&self) -> BlockPos {
        self.inner.as_ref().map(|g| g.door_pos).unwrap_or(BlockPos::ZERO)
    }

    fn has_door(&self) -> bool {
        self.inner.as_ref().map(|g| g.has_door).unwrap_or(false)
    }
}

impl Goal for BreakDoorGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let Some(inner) = self.inner.as_mut() else {
            return false;
        };
        if !inner.base_can_use(mob) {
            return false;
        }
        let Some(level) = mob.living.entity.level.clone() else {
            return false;
        };
        level.mob_griefing()
            && (self.valid_difficulties)(level.difficulty())
            && !inner.is_open(mob)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(inner) = self.inner.as_mut() else {
            return false;
        };
        let Some(level) = mob.living.entity.level.clone() else {
            return false;
        };
        let total = self.door_break_time.max(240);
        let closer = inner.door_pos.closer_to_center_than(mob.living.entity.position(), 2.0);
        self.break_time <= total
            && !inner.is_open(mob)
            && closer
            && (self.valid_difficulties)(level.difficulty())
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(inner) = self.inner.as_mut() {
            inner.start(mob);
        }
        self.break_time = 0;
    }

    fn stop(&mut self, mob: &mut Mob) {
        let pos = self.door_pos();
        if let Some(level) = mob.living.entity.level.clone() {
            level.destroy_block_progress(mob.living.entity.id, pos, -1);
        }
        if let Some(inner) = self.inner.as_mut() {
            inner.stop(mob);
        }
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(inner) = self.inner.as_mut() else {
            return;
        };
        inner.base_tick(mob);
        let Some(level) = mob.living.entity.level.clone() else {
            return;
        };
        let door_pos = inner.door_pos;
        if mob.random.borrow_mut().next_int_bound(20) == 0 {
            level.level_event(SOUND_ZOMBIE_WOODEN_DOOR, door_pos, 0);
            if !mob.living.swinging {
                let hand = mob.living.swinging_arm;
                mob.living.swing(hand);
            }
        }

        self.break_time += 1;
        let total = self.get_door_break_time();
        let i = (self.break_time as f32 / total as f32 * 10.0) as i32;
        if i != self.last_break_progress {
            level.destroy_block_progress(mob.living.entity.id, door_pos, i);
            self.last_break_progress = i;
        }

        if self.break_time == total && (self.valid_difficulties)(level.difficulty()) {
            // `removeBlock(doorPos, false)` — air.
            let air = BlockState::air();
            level.set_block_state(door_pos, &air);
            level.level_event(SOUND_ZOMBIE_DOOR_CRASH, door_pos, 0);
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::empty()
    }
}
