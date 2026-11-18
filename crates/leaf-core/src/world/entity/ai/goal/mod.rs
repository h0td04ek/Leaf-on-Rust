//! `net.minecraft.world.entity.ai.goal` — система целей AI.

pub mod goal;
pub mod goal_selector;

pub use goal::{dyn_goal, DynGoal, Goal, GoalFlag, GoalFlagSet, WrappedGoal};
pub use goal_selector::GoalSelector;
