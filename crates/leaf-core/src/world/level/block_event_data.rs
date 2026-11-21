//! `net.minecraft.world.level.BlockEventData` — record
//! `(pos, block, paramA, paramB)` для block events.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::world::level::block::Block;

/// `BlockEventData`.
#[derive(Clone, Debug)]
pub struct BlockEventData {
    /// `pos`.
    pub pos: BlockPos,
    /// `block`.
    pub block: Arc<Block>,
    /// `paramA`.
    pub param_a: i32,
    /// `paramB`.
    pub param_b: i32,
}

impl BlockEventData {
    /// `new BlockEventData(pos, block, paramA, paramB)`.
    pub fn new(pos: BlockPos, block: Arc<Block>, param_a: i32, param_b: i32) -> Self {
        Self { pos, block, param_a, param_b }
    }
}
