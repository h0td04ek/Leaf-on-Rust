//! Перенос `net.minecraft.world.phys.shapes` — формы блоков.

pub mod collision_context;
pub mod voxel_shape;

pub use collision_context::CollisionContext;

pub use voxel_shape::{
    collision_aabb_is_empty, face_shape_occludes, merged_face_occludes, merged_may_occlude_block,
    single_aabb_representation, voxel_shape_intersect_no_empty, BooleanOp, VoxelShape,
    COLLISION_EPSILON,
};
