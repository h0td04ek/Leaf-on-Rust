//! `net.minecraft.world.entity.ai.goal` — система целей AI.

pub mod cat_goals;
pub mod combat;
pub mod door_goals;
pub mod goal;
pub mod util_goals;
pub mod goal_selector;
pub mod movement_look;
pub mod raid_goals;
pub mod social_goals;
pub mod special_goals;
pub mod target;
pub mod village_goals;

pub use goal::{dyn_goal, DynGoal, Goal, GoalFlag, GoalFlagSet, WrappedGoal};
pub use goal_selector::GoalSelector;
