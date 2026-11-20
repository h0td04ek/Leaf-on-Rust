//! `net.minecraft.world.phys.shapes.CollisionContext` +
//! `EntityCollisionContext` — контекст коллизий (entity-aware
//! shapes, fluids, placement).
//!
//! В Java — интерфейс + protected-класс с `Empty`-singleton'ами.
//! Здесь — единая структура с флагом `empty` (семантика
//! `EntityCollisionContext.Empty`: `isAbove` → `canAscend`).

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::direction::Axis;
use crate::world::entity::entity::Entity;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::shapes::voxel_shape::VoxelShape;
use crate::world::phys::Vec3;

/// `CollisionContext` — контекст для entity-aware collision shapes.
#[derive(Clone)]
pub struct CollisionContext {
    /// `Empty`-вариант (`isAbove` → `canAscend`).
    empty: bool,
    /// `placement`.
    placement: bool,
    /// `alwaysCollideWithFluid`.
    always_collide_with_fluid: bool,
    /// `entity` (null → None).
    entity: Option<Arc<Entity>>,
}

impl CollisionContext {
    /// `CollisionContext.empty()` — `WITHOUT_FLUID_COLLISIONS`.
    pub fn empty() -> Self {
        Self {
            empty: true,
            placement: false,
            always_collide_with_fluid: false,
            entity: None,
        }
    }

    /// `CollisionContext.emptyWithFluidCollisions()`.
    pub fn empty_with_fluid_collisions() -> Self {
        Self {
            empty: true,
            placement: false,
            always_collide_with_fluid: true,
            entity: None,
        }
    }

    /// `CollisionContext.of(entity)` — minecart-ветка требует
    /// `AbstractMinecart.useExperimentalMovement`; до порта
    /// minecart — обычный `EntityCollisionContext(entity,false,false)`.
    pub fn of_entity(entity: Arc<Entity>) -> Self {
        Self {
            empty: false,
            placement: false,
            always_collide_with_fluid: false,
            entity: Some(entity),
        }
    }

    /// `CollisionContext.of(entity, alwaysCollideWithFluid)`.
    pub fn of_entity_with_fluid(entity: Arc<Entity>, always_collide_with_fluid: bool) -> Self {
        Self {
            empty: false,
            placement: false,
            always_collide_with_fluid,
            entity: Some(entity),
        }
    }

    /// `CollisionContext.placementContext(player)` —
    /// `EntityCollisionContext(descending, true, y,
    /// mainHandItem, false, player)`.
    pub fn placement_context(player: Option<Arc<Entity>>) -> Self {
        Self {
            empty: false,
            placement: true,
            always_collide_with_fluid: false,
            entity: player,
        }
    }

    /// `CollisionContext.withPosition(entity, bottom)`.
    pub fn with_position(entity: Option<Arc<Entity>>, _bottom: f64) -> Self {
        // Gale: bottom больше не хранится — `isAbove` читает
        // entity.getY() live.
        Self {
            empty: false,
            placement: false,
            always_collide_with_fluid: false,
            entity,
        }
    }

    /// `isDescending()`.
    pub fn is_descending(&self) -> bool {
        self.entity.as_ref().is_some_and(|e| e.is_descending())
    }

    /// `isAbove(shape, pos, canAscend)`.
    /// Empty → `canAscend`; иначе `entityY > pos.y + shape.max(Y) - eps`.
    pub fn is_above(&self, shape: &VoxelShape, pos: BlockPos, can_ascend: bool) -> bool {
        if self.empty {
            return can_ascend;
        }
        let entity_y = self
            .entity
            .as_ref()
            .map(|e| e.get_y())
            .unwrap_or(f64::NEG_INFINITY);
        entity_y > pos.get_y() as f64 + shape.max(Axis::Y) - 1.0e-5
    }

    /// `isHoldingItem(item)` — `LivingEntity.getMainHandItem().is(item)`;
    /// не-игрок → `ItemStack.EMPTY.is(item)` = false.
    pub fn is_holding_item(&self, item_id: &crate::resources::identifier::Identifier) -> bool {
        self.entity
            .as_ref()
            .and_then(|e| e.get_main_hand_item())
            .is_some_and(|stack| stack.is(item_id))
    }

    /// `alwaysCollideWithFluid()`.
    pub fn always_collide_with_fluid(&self) -> bool {
        self.always_collide_with_fluid
    }

    /// `canStandOnFluid(fluid1, fluid2)` —
    /// `LivingEntity.canStandOnFluid(fluid2) && !fluid1.isSame(fluid2)`.
    pub fn can_stand_on_fluid(&self, fluid1: &FluidState, fluid2: &FluidState) -> bool {
        self.entity
            .as_ref()
            .is_some_and(|e| e.is_living_entity() && e.can_stand_on_fluid(fluid2))
            && !fluid1.is_same_type(fluid2)
    }

    /// `getCollisionShape(state, collisionGetter, pos)` —
    /// делегирует в `state.getCollisionShape` (entity-aware
    /// формы — когда collision_shape будет принимать контекст).
    pub fn get_collision_shape(
        &self,
        state: &BlockState,
        level: &dyn LevelReader,
        pos: BlockPos,
    ) -> VoxelShape {
        state.get_collision_shape(level, pos)
    }

    /// `getCollisionShape(state, collisionGetter, pos)` для
    /// `CollisionGetter`/`BlockGetter`-контекстов (entity-aware
    /// формы — когда collision_shape будет принимать контекст).
    /// Если getter может дать `LevelReader` — используем его,
    /// иначе unleveled-форму.
    pub fn get_collision_shape_bg(
        &self,
        state: &BlockState,
        getter: &dyn crate::world::level::block_getter::BlockGetter,
        pos: BlockPos,
    ) -> VoxelShape {
        if let Some(level) = getter.as_level_reader() {
            state.get_collision_shape(level, pos)
        } else {
            state.collision_shape_unleveled()
        }
    }

    /// `isPlacement()`.
    pub fn is_placement(&self) -> bool {
        self.placement
    }

    /// `getEntity()`.
    pub fn entity(&self) -> Option<&Arc<Entity>> {
        self.entity.as_ref()
    }
}
