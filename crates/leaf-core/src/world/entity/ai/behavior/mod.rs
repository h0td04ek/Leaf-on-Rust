//! `net.minecraft.world.entity.ai.behavior` — поведения Brain.

pub mod behavior;

pub use behavior::{
    cond_set, mem_cond, BehaviorBase, BehaviorControl, BehaviorStatus, PriorityBehavior,
    ShufflingList,
};
