//! `net.minecraft.world.entity.ai.goal` — система целей AI.

pub mod combat;
pub mod goal;
pub mod util_goals;
pub mod goal_selector;
pub mod movement_look;
pub mod target;

pub use goal::{dyn_goal, DynGoal, Goal, GoalFlag, GoalFlagSet, WrappedGoal};
pub use goal_selector::GoalSelector;
