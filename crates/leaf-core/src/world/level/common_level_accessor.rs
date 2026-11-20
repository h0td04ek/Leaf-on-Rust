//! `net.minecraft.world.level.CommonLevelAccessor` —
/// `EntityGetter + LevelReader + LevelSimulatedRW`.


use crate::core::block_pos::BlockPos;
use crate::world::entity::entity::Entity;
use crate::world::level::level_simulated::LevelSimulatedRw;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::shapes::voxel_shape::VoxelShape;

/// `CommonLevelAccessor`.
///
/// В Java — интерфейс с default-методами, делегирующими в
/// `EntityGetter`/`LevelReader`. Здесь — трейт с теми же
/// default-реализациями поверх `LevelReader`-хуков
/// (`get_entity_collisions`/`is_unobstructed` уже есть в LevelReader
/// через EntityGetter-эквиваленты).
pub trait CommonLevelAccessor: LevelSimulatedRw + LevelReader + crate::world::level::entity_getter::EntityGetter {
    /// `getBlockEntity(pos, type)` — default → `LevelReader`.
    fn common_get_block_entity_typed(
        &self,
        pos: BlockPos,
        type_: &crate::world::level::block::entity::block_entity_type::BlockEntityType,
    ) -> Option<crate::world::level::block::entity::block_entity::BlockEntityRef> {
        crate::world::level::level_reader::LevelReader::get_block_entity(self, pos)
            .filter(|be| be.block_entity_type_id() == &type_.id)
    }

    /// `getEntityCollisions(entity, collisionBox)` — default →
    /// `EntityGetter.super` (в LevelReader — `get_entity_collisions`).
    fn common_get_entity_collisions(
        &self,
        entity: Option<&Entity>,
        collision_box: &Aabb,
    ) -> Vec<VoxelShape> {
        crate::world::level::entity_getter::EntityGetter::get_entity_collisions(self, entity, collision_box)
    }

    /// `isUnobstructed(entity, shape)` — default → `EntityGetter.super`.
    fn common_is_unobstructed(
        &self,
        entity: Option<&Entity>,
        shape: &VoxelShape,
    ) -> bool {
        crate::world::level::entity_getter::EntityGetter::is_unobstructed(self, entity, shape)
    }

    /// `getHeightmapPos(heightmapType, pos)` — default → `LevelReader`.
    fn common_get_heightmap_pos(
        &self,
        heightmap_type: HeightmapType,
        pos: BlockPos,
    ) -> BlockPos {
        crate::world::level::level_reader::LevelReader::get_heightmap_pos(self, heightmap_type, pos)
    }
}
