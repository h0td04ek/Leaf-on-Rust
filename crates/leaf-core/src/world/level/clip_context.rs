//! `net.minecraft.world.level.ClipContext` — контекст рейкаста:
//! from/to + режимы блоков/жидкостей + `CollisionContext`.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::world::entity::entity::Entity;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::shapes::collision_context::CollisionContext;
use crate::world::phys::shapes::voxel_shape::VoxelShape;
use crate::world::phys::Vec3;

/// `ClipContext.Block` — какой шейп использовать для блоков.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipBlockMode {
    /// `COLLIDER` — `getCollisionShape`.
    Collider,
    /// `OUTLINE` — `getShape`.
    Outline,
    /// `VISUAL` — `getVisualShape`.
    Visual,
    /// `FALLDAMAGE_RESETTING` — `Shapes.block()` для
    /// `FALL_DAMAGE_RESETTING`-блоков и portal-edge кейсов.
    FallDamageResetting,
}

impl ClipBlockMode {
    /// `get(state, level, pos, collisionContext)` —
    /// `ShapeGetter.get` → `VoxelShape`.
    ///
    /// Наши `BlockState`-формы пока не принимают `CollisionContext`
    /// (entity-aware shapes — при порте коллизий сущностей).
    pub fn shape(
        self,
        state: &BlockState,
        level: &dyn LevelReader,
        pos: BlockPos,
    ) -> VoxelShape {
        match self {
            ClipBlockMode::Collider => state.get_collision_shape(level, pos),
            ClipBlockMode::Outline => state.get_shape(level, pos),
            ClipBlockMode::Visual => state.get_visual_shape(level, pos),
            ClipBlockMode::FallDamageResetting => {
                // `state.is(FALL_DAMAGE_RESETTING)` → block();
                // portal-edge + PLAYER + gameRule-ветки требуют
                // ServerLevel/EntityType — до их порта возвращаем
                // collision-семантику пустого шейпа, кроме тега.
                if state.is_named_tag(
                    &crate::tags::block_tags::BlockTags::fall_damage_resetting(),
                ) {
                    VoxelShape::block()
                } else {
                    VoxelShape::empty()
                }
            }
        }
    }
}

/// `ClipContext.Fluid` — какие жидкости «подбираются» рейкастом.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipFluidMode {
    /// `NONE` — никогда.
    None,
    /// `SOURCE_ONLY` — только `isSource`.
    SourceOnly,
    /// `ANY` — любая непустая жидкость.
    Any,
    /// `WATER` — `FluidTags.WATER`.
    Water,
}

impl ClipFluidMode {
    /// `canPick(state)`.
    pub fn can_pick(self, state: &FluidState) -> bool {
        match self {
            ClipFluidMode::None => false,
            ClipFluidMode::SourceOnly => state.fluid.is_source_state(state),
            ClipFluidMode::Any => !state.is_empty(),
            ClipFluidMode::Water => state.is_in_named_tag(
                &crate::tags::fluid_tags::FluidTags::water(),
            ),
        }
    }
}

/// `ClipContext`.
#[derive(Clone)]
pub struct ClipContext {
    from: Vec3,
    to: Vec3,
    block: ClipBlockMode,
    /// `fluid` — public в Paper (`public final`).
    pub fluid: ClipFluidMode,
    collision_context: CollisionContext,
}

impl ClipContext {
    /// `new(from, to, block, fluid, entity)` —
    /// `entity == null` → `CollisionContext.empty()`
    /// (CraftBukkit-семантика: `empty()` = WITHOUT_FLUID_COLLISIONS).
    pub fn new(
        from: Vec3,
        to: Vec3,
        block: ClipBlockMode,
        fluid: ClipFluidMode,
        entity: Option<Arc<Entity>>,
    ) -> Self {
        let collision_context = match entity {
            Some(e) => CollisionContext::of_entity(e),
            None => CollisionContext::empty(),
        };
        Self { from, to, block, fluid, collision_context }
    }

    /// `new` с явным `CollisionContext`.
    pub fn with_collision_context(
        from: Vec3,
        to: Vec3,
        block: ClipBlockMode,
        fluid: ClipFluidMode,
        collision_context: CollisionContext,
    ) -> Self {
        Self { from, to, block, fluid, collision_context }
    }

    /// `getTo()`.
    pub fn to(&self) -> &Vec3 {
        &self.to
    }

    /// `getFrom()`.
    pub fn from(&self) -> &Vec3 {
        &self.from
    }

    /// `getBlock()`.
    pub fn block(&self) -> ClipBlockMode {
        self.block
    }

    /// `getCollisionContext()`.
    pub fn collision_context(&self) -> &CollisionContext {
        &self.collision_context
    }

    /// `getBlockShape(state, level, pos)`.
    pub fn block_shape(
        &self,
        state: &BlockState,
        level: &dyn LevelReader,
        pos: BlockPos,
    ) -> VoxelShape {
        self.block.shape(state, level, pos)
    }

    /// `getFluidShape(state, level, pos)` — `canPick` →
    /// `state.getShape`, иначе `Shapes.empty()`.
    pub fn fluid_shape(
        &self,
        state: &FluidState,
        level: &dyn LevelReader,
        pos: BlockPos,
    ) -> VoxelShape {
        if self.fluid.can_pick(state) {
            state.get_shape(level, pos)
        } else {
            VoxelShape::empty()
        }
    }
}
