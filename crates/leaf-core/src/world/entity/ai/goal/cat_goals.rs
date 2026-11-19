//! Перенос кошачьих AI-целей `net.minecraft.world.entity.ai.goal`:
//! CatSitOnBlockGoal, CatLieOnBedGoal (обёртки над MoveToBlockGoal).

use crate::core::block_pos::BlockPos;
use crate::resources::identifier::Identifier;
use crate::world::entity::ai::goal::goal::{Goal, GoalFlag, GoalFlagSet};
use crate::world::entity::ai::goal::special_goals::MoveToBlockGoal;
use crate::world::entity::mob::Mob;
use crate::world::level::block::state::property::StateValue;
use crate::world::level::level_reader::LevelReader;

/// `BlockTags.BEDS` id.
fn beds_tag() -> Identifier {
    Identifier::minecraft("beds")
}

/// `CatSitOnBlockGoal` — кот садится на сундук/печь/кровать.
pub struct CatSitOnBlockGoal {
    inner: MoveToBlockGoal,
}

impl CatSitOnBlockGoal {
    /// `new(cat, speedModifier)` — `super(cat, speed, 8)`.
    pub fn new(mob: &Mob, speed_modifier: f64) -> Self {
        Self {
            inner: MoveToBlockGoal::new(mob, speed_modifier, 8, |level, pos| {
                Self::is_valid_target(level, pos)
            }),
        }
    }

    /// `isValidTarget(level, pos)`:
    /// над блоком пусто; сундук с <1 открытым, горящая печь,
    /// или кровать (part != HEAD).
    fn is_valid_target(level: &dyn LevelReader, pos: BlockPos) -> bool {
        let above_air = level
            .get_block_state(pos.above())
            .map(|s| s.is_air())
            .unwrap_or(false);
        if !above_air {
            return false;
        }
        let Some(state) = level.get_block_state(pos) else {
            return false;
        };
        match state.block.id.path.as_str() {
            "chest" => level.container_open_count(pos) < 1,
            "furnace" => matches!(
                state.values.get_by_name("lit"),
                Some(StateValue::Bool(true))
            ),
            _ => {
                state.is_block_tag(&beds_tag())
                    && state
                        .values
                        .get_by_name("part")
                        .map(|v| !matches!(v, StateValue::Str(s) if s == "head"))
                        .unwrap_or(true)
            }
        }
    }
}

impl Goal for CatSitOnBlockGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        mob.tamable_is_tame()
            && !mob.tamable_is_ordered_to_sit()
            && self.inner.can_use(mob)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.inner.can_continue_to_use(mob)
    }

    fn start(&mut self, mob: &mut Mob) {
        self.inner.start(mob);
        mob.set_in_sitting_pose(false);
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.inner.stop(mob);
        mob.set_in_sitting_pose(false);
    }

    fn tick(&mut self, mob: &mut Mob) {
        self.inner.tick(mob);
        let reached = self.inner.is_reached_target();
        mob.set_in_sitting_pose(reached);
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn get_flags(&self) -> GoalFlagSet {
        self.inner.get_flags()
    }
}

/// `CatLieOnBedGoal` — кот ложится на кровать.
pub struct CatLieOnBedGoal {
    inner: MoveToBlockGoal,
}

impl CatLieOnBedGoal {
    /// `new(cat, speedModifier, searchRange)` —
    /// `super(cat, speed, searchRange, 6)`, verticalSearchStart=-2,
    /// flags JUMP+MOVE, `nextStartTick` = 40.
    pub fn new(mob: &Mob, speed_modifier: f64, search_range: i32) -> Self {
        let mut inner = MoveToBlockGoal::with_vertical(
            mob,
            speed_modifier,
            search_range,
            6,
            |level, pos| Self::is_valid_target(level, pos),
        );
        inner.vertical_search_start = -2;
        inner.next_start_tick_override = Some(40);
        Self { inner }
    }

    /// `isValidTarget` — над блоком пусто и блок в `BlockTags.BEDS`.
    fn is_valid_target(level: &dyn LevelReader, pos: BlockPos) -> bool {
        let above_air = level
            .get_block_state(pos.above())
            .map(|s| s.is_air())
            .unwrap_or(false);
        above_air
            && level
                .get_block_state(pos)
                .map(|s| s.is_block_tag(&beds_tag()))
                .unwrap_or(false)
    }
}

impl Goal for CatLieOnBedGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        mob.tamable_is_tame()
            && !mob.tamable_is_ordered_to_sit()
            && !mob.lying
            && self.inner.can_use(mob)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.inner.can_continue_to_use(mob)
    }

    fn start(&mut self, mob: &mut Mob) {
        self.inner.start(mob);
        mob.set_in_sitting_pose(false);
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.inner.stop(mob);
        mob.lying = false;
    }

    fn tick(&mut self, mob: &mut Mob) {
        self.inner.tick(mob);
        mob.set_in_sitting_pose(false);
        if !self.inner.is_reached_target() {
            mob.lying = false;
        } else if !mob.lying {
            mob.lying = true;
        }
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn get_flags(&self) -> GoalFlagSet {
        GoalFlagSet::from_iter([GoalFlag::Jump, GoalFlag::Move])
    }
}
