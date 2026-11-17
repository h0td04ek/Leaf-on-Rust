//! `net.minecraft.world.level.levelgen.blending` — блендинг генераций.
pub mod blender;
pub mod blending_data;

pub use blender::{BlendingOutput, Blender, DistanceGetter};
pub use blending_data::{BlendingData, ChunkGenStatus, PackedBlendingData, CELL_COLUMN_COUNT, NO_VALUE};
