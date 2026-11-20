//! `net.minecraft.world.level.ClipBlockStateContext` —
//! from/to + предикат целевого состояния для `isBlockInLine`.

use crate::core::block_pos::BlockPos;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::phys::vec3::Vec3;

/// `ClipBlockStateContext`.
pub struct ClipBlockStateContext {
    from: Vec3,
    to: Vec3,
    block: Box<dyn Fn(&BlockState) -> bool + Send + Sync>,
}

impl ClipBlockStateContext {
    /// `new(from, to, predicate)`.
    pub fn new(
        from: Vec3,
        to: Vec3,
        block: impl Fn(&BlockState) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self { from, to, block: Box::new(block) }
    }

    /// `getTo()`.
    pub fn to(&self) -> &Vec3 {
        &self.to
    }

    /// `getFrom()`.
    pub fn from(&self) -> &Vec3 {
        &self.from
    }

    /// `isTargetBlock()` — предикат.
    pub fn is_target_block(&self) -> &dyn Fn(&BlockState) -> bool {
        &*self.block
    }
}

/// Позиция-контекст для `traverseBlocks` (BlockPos как
/// мутабельный курсор — в Java `MutableBlockPos`).
pub type TraversePos = BlockPos;
