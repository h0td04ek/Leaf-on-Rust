//! `net.minecraft.world.level.LevelWriter` — запись в мир.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::world::entity::entity::Entity;
use crate::world::level::block::state::block_state::BlockState;

/// `LevelWriter`.
pub trait LevelWriter {
    /// `setBlock(pos, state, flags, recursionLeft)`.
    /// `flags` — `Block.UpdateFlags` bitmask.
    fn set_block(
        &self,
        pos: BlockPos,
        state: BlockState,
        flags: i32,
        recursion_left: i32,
    ) -> bool;

    /// `setBlock(pos, state, flags)` — recursionLeft=512.
    fn set_block_default(&self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
        self.set_block(pos, state, flags, 512)
    }

    /// `removeBlock(pos, movedByPiston)`.
    fn remove_block(&self, pos: BlockPos, moved_by_piston: bool) -> bool;

    /// `destroyBlock(pos, dropBlock, entity, recursionLeft)`.
    fn destroy_block(
        &self,
        pos: BlockPos,
        drop_block: bool,
        entity: Option<Arc<Entity>>,
        recursion_left: i32,
    ) -> bool;

    /// `destroyBlock(pos, dropBlock)` — recursionLeft=512, entity=null.
    fn destroy_block_simple(&self, pos: BlockPos, drop_block: bool) -> bool {
        self.destroy_block(pos, drop_block, None, 512)
    }

    /// `destroyBlock(pos, dropBlock, entity)` — recursionLeft=512.
    fn destroy_block_with_entity(
        &self,
        pos: BlockPos,
        drop_block: bool,
        entity: Option<Arc<Entity>>,
    ) -> bool {
        self.destroy_block(pos, drop_block, entity, 512)
    }

    /// `addFreshEntity(entity)`.
    fn add_fresh_entity(&self, _entity: Arc<Entity>) -> bool {
        false
    }
}
