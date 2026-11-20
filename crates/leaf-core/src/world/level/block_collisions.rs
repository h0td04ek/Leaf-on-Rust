//! `net.minecraft.world.level.BlockCollisions<T>` — итератор
//! коллизионных форм блоков в AABB (vanilla `AbstractIterator`).
//!
//! Порт `computeNext`: курсор `Cursor3D` по расширенному боксу,
//! кэш последнего chunk-getter'а, фильтры suffocating/large-shape/
//! moving-piston, full-block fast path и voxel-пересечение.

use std::sync::Arc;

use crate::core::block_pos::{BlockPos, MutableBlockPos};
use crate::core::cursor3d::{Cursor3D, TYPE_CORNER, TYPE_EDGE, TYPE_FACE};
use crate::core::section_pos::SectionPos;
use crate::world::entity::entity::Entity;
use crate::world::level::block_getter::BlockGetter;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::collision_getter::CollisionGetter;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::shapes::collision_context::CollisionContext;
use crate::world::phys::shapes::voxel_shape::{join_is_not_empty, single_aabb_representation, BooleanOp, VoxelShape};

/// `BlockCollisions<T>` — `AbstractIterator<T>`.
///
/// `F` — аналог `BiFunction<BlockPos.MutableBlockPos, VoxelShape, T>`
/// (`resultProvider`).
pub struct BlockCollisions<'a, G, T, F>
where
    G: CollisionGetter + ?Sized,
    F: FnMut(&mut MutableBlockPos, VoxelShape) -> T,
{
    box_: Aabb,
    context: CollisionContext,
    cursor: Cursor3D,
    pos: MutableBlockPos,
    entity_shape: VoxelShape,
    collision_getter: &'a G,
    only_suffocating_blocks: bool,
    cached_block_getter: Option<(i64, &'a (dyn BlockGetter + 'a))>,
    result_provider: F,
}

impl<'a, G, T, F> BlockCollisions<'a, G, T, F>
where
    G: CollisionGetter + ?Sized,
    F: FnMut(&mut MutableBlockPos, VoxelShape) -> T,
{
    /// `new BlockCollisions(collisionGetter, entity, box,
    /// onlySuffocatingBlocks, resultProvider)` — `entity == null`
    /// → `CollisionContext.empty()`.
    pub fn new(
        collision_getter: &'a G,
        entity: Option<Arc<Entity>>,
        aabb: &Aabb,
        only_suffocating_blocks: bool,
        result_provider: F,
    ) -> Self {
        let context = match entity {
            Some(e) => CollisionContext::of_entity(e),
            None => CollisionContext::empty(),
        };
        Self::with_context(collision_getter, context, aabb, only_suffocating_blocks, result_provider)
    }

    /// `new BlockCollisions(collisionGetter, context, box,
    /// onlySuffocatingBlocks, resultProvider)`.
    pub fn with_context(
        collision_getter: &'a G,
        context: CollisionContext,
        aabb: &Aabb,
        only_suffocating_blocks: bool,
        result_provider: F,
    ) -> Self {
        let entity_shape = VoxelShape::of(aabb.clone());
        let pos = MutableBlockPos::new(0, 0, 0);
        // cursor bounds: floor(min - eps) - 1 .. floor(max + eps) + 1
        let floor = |v: f64| v.floor() as i32;
        let i = floor(aabb.min_x - 1.0E-7) - 1;
        let i1 = floor(aabb.max_x + 1.0E-7) + 1;
        let i2 = floor(aabb.min_y - 1.0E-7) - 1;
        let i3 = floor(aabb.max_y + 1.0E-7) + 1;
        let i4 = floor(aabb.min_z - 1.0E-7) - 1;
        let i5 = floor(aabb.max_z + 1.0E-7) + 1;
        Self {
            box_: aabb.clone(),
            context,
            pos,
            entity_shape,
            collision_getter,
            cursor: Cursor3D::new(i, i2, i4, i1, i3, i5),
            only_suffocating_blocks,
            cached_block_getter: None,
            result_provider,
        }
    }

    /// `getChunk(x, z)` — кэшированный `BlockGetter` чанка.
    fn get_chunk(&mut self, x: i32, z: i32) -> Option<&'a (dyn BlockGetter + 'a)> {
        let section_pos_x = SectionPos::block_to_section_coord(x);
        let section_pos_z = SectionPos::block_to_section_coord(z);
        let packed_chunk_pos = ChunkPos::as_long_static(section_pos_x, section_pos_z);
        if let Some((cached_pos, getter)) = self.cached_block_getter {
            if cached_pos == packed_chunk_pos {
                return Some(getter);
            }
        }
        let chunk = self
            .collision_getter
            .get_chunk_for_collisions(section_pos_x, section_pos_z);
        self.cached_block_getter = chunk.map(|g| (packed_chunk_pos, g));
        chunk
    }

    /// `computeNext()`.
    fn compute_next(&mut self) -> Option<T> {
        while self.cursor.advance() {
            let i = self.cursor.next_x();
            let i1 = self.cursor.next_y();
            let i2 = self.cursor.next_z();
            let next_type = self.cursor.get_next_type();
            if next_type == TYPE_CORNER {
                continue;
            }
            let Some(chunk) = self.get_chunk(i, i2) else {
                continue;
            };
            self.pos.set(i, i1, i2);
            let block_pos = self.pos.immutable();
            let Some(block_state) = chunk.get_block_state(block_pos) else {
                continue;
            };
            // условие suffocating: `!onlySuffocating || isSuffocating`.
            if self.only_suffocating_blocks && !block_state.is_suffocating() {
                continue;
            }
            if next_type == TYPE_FACE && block_state.has_large_collision_shape() {
                continue;
            }
            if next_type == TYPE_EDGE && block_state.is_moving_piston() {
                continue;
            }
            let collision_shape = self
                .context
                .get_collision_shape_bg(&block_state, chunk, block_pos);
            // `collisionShape == Shapes.block()` — identity-check в
            // Java; эквивалент — одиночный полный куб.
            let is_full_block = single_aabb_representation(&collision_shape)
                .map(|a| {
                    (a.min_x - 0.0).abs() < f64::EPSILON
                        && (a.min_y - 0.0).abs() < f64::EPSILON
                        && (a.min_z - 0.0).abs() < f64::EPSILON
                        && (a.max_x - 1.0).abs() < f64::EPSILON
                        && (a.max_y - 1.0).abs() < f64::EPSILON
                        && (a.max_z - 1.0).abs() < f64::EPSILON
                })
                .unwrap_or(false);
            if is_full_block {
                if self.box_.intersects_coords(
                    i as f64,
                    i1 as f64,
                    i2 as f64,
                    i as f64 + 1.0,
                    i1 as f64 + 1.0,
                    i2 as f64 + 1.0,
                ) {
                    let moved = collision_shape.move_shape(i as f64, i1 as f64, i2 as f64);
                    return Some((self.result_provider)(&mut self.pos, moved));
                }
            } else {
                let voxel_shape =
                    collision_shape.move_shape(i as f64, i1 as f64, i2 as f64);
                if !voxel_shape.is_empty()
                    && join_is_not_empty(&voxel_shape, &self.entity_shape, BooleanOp::And)
                {
                    return Some((self.result_provider)(&mut self.pos, voxel_shape));
                }
            }
        }
        None
    }
}

impl<'a, G, T, F> Iterator for BlockCollisions<'a, G, T, F>
where
    G: CollisionGetter + ?Sized,
    F: FnMut(&mut MutableBlockPos, VoxelShape) -> T,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.compute_next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Простейший CollisionGetter для теста курсора/итерации:
    // все чанки пустые -> итератор сразу заканчивается.
    struct EmptyGetter;

    impl crate::world::level::level_height_accessor::LevelHeightAccessor for EmptyGetter {
        fn get_min_y(&self) -> i32 { -64 }
        fn get_height(&self) -> i32 { 384 }
    }

    impl BlockGetter for EmptyGetter {
        fn get_block_entity(
            &self,
            _pos: BlockPos,
        ) -> Option<Arc<dyn crate::world::level::block::entity::block_entity::BlockEntityAccess>> {
            None
        }
        fn get_block_state(&self, _pos: BlockPos) -> Option<crate::world::level::block::state::block_state::BlockState> {
            None
        }
        fn get_fluid_state(&self, _pos: BlockPos) -> Option<crate::world::level::block::state::fluid_state::FluidState> {
            None
        }
    }

    impl CollisionGetter for EmptyGetter {
        fn get_world_border(&self) -> &crate::world::level::border::world_border::WorldBorder {
            unreachable!()
        }
        fn get_chunk_for_collisions(&self, _x: i32, _z: i32) -> Option<&dyn BlockGetter> {
            None
        }
        fn get_entity_collisions(
            &self,
            _entity: Option<&Arc<Entity>>,
            _collision_box: &Aabb,
        ) -> Vec<VoxelShape> {
            Vec::new()
        }
    }

    #[test]
    fn block_collisions_empty_world() {
        let g = EmptyGetter;
        let aabb = Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let count = BlockCollisions::new(&g, None, &aabb, false, |_, s| s)
            .count();
        assert_eq!(count, 0);
    }
}
