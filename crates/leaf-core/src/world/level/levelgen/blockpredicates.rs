//! Перенос `net.minecraft.world.level.levelgen.blockpredicates.BlockPredicate`
//! — предикаты по `WorldGenLevel`/`BlockPos` (14 вариантов + фабрики).

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::core::direction::Direction;
use crate::core::vec3i::Vec3i;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::tags::TagKey;
use crate::world::level::block::Block;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::Fluid;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::aabb::Aabb;

/// `BlockPredicate` — tagged union предикатов генерации.
#[derive(Clone)]
pub enum BlockPredicate {
    /// `AllOfPredicate`.
    AllOf(Vec<BlockPredicate>),
    /// `AnyOfPredicate`.
    AnyOf(Vec<BlockPredicate>),
    /// `MatchingBlocksPredicate`.
    MatchingBlocks { offset: Vec3i, blocks: HolderSet<Block> },
    /// `MatchingBlockTagPredicate`.
    MatchingBlockTag { offset: Vec3i, tag: TagKey<Block> },
    /// `MatchingFluidsPredicate`.
    MatchingFluids { offset: Vec3i, fluids: HolderSet<Fluid> },
    /// `NotPredicate`.
    Not(Box<BlockPredicate>),
    /// `ReplaceablePredicate`.
    Replaceable { offset: Vec3i },
    /// `WouldSurvivePredicate`.
    WouldSurvive { offset: Vec3i, state: BlockState },
    /// `HasSturdyFacePredicate`.
    HasSturdyFace { offset: Vec3i, direction: Direction },
    /// `SolidPredicate`.
    Solid { offset: Vec3i },
    /// `InsideWorldBoundsPredicate`.
    InsideWorldBounds { offset: Vec3i },
    /// `TrueBlockPredicate`.
    True,
    /// `UnobstructedPredicate`.
    Unobstructed { offset: Vec3i },
}

impl BlockPredicate {
    /// `allOf(predicates)`.
    pub fn all_of(predicates: Vec<BlockPredicate>) -> Self {
        Self::AllOf(predicates)
    }

    /// `anyOf(predicates)`.
    pub fn any_of(predicates: Vec<BlockPredicate>) -> Self {
        Self::AnyOf(predicates)
    }

    /// `matchesBlocks(offset, blocks)` — `HolderSet.direct`.
    pub fn matches_blocks(offset: Vec3i, blocks: Vec<Holder<Block>>) -> Self {
        Self::MatchingBlocks { offset, blocks: HolderSet::Direct(blocks) }
    }

    /// `matchesBlocks(blocks)` — offset ZERO.
    pub fn matches_blocks_zero(blocks: Vec<Holder<Block>>) -> Self {
        Self::matches_blocks(Vec3i::ZERO, blocks)
    }

    /// `matchesTag(offset, tag)`.
    pub fn matches_tag(offset: Vec3i, tag: TagKey<Block>) -> Self {
        Self::MatchingBlockTag { offset, tag }
    }

    /// `matchesFluids(offset, fluids)`.
    pub fn matches_fluids(offset: Vec3i, fluids: HolderSet<Fluid>) -> Self {
        Self::MatchingFluids { offset, fluids }
    }

    /// `not(predicate)`.
    pub fn not(predicate: BlockPredicate) -> Self {
        Self::Not(Box::new(predicate))
    }

    /// `replaceable(offset)`.
    pub fn replaceable(offset: Vec3i) -> Self {
        Self::Replaceable { offset }
    }

    /// `wouldSurvive(state, offset)`.
    pub fn would_survive(state: BlockState, offset: Vec3i) -> Self {
        Self::WouldSurvive { offset, state }
    }

    /// `hasSturdyFace(offset, direction)`.
    pub fn has_sturdy_face(offset: Vec3i, direction: Direction) -> Self {
        Self::HasSturdyFace { offset, direction }
    }

    /// `solid(offset)`.
    pub fn solid(offset: Vec3i) -> Self {
        Self::Solid { offset }
    }

    /// `insideWorld(offset)`.
    pub fn inside_world(offset: Vec3i) -> Self {
        Self::InsideWorldBounds { offset }
    }

    /// `alwaysTrue()`.
    pub fn always_true() -> Self {
        Self::True
    }

    /// `unobstructed(offset)`.
    pub fn unobstructed(offset: Vec3i) -> Self {
        Self::Unobstructed { offset }
    }

    /// `test(worldGenLevel, pos)` — `BiPredicate`.
    /// `WorldGenLevel.getBlockState` в Java не бывает null —
    /// отсутствующий чанк трактуем как false (консервативно).
    pub fn test(&self, level: &dyn LevelReader, pos: BlockPos) -> bool {
        match self {
            BlockPredicate::AllOf(predicates) => {
                predicates.iter().all(|p| p.test(level, pos))
            }
            BlockPredicate::AnyOf(predicates) => {
                predicates.iter().any(|p| p.test(level, pos))
            }
            BlockPredicate::Not(predicate) => !predicate.test(level, pos),
            BlockPredicate::True => true,
            BlockPredicate::InsideWorldBounds { offset } => {
                let p = pos.offset(offset.x, offset.y, offset.z);
                level.is_inside_build_height(p.y)
            }
            BlockPredicate::HasSturdyFace { offset, direction } => {
                let p = pos.offset(offset.x, offset.y, offset.z);
                // Java: `level.getBlockState(p).isFaceSturdy(level, p, dir)`.
                // is_face_sturdy читает состояние сама.
                let _ = level.get_block_state(p);
                level.is_face_sturdy(p, *direction)
            }
            BlockPredicate::Unobstructed { offset } => {
                let p = pos.offset(offset.x, offset.y, offset.z);
                // `Shapes.block().move(pos)` — куб 1×1×1 в позиции.
                let aabb = Aabb::new(
                    p.x as f64,
                    p.y as f64,
                    p.z as f64,
                    p.x as f64 + 1.0,
                    p.y as f64 + 1.0,
                    p.z as f64 + 1.0,
                );
                level.no_collision(&aabb)
            }
            BlockPredicate::WouldSurvive { offset, state } => {
                let p = pos.offset(offset.x, offset.y, offset.z);
                state.can_survive(level, p)
            }
            // StateTestingPredicate-варианты.
            BlockPredicate::MatchingBlocks { offset, blocks } => {
                level
                    .get_block_state(pos.offset(offset.x, offset.y, offset.z))
                    .map(|s| s.is(blocks))
                    .unwrap_or(false)
            }
            BlockPredicate::MatchingBlockTag { offset, tag } => {
                level
                    .get_block_state(pos.offset(offset.x, offset.y, offset.z))
                    .map(|s| s.is_named_tag(&tag.location))
                    .unwrap_or(false)
            }
            BlockPredicate::MatchingFluids { offset, fluids } => {
                level
                    .get_block_state(pos.offset(offset.x, offset.y, offset.z))
                    .map(|s| s.get_fluid_state().is(fluids))
                    .unwrap_or(false)
            }
            BlockPredicate::Replaceable { offset } => {
                level
                    .get_block_state(pos.offset(offset.x, offset.y, offset.z))
                    .map(|s| s.can_be_replaced())
                    .unwrap_or(false)
            }
            BlockPredicate::Solid { offset } => {
                level
                    .get_block_state(pos.offset(offset.x, offset.y, offset.z))
                    .map(|s| s.block.properties.has_collision)
                    .unwrap_or(false)
            }
        }
    }
}

fn block_holder_key(path: &str) -> Holder<Block> {
    Holder::Key(ResourceKey::create_with_registry_name(
        Identifier::minecraft("block"),
        Identifier::minecraft(path),
    ))
}

/// `ONLY_IN_AIR_PREDICATE` — `matchesBlocks(Blocks.AIR)`.
pub fn only_in_air_predicate() -> BlockPredicate {
    BlockPredicate::matches_blocks_zero(vec![block_holder_key("air")])
}

/// `ONLY_IN_AIR_OR_WATER_PREDICATE`.
pub fn only_in_air_or_water_predicate() -> BlockPredicate {
    BlockPredicate::matches_blocks_zero(vec![
        block_holder_key("air"),
        block_holder_key("water"),
    ])
}
