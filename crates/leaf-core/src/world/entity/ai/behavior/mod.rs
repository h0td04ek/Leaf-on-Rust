//! `net.minecraft.world.entity.ai.behavior` — поведения Brain.

pub mod behavior;
pub mod behavior_utils;
pub mod gate;
pub mod sinks;
pub mod walk_behaviors;
pub mod simple;

pub use behavior::{
    cond_set, mem_cond, BehaviorBase, BehaviorControl, BehaviorStatus, PriorityBehavior,
    ShufflingList,
};
pub use behavior_utils::*;
pub use gate::*;
pub use sinks::*;
pub use walk_behaviors::*;
pub use simple::*;
