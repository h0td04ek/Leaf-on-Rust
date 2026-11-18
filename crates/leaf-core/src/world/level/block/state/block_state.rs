//! Перенос `net.minecraft.world.level.block.state.BlockStateBase`
//! (базовая часть `BlockState`) — состояние блока: ссылка на блок,
//! `StateDefinition` и значения свойств.

use super::state_definition::StateDefinition;
use super::state_holder::StateHolder;
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::world::level::block::Block;
use std::sync::Arc;

/// `BlockState` — `StateHolder<Block, BlockState>`.
///
/// В Java `BlockBehaviour.BlockStateBase` — runtime-тип с кэшем
/// соседних состояний и делегированием в BlockBehaviour; здесь —
/// data-ориентированная модель: блок + значения свойств.
#[derive(Clone)]
pub struct BlockState {
    /// Блок-владелец.
    pub block: Arc<Block>,
    /// Определение состояний блока.
    pub definition: Arc<StateDefinition>,
    /// Значения свойств этого состояния.
    pub values: StateHolder,
    /// `fluidState` — кэшированное состояние жидкости блока
    /// (`BlockStateBase.fluidState`).
    pub fluid_state: super::fluid_state::FluidState,
}

impl BlockState {
    /// `air()` — дефолтное состояние воздуха.
    pub fn air() -> Self {
        // Placeholder — requires Blocks registry
        // For now, return a minimal air state
        Self::new(Arc::new(crate::world::level::block::Block::new(crate::resources::identifier::Identifier::from_namespace_and_path("minecraft", "air").unwrap())), StateHolder::default())
    }

    /// `BlockStateBase` конструктор.
    pub fn new(block: Arc<Block>, values: StateHolder) -> Self {
        let definition = block
            .state_definition()
            .expect("block without state_definition")
            .clone();
        let fluid_state = super::fluid_state::FluidState::empty();
        Self { block, definition, values, fluid_state }
    }

    /// `withFluidState` — установить кэш fluid state (вызывается
    /// при построении `StateDefinition`/`BlockBehaviour`).
    pub fn with_fluid_state(mut self, fluid_state: super::fluid_state::FluidState) -> Self {
        self.fluid_state = fluid_state;
        self
    }

    /// `getFluidState`.
    pub fn get_fluid_state(&self) -> super::fluid_state::FluidState {
        self.fluid_state.clone()
    }

    /// `getLightBlock` — блокировка света.
    pub fn get_light_block(&self) -> i32 {
        15 // Placeholder — requires block properties
    }

    /// `getLightEmission` — излучение света.
    pub fn get_light_emission(&self) -> u8 {
        0 // Placeholder — requires block properties
    }

    /// `useShapeForLightOcclusion` — использовать форму для окклюзии.
    pub fn use_shape_for_light_occlusion(&self) -> bool {
        false // Placeholder — requires block properties
    }

    /// `canOcclude` — может ли блокировать.
    pub fn can_occlude(&self) -> bool {
        true // Placeholder — requires block properties
    }

    /// `getBlock() instanceof LiquidBlock`.
    pub fn is_liquid_block(&self) -> bool {
        self.block.properties.liquid
    }

    /// `blocksMotion` — блок препятствует движению (`Material.blocksMotion`).
    /// До порта VoxelShape/материалов — по флагу `has_collision`.
    pub fn blocks_motion(&self) -> bool {
        self.block.properties.has_collision
    }

    /// `canSurvive(level, pos)` — делегирует в Block.
    pub fn can_survive(&self, level: &dyn crate::world::level::level_reader::LevelReader, pos: crate::core::block_pos::BlockPos) -> bool {
        self.block.can_survive(self, level, pos)
    }

    /// `isAir`.
    pub fn is_air(&self) -> bool {
        self.block.properties.is_air
    }

    /// `getBlock`.
    pub fn block_holder(&self) -> Holder<Block> {
        Holder::Direct((*self.block).clone())
    }

    /// `is(Holder<Block>)` / `is(Block)`.
    pub fn is_block(&self, block: &Block) -> bool {
        self.block.id == block.id
    }

    /// `is(HolderSet<Block>)` — проверка вхождения блока в набор.
    pub fn is(&self, set: &HolderSet<Block>) -> bool {
        match set {
            HolderSet::Direct(list) => list.iter().any(|h| match h {
                Holder::Direct(b) => b.id == self.block.id,
                Holder::Key(k) => k.id() == &self.block.id,
                Holder::Reference(id) => crate::world::level::block::block_registry()
                    .read()
                    .unwrap()
                    .get(*id as usize)
                    .is_some_and(|b| b.id == self.block.id),
            }),
            HolderSet::Named(_) => {
                // Named-теги резолвятся через реестр тегов — без
                // резолва консервативно false (как unresolved holder).
                false
            }
        }
    }

    /// `getValue(Property)` — `Option<StateValue>`.
    pub fn get_value(&self, property: &dyn super::property::PropertyTrait) -> Option<super::property::StateValue> {
        self.values.get(property)
    }

    /// `hasProperty(Property)`.
    pub fn has_property(&self, property: &dyn super::property::PropertyTrait) -> bool {
        self.values.get(property).is_some()
    }

    /// Типизированный `getValue`.
    pub fn get_typed<T: super::property::PropertyValue>(
        &self,
        property: &dyn super::property::PropertyTrait,
    ) -> Option<T> {
        self.values.get_typed(property)
    }

    /// `is(TagKey)` по именованному тегу блоков.
    pub fn is_named_tag(&self, tag_location: &crate::resources::identifier::Identifier) -> bool {
        crate::tags::is_block_in_named_tag(&self.block.id, tag_location)
    }

    /// `is(BlockTags.X)` — короткая форма для тегов блоков.
    pub fn is_block_tag(&self, tag_location: &crate::resources::identifier::Identifier) -> bool {
        self.is_named_tag(tag_location)
    }

    /// `CampfireBlock.isLitCampfire(state)` — campfire/soul_campfire
    /// со свойством `lit=true`.
    pub fn is_lit_campfire(&self) -> bool {
        matches!(
            self.values.get_by_name("lit"),
            Some(super::property::StateValue::Bool(true))
        )
    }

    /// Свойство `open` (doors/fence gates/trapdoors) —
    /// `state.getValue(X.OPEN)` в Java.
    pub fn is_open_property(&self) -> bool {
        matches!(
            self.values.get_by_name("open"),
            Some(super::property::StateValue::Bool(true))
        )
    }

    /// `getShape(level, pos)` — визуальная форма блока.
    /// По умолчанию совпадает с collision shape (большинство
    /// блоков), если у блока задана явная форма — она и есть.
    pub fn get_shape(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
    ) -> crate::world::phys::shapes::VoxelShape {
        self.block_shape(level, pos, false)
    }

    /// `getCollisionShape(level, pos)`.
    pub fn get_collision_shape(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
    ) -> crate::world::phys::shapes::VoxelShape {
        if !self.block.properties.has_collision {
            return crate::world::phys::shapes::VoxelShape::empty();
        }
        self.block_shape(level, pos, true)
    }

    /// Общая форма блока: явная `collision_shape` из свойств,
    /// иначе — полный куб для твёрдых, пусто для air/liquid.
    fn block_shape(
        &self,
        _level: &dyn crate::world::level::level_reader::LevelReader,
        _pos: crate::core::block_pos::BlockPos,
        _collision: bool,
    ) -> crate::world::phys::shapes::VoxelShape {
        if let Some(shape) = &self.block.properties.collision_shape {
            return shape.clone();
        }
        if self.is_air() || self.block.properties.liquid {
            return crate::world::phys::shapes::VoxelShape::empty();
        }
        crate::world::phys::shapes::VoxelShape::block()
    }

    /// `isCollisionShapeFullBlock(level, pos)` —
    /// `Block.isShapeFullBlock(getCollisionShape)`.
    pub fn is_collision_shape_full_block(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
    ) -> bool {
        let shape = self.get_collision_shape(level, pos);
        crate::world::level::block::Block::is_shape_full_block(&shape)
    }

    /// `entityCanStandOn(level, pos, entity)` —
    /// `Block.isFaceFull(collisionShape, UP)`; entity пока не
    /// влияет на форму (CollisionContext.of(entity) — entity-aware
    /// shapes добавим с портом коллизий сущностей).
    pub fn entity_can_stand_on(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
    ) -> bool {
        let shape = self.get_collision_shape(level, pos);
        !shape.is_empty() && shape.max(crate::core::direction::Axis::Y) >= 0.999
    }

    /// `isSolidRender(level, pos)` — full collision cube.
    pub fn is_solid_render(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
    ) -> bool {
        self.is_collision_shape_full_block(level, pos)
    }

    /// `isSolid()` — материал твёрдый (для pathfinding —
    /// full collision shape).
    pub fn is_solid(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
    ) -> bool {
        self.is_solid_render(level, pos)
    }

    /// `BlockBehaviour.isPathfindable(state, type)` — дефолтная
    /// реализация + переопределение через `properties.pathfindable`.
    pub fn is_pathfindable(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
        type_: crate::world::level::pathfinder::path_computation_type::PathComputationType,
    ) -> bool {
        use crate::world::level::pathfinder::path_computation_type::PathComputationType as T;
        if let Some(v) = self.block.properties.pathfindable {
            return v;
        }
        match type_ {
            T::Land | T::Air => !self.is_collision_shape_full_block(level, pos),
            T::Water => {
                let fs = self.get_fluid_state();
                fs.is_in_named_tag(&crate::tags::fluid_tags::FluidTags::water())
                    || fs.is_water()
            }
        }
    }
}

impl PartialEq for BlockState {
    fn eq(&self, other: &Self) -> bool {
        self.block.id == other.block.id && self.values == other.values
    }
}

impl Eq for BlockState {}

impl std::fmt::Debug for BlockState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockState")
            .field("block", &self.block.id)
            .field("values", &self.values)
            .finish()
    }
}

impl std::hash::Hash for BlockState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block.id.hash(state);
        self.values.hash(state);
    }
}
