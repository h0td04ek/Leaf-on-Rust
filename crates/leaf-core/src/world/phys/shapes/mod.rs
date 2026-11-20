//! Перенос `net.minecraft.world.phys.shapes` — формы блоков.

pub mod voxel_shape;

pub use voxel_shape::{
    face_shape_occludes, merged_face_occludes, merged_may_occlude_block, BooleanOp, VoxelShape,
    COLLISION_EPSILON,
};
