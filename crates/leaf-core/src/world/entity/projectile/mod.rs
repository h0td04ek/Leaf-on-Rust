//! `net.minecraft.world.entity.projectile` — снаряды.
//!
pub mod projectile;
pub mod projectile_util;

pub use projectile::{
    ChunkLoadLimitAction, Projectile, ProjectileChunkLoadBudget, ProjectileChunkLoadLimits,
    ProjectileDeflection, ProjectileHit,
};
