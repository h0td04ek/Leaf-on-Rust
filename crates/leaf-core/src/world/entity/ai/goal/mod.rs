//! `net.minecraft.world.entity.ai.goal` — система целей AI.

pub mod goal;
pub mod goal_selector;
pub mod movement_look;

pub use goal::{dyn_goal, DynGoal, Goal, GoalFlag, GoalFlagSet, WrappedGoal};
pub use goal_selector::GoalSelector;
