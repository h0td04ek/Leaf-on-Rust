//! Перенос `RuleTest` + всех наследников
//! (`AlwaysTrueTest`, `BlockMatchTest`, `BlockStateMatchTest`,
//! `RandomBlockMatchTest`, `RandomBlockStateMatchTest`,
//! `TagMatchTest`, `BlockTagMatchTest`,
//! `RandomBlockTagMatchTest`).

use crate::resources::identifier::Identifier;
use crate::random::RandomSource;
use crate::tags::is_block_in_named_tag;
use crate::world::level::block::Block;
use crate::world::level::block::state::block_state::BlockState;
use std::sync::Arc;

/// `RuleTest` — предикат на BlockState.
pub trait RuleTest: Send + Sync {
    /// `test(state, random)`.
    fn test(&self, state: &BlockState, random: &mut dyn RandomSource) -> bool;
}

/// `AlwaysTrueTest`.
pub struct AlwaysTrueTest;

impl RuleTest for AlwaysTrueTest {
    fn test(&self, _state: &BlockState, _random: &mut dyn RandomSource) -> bool {
        true
    }
}

/// `BlockMatchTest` — `state.is(block)`.
pub struct BlockMatchTest {
    /// `block`.
    pub block: Arc<Block>,
}

impl BlockMatchTest {
    /// `new(block)`.
    pub fn new(block: Arc<Block>) -> Self {
        Self { block }
    }
}

impl RuleTest for BlockMatchTest {
    fn test(&self, state: &BlockState, _random: &mut dyn RandomSource) -> bool {
        state.is_block(&self.block)
    }
}

/// `BlockStateMatchTest` — `state == blockState`.
pub struct BlockStateMatchTest {
    /// `blockState`.
    pub block_state: BlockState,
}

impl BlockStateMatchTest {
    /// `new(blockState)`.
    pub fn new(block_state: BlockState) -> Self {
        Self { block_state }
    }
}

impl RuleTest for BlockStateMatchTest {
    fn test(&self, state: &BlockState, _random: &mut dyn RandomSource) -> bool {
        state == &self.block_state
    }
}

/// `RandomBlockMatchTest` — `state.is(block) &&
/// random.nextFloat() < probability`.
pub struct RandomBlockMatchTest {
    /// `block`.
    pub block: Arc<Block>,
    /// `probability`.
    pub probability: f32,
}

impl RandomBlockMatchTest {
    /// `new(block, probability)`.
    pub fn new(block: Arc<Block>, probability: f32) -> Self {
        Self { block, probability }
    }
}

impl RuleTest for RandomBlockMatchTest {
    fn test(&self, state: &BlockState, random: &mut dyn RandomSource) -> bool {
        state.is_block(&self.block) && random.next_float() < self.probability
    }
}

/// `RandomBlockStateMatchTest`.
pub struct RandomBlockStateMatchTest {
    /// `blockState`.
    pub block_state: BlockState,
    /// `probability`.
    pub probability: f32,
}

impl RandomBlockStateMatchTest {
    /// `new(blockState, probability)`.
    pub fn new(block_state: BlockState, probability: f32) -> Self {
        Self { block_state, probability }
    }
}

impl RuleTest for RandomBlockStateMatchTest {
    fn test(&self, state: &BlockState, random: &mut dyn RandomSource) -> bool {
        state == &self.block_state && random.next_float() < self.probability
    }
}

/// `TagMatchTest` — `state.is(tag)`.
pub struct TagMatchTest {
    /// `tag`.
    pub tag: Identifier,
}

impl TagMatchTest {
    /// `new(tag)`.
    pub fn new(tag: Identifier) -> Self {
        Self { tag }
    }
}

impl RuleTest for TagMatchTest {
    fn test(&self, state: &BlockState, _random: &mut dyn RandomSource) -> bool {
        is_block_in_named_tag(&state.block.id, &self.tag)
    }
}

/// `BlockTagMatchTest` — то же через BlockTags-константу.
pub struct BlockTagMatchTest {
    /// `tag`.
    pub tag: Identifier,
}

impl RuleTest for BlockTagMatchTest {
    fn test(&self, state: &BlockState, _random: &mut dyn RandomSource) -> bool {
        is_block_in_named_tag(&state.block.id, &self.tag)
    }
}

/// `RandomBlockTagMatchTest` — tag + probability.
pub struct RandomBlockTagMatchTest {
    /// `tag`.
    pub tag: Identifier,
    /// `probability`.
    pub probability: f32,
}

impl RuleTest for RandomBlockTagMatchTest {
    fn test(&self, state: &BlockState, random: &mut dyn RandomSource) -> bool {
        is_block_in_named_tag(&state.block.id, &self.tag) && random.next_float() < self.probability
    }
}

use crate::tags::block_tags::BlockTags;
