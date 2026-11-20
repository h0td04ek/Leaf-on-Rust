//! `net.minecraft.world.level.CollisionGetter` — интерфейс
//! коллизионных запросов уровня (блоки + сущности + border).
//!
//! Отличия от Java:
//! - `Iterable<VoxelShape>` заменён на `Vec<VoxelShape>`
//!   (коллизии собираются eagerly — lazy-итераторы усложнили бы
//!   `&dyn CollisionGetter` без выигрыша на практике).
//! - `Entity` передаётся как `Option<&Arc<Entity>>`.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::world::entity::entity::Entity;
use crate::world::level::block_getter::BlockGetter;
use crate::world::level::border::world_border::WorldBorder;
use crate::world::level::clip_context::ClipContext;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::hit_result::BlockHitResult;
use crate::world::phys::shapes::collision_context::CollisionContext;
use crate::world::phys::shapes::voxel_shape::{join_is_not_empty, BooleanOp, VoxelShape};
use crate::world::phys::vec3::Vec3;

use super::block_collisions::BlockCollisions;
use super::block::state::block_state::BlockState;

/// `CollisionGetter extends BlockGetter`.
pub trait CollisionGetter: BlockGetter {
    /// `getWorldBorder()`.
    fn get_world_border(&self) -> &WorldBorder;

    /// `getChunkForCollisions(chunkX, chunkZ)` — `None` если чанк
    /// не загружен (`@Nullable`).
    fn get_chunk_for_collisions(&self, chunk_x: i32, chunk_z: i32) -> Option<&dyn BlockGetter>;

    /// `getEntityCollisions(entity, collisionBox)` — список форм
    /// коллизий сущностей в боксе.
    fn get_entity_collisions(
        &self,
        entity: Option<&Arc<Entity>>,
        collision_box: &Aabb,
    ) -> Vec<VoxelShape>;

    /// `isUnobstructed(entity, shape)` — default `true`
    /// (переопределяется в `Level`).
    fn is_unobstructed(&self, _entity: Option<&Arc<Entity>>, _shape: &VoxelShape) -> bool {
        true
    }

    /// `isUnobstructed(state, pos, context)`.
    fn is_unobstructed_state(
        &self,
        state: &BlockState,
        pos: BlockPos,
        context: &CollisionContext,
    ) -> bool {
        let collision_shape = self.get_block_collision_shape(state, pos, context);
        collision_shape.is_empty()
            || self.is_unobstructed(None, &collision_shape.move_shape(pos.x as f64, pos.y as f64, pos.z as f64))
    }

    /// `isUnobstructed(entity)`.
    fn is_unobstructed_entity(&self, entity: &Arc<Entity>) -> bool {
        self.is_unobstructed(Some(entity), &VoxelShape::of(entity.get_bounding_box()))
    }

    /// `noCollision(collisionBox)`.
    fn no_collision_box(&self, collision_box: &Aabb) -> bool {
        self.no_collision(None, collision_box)
    }

    /// `noCollision(entity)`.
    fn no_collision_entity(&self, entity: &Arc<Entity>) -> bool {
        self.no_collision(Some(entity), &entity.get_bounding_box())
    }

    /// `noCollision(entity, collisionBox)`.
    fn no_collision(&self, entity: Option<&Arc<Entity>>, collision_box: &Aabb) -> bool {
        self.no_collision_full(entity, collision_box, false)
    }

    /// `noCollision(entity, collisionBox, checkLiquid)`.
    fn no_collision_full(
        &self,
        entity: Option<&Arc<Entity>>,
        collision_box: &Aabb,
        check_liquid: bool,
    ) -> bool {
        self.no_block_collision_liquid(entity, collision_box, check_liquid)
            && self.no_entity_collision(entity, collision_box)
            && self.no_border_collision(entity, collision_box)
    }

    /// `noBlockCollision(entity, boundingBox)`.
    fn no_block_collision(&self, entity: Option<&Arc<Entity>>, bounding_box: &Aabb) -> bool {
        self.no_block_collision_liquid(entity, bounding_box, false)
    }

    /// `noBlockCollision(entity, aabb, flag)`.
    fn no_block_collision_liquid(
        &self,
        entity: Option<&Arc<Entity>>,
        aabb: &Aabb,
        flag: bool,
    ) -> bool {
        let shapes = if flag {
            self.get_block_and_liquid_collisions(entity, aabb)
        } else {
            self.get_block_collisions(entity, aabb)
        };
        shapes.iter().all(|s| s.is_empty())
    }

    /// `noEntityCollision(entity, aabb)`.
    fn no_entity_collision(&self, entity: Option<&Arc<Entity>>, aabb: &Aabb) -> bool {
        self.get_entity_collisions(entity, aabb).is_empty()
    }

    /// `noBorderCollision(entity, aabb)`.
    fn no_border_collision(&self, entity: Option<&Arc<Entity>>, aabb: &Aabb) -> bool {
        let Some(entity) = entity else { return true };
        let Some(shape) = self.border_collision(entity.as_ref(), aabb) else {
            return true;
        };
        let entity_shape = VoxelShape::of(aabb.clone());
        !join_is_not_empty(&shape, &entity_shape, BooleanOp::And)
    }

    /// `getCollisions(entity, collisionBox)` — entity + block.
    fn get_collisions(&self, entity: Option<&Arc<Entity>>, collision_box: &Aabb) -> Vec<VoxelShape> {
        let mut out = self.get_entity_collisions(entity, collision_box);
        out.extend(self.get_block_collisions(entity, collision_box));
        out
    }

    /// `getPreMoveCollisions(entity, collisionBox, pos)`.
    fn get_pre_move_collisions(
        &self,
        entity: Option<&Arc<Entity>>,
        collision_box: &Aabb,
        pos: &Vec3,
    ) -> Vec<VoxelShape> {
        let mut out = self.get_entity_collisions(entity, collision_box);
        out.extend(
            self.get_block_collisions_from_context(
                CollisionContext::with_position(entity.cloned(), pos.y),
                collision_box,
            ),
        );
        out
    }

    /// `getBlockCollisions(entity, collisionBox)`.
    fn get_block_collisions(&self, entity: Option<&Arc<Entity>>, collision_box: &Aabb) -> Vec<VoxelShape> {
        let context = match entity {
            Some(e) => CollisionContext::of_entity(e.clone()),
            None => CollisionContext::empty(),
        };
        self.get_block_collisions_from_context(context, collision_box)
    }

    /// `getBlockAndLiquidCollisions(entity, collisionBox)`.
    fn get_block_and_liquid_collisions(
        &self,
        entity: Option<&Arc<Entity>>,
        collision_box: &Aabb,
    ) -> Vec<VoxelShape> {
        let context = match entity {
            Some(e) => CollisionContext::of_entity_with_fluid(e.clone(), true),
            None => CollisionContext::empty_with_fluid_collisions(),
        };
        self.get_block_collisions_from_context(context, collision_box)
    }

    /// `getBlockCollisionsFromContext(context, collisionBox)`.
    fn get_block_collisions_from_context(
        &self,
        context: CollisionContext,
        collision_box: &Aabb,
    ) -> Vec<VoxelShape> {
        BlockCollisions::with_context(self, context, collision_box, false, |_, shape| shape)
            .collect()
    }

    /// `getCollisionShape(state, pos, context)` — хелпер:
    /// `state.getCollisionShape(this, pos, context)`; level-aware
    /// если доступен `LevelReader`, иначе unleveled.
    fn get_block_collision_shape(
        &self,
        state: &BlockState,
        pos: BlockPos,
        _context: &CollisionContext,
    ) -> VoxelShape {
        if let Some(level) = self.as_level_reader() {
            state.get_collision_shape(level, pos)
        } else {
            state.collision_shape_unleveled()
        }
    }

    /// `borderCollision(entity, box)` — `@Nullable VoxelShape`.
    fn border_collision(&self, entity: &Entity, aabb: &Aabb) -> Option<VoxelShape> {
        let border = self.get_world_border();
        if border.is_inside_close_to_border(entity, aabb) {
            Some(border.get_collision_shape())
        } else {
            None
        }
    }

    /// `clipIncludingBorder(clipContext)`.
    fn clip_including_border(&self, clip_context: &ClipContext) -> BlockHitResult {
        let block_hit = self.clip(clip_context);
        let border = self.get_world_border();
        if border.is_within_bounds_vec(clip_context.from())
            && !border.is_within_bounds_vec(&block_hit.location)
        {
            let vec3 = block_hit.location.subtract_vec(clip_context.from());
            let direction = Direction::get_approximate_nearest(
                vec3.x as f32,
                vec3.y as f32,
                vec3.z as f32,
            );
            let clamped = border.clamp_vec3_to_bound_vec(&block_hit.location);
            BlockHitResult::new_border(
                clamped.clone(),
                direction,
                BlockPos::containing(clamped.x, clamped.y, clamped.z),
                false,
                true,
            )
        } else {
            block_hit
        }
    }

    /// `collidesWithSuffocatingBlock(entity, box)`.
    fn collides_with_suffocating_block(
        &self,
        entity: Option<&Arc<Entity>>,
        aabb: &Aabb,
    ) -> bool {
        let context = match entity {
            Some(e) => CollisionContext::of_entity(e.clone()),
            None => CollisionContext::empty(),
        };
        BlockCollisions::with_context(self, context, aabb, true, |_, shape| shape)
            .any(|s| !s.is_empty())
    }

    /// `findSupportingBlock(entity, box)` — ближайший блок под
    /// сущностью (для stepping-up).
    fn find_supporting_block(&self, entity: &Arc<Entity>, aabb: &Aabb) -> Option<BlockPos> {
        let context = CollisionContext::of_entity(entity.clone());
        let mut best_pos: Option<BlockPos> = None;
        let mut best_dist = f64::MAX;
        let entity_pos = entity.position();
        for block_pos in BlockCollisions::with_context(self, context, aabb, false, |pos, _| {
            pos.immutable()
        }) {
            let d = block_pos.dist_to_center_sqr(entity_pos.x, entity_pos.y, entity_pos.z);
            let better = match &best_pos {
                None => true,
                Some(bp) => {
                    d < best_dist || (d == best_dist && bp.compare_to(&block_pos) == std::cmp::Ordering::Less)
                }
            };
            if better {
                best_pos = Some(block_pos);
                best_dist = d;
            }
        }
        best_pos
    }

    /// `findFreePosition(entity, shape, pos, x, y, z)` —
    /// `Optional<Vec3>`.
    fn find_free_position(
        &self,
        entity: Option<&Arc<Entity>>,
        shape: &VoxelShape,
        pos: &Vec3,
        x: f64,
        y: f64,
        z: f64,
    ) -> Option<Vec3> {
        if shape.is_empty() {
            return None;
        }
        let bounds = shape.bounds()?;
        let aabb = bounds.inflate(x, y, z);
        let mut merged = VoxelShape::empty();
        let border = self.get_world_border();
        for s in self.get_block_collisions(entity, &aabb) {
            if let Some(b) = s.bounds() {
                if !border.is_within_bounds_aabb(&b) {
                    continue;
                }
            }
            for a in s.to_aabbs() {
                let inflated = a.inflate(x / 2.0, y / 2.0, z / 2.0);
                merged = merged.join(&VoxelShape::of(inflated), BooleanOp::Or);
            }
        }
        let voxel_shape1 = shape.join(&merged, BooleanOp::OnlyFirst);
        voxel_shape1.closest_point_to(pos)
    }
}
