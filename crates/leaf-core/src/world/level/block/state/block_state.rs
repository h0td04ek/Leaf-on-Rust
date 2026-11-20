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

    /// `canBeReplaced()` — `properties.replaceable` блока.
    pub fn can_be_replaced(&self) -> bool {
        self.block.properties.replaceable
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

    /// Int-значение свойства по имени (`getValue(RedStoneWireBlock.POWER)`
    /// и подобные integer-property).
    pub fn get_int_property(&self, name: &str) -> Option<i32> {
        let prop = self.definition.get_property(name)?;
        match self.values.get(prop.as_ref())? {
            super::property::StateValue::Int(v) => Some(v),
            _ => None,
        }
    }

    /// `state.is(block)` по registry-id (`minecraft:stone` и т.п.).
    pub fn is_block_id(&self, id: &str) -> bool {
        self.block.id.to_string() == id
    }

    /// `getSignal(level, pos, direction)` — `BlockBehaviour.getSignal`
    /// (default 0).
    pub fn get_signal(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
        direction: crate::core::direction::Direction,
    ) -> i32 {
        match &self.block.properties.get_signal {
            Some(f) => f(self, level, pos, direction),
            None => 0,
        }
    }

    /// `getDirectSignal(level, pos, direction)` —
    /// `BlockBehaviour.getDirectSignal` (default 0).
    pub fn get_direct_signal(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
        direction: crate::core::direction::Direction,
    ) -> i32 {
        match &self.block.properties.get_direct_signal {
            Some(f) => f(self, level, pos, direction),
            None => 0,
        }
    }

    /// `updateShape(level, scheduledTickAccess, pos, direction,
    /// neighborPos, neighborState, random)` — `BlockBehaviour.updateShape`
    /// (default: identity).
    #[allow(clippy::too_many_arguments)]
    pub fn update_shape(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        scheduled_tick_access: &dyn crate::world::level::scheduled_tick_access::ScheduledTickAccess,
        pos: crate::core::block_pos::BlockPos,
        direction: crate::core::direction::Direction,
        neighbor_pos: crate::core::block_pos::BlockPos,
        neighbor_state: &BlockState,
        random: std::sync::Arc<dyn crate::random::RandomSource>,
    ) -> BlockState {
        match &self.block.properties.update_shape {
            Some(f) => f(self, level, scheduled_tick_access, pos, direction, neighbor_pos, neighbor_state, random),
            None => self.clone(),
        }
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

    /// `getInteractionShape(level, pos)` — форма для
    /// raytrace-взаимодействия (default = `getShape`,
    /// как в `BlockBehaviour`).
    pub fn get_interaction_shape(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
    ) -> crate::world::phys::shapes::VoxelShape {
        self.get_shape(level, pos)
    }

    /// `getVisualShape(level, pos)` — визуальная форма
    /// (default = `getShape`, как в `BlockBehaviour`).
    pub fn get_visual_shape(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
    ) -> crate::world::phys::shapes::VoxelShape {
        self.get_shape(level, pos)
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

    /// `getCollisionShape` без `level` — для `LevelReader::
    /// block_collisions_in_box`, где `&dyn LevelReader` недоступен
    /// (`Self` unsized). Текущие формы не зависят от level.
    pub fn collision_shape_unleveled(&self) -> crate::world::phys::shapes::VoxelShape {
        if !self.block.properties.has_collision {
            return crate::world::phys::shapes::VoxelShape::empty();
        }
        self.block_shape_static()
    }

    /// Общая форма блока без level-контекста.
    fn block_shape_static(&self) -> crate::world::phys::shapes::VoxelShape {
        if let Some(shape) = &self.block.properties.collision_shape {
            return shape.clone();
        }
        if self.is_air() || self.block.properties.liquid {
            return crate::world::phys::shapes::VoxelShape::empty();
        }
        crate::world::phys::shapes::VoxelShape::block()
    }

    /// `starlight$isConditionallyFullOpaque` — `canOcclude &
    /// useShapeForLightOcclusion` (Moonrise-патч).
    pub fn starlight_is_conditionally_full_opaque(&self) -> bool {
        self.can_occlude() && self.use_shape_for_light_occlusion()
    }

    /// `getOcclusionShape()` без level — форма, блокирующая свет
    /// (без направления). Пустая, если `isEmptyShape`.
    pub fn occlusion_shape_unleveled(&self) -> crate::world::phys::shapes::VoxelShape {
        if !self.can_occlude() || !self.use_shape_for_light_occlusion() {
            return crate::world::phys::shapes::VoxelShape::empty();
        }
        self.block_shape_static()
    }

    /// `getFaceOcclusionShape(direction)` — форма окклюзии грани.
    /// Направление зарезервировано (vanilla различает формы по
    /// направлениям; в текущей модели форм направления не
    /// различаются).
    pub fn face_occlusion_shape_unleveled(
        &self,
        _direction: crate::core::direction::Direction,
    ) -> crate::world::phys::shapes::VoxelShape {
        self.occlusion_shape_unleveled()
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

    /// `hasLargeCollisionShape()` — collision shape не является
    /// полным кубом. Level-free вариант (unleveled-форма).
    pub fn has_large_collision_shape(&self) -> bool {
        !crate::world::level::block::Block::is_shape_full_block(
            &self.collision_shape_unleveled(),
        )
    }

    /// `isSuffocating()` — полный solid-рендер без жидкости
    /// (unleveled; Java-сигнатура `(BlockGetter, BlockPos)`).
    pub fn is_suffocating(&self) -> bool {
        crate::world::level::block::Block::is_shape_full_block(
            &self.collision_shape_unleveled(),
        ) && self.get_fluid_state().is_empty()
    }

    /// `is(Blocks.MOVING_PISTON)` — движущийся поршень.
    pub fn is_moving_piston(&self) -> bool {
        self.is_block_id("minecraft:moving_piston")
    }

    /// `getId`-эквивалент — числовой id состояния в реестре
    /// (для палитр/сериализации).
    pub fn id(&self) -> i32 {
        use crate::world::level::chunk::palette::IdMap;
        super::block_state_registry::block_state_id_map().get_id(self)
    }

    /// `mirror(mirror)` — зеркальная трансформация состояния.
    pub fn mirror(&self, mirror: crate::world::level::block::mirror::Mirror) -> Self {
        crate::world::level::levelgen::structure::structure_piece::state_mirror(
            self.clone(),
            mirror,
        )
    }

    /// `rotate(rotation)` — вращение состояния.
    pub fn rotate(&self, rotation: crate::world::level::block::rotation::Rotation) -> Self {
        crate::world::level::levelgen::structure::structure_piece::state_rotate(
            self.clone(),
            rotation,
        )
    }

    /// `setValue(WATERLOGGED, v)` — установка waterlogged, если
    /// свойство есть.
    pub fn with_waterlogged(&self, waterlogged: bool) -> Self {
        let mut state = self.clone();
        if state.values.get_by_name("waterlogged").is_some() {
            state.values.set_erased(
                "waterlogged",
                super::property::StateValue::Bool(waterlogged),
            );
        }
        state
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

    /// `isSignalSource()` — редстоун-источник.
    pub fn is_signal_source(&self) -> bool {
        self.block.properties.signal_source
    }

    /// `isRedstoneConductor(level, pos)` —
    /// `properties.isRedstoneConductor` или дефолт
    /// `isCollisionShapeFullBlock`.
    pub fn is_redstone_conductor(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
    ) -> bool {
        if let Some(f) = &self.block.properties.is_redstone_conductor {
            f(self, level, pos)
        } else {
            self.is_collision_shape_full_block(level, pos)
        }
    }

    /// `isValidSpawn(level, pos, entityType)` —
    /// `properties.isValidSpawn` или дефолт
    /// `isFaceSturdy(UP) && lightEmission < 14`.
    pub fn is_valid_spawn(
        &self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        pos: crate::core::block_pos::BlockPos,
        entity_type: &crate::entity::entity_type::EntityType,
    ) -> bool {
        if let Some(f) = &self.block.properties.is_valid_spawn {
            f(self, level, pos, entity_type)
        } else {
            level.is_face_sturdy(pos, crate::core::direction::Direction::UP)
                && self.get_light_emission() < 14
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

impl BlockState {

    /// `getExplosionResistance()` — сопротивление взрыву блока
    /// (`Block.getExplosionResistance` с учётом fluid state).
    pub fn get_explosion_resistance(&self) -> f32 {
        self.block.properties.explosion_resistance.unwrap_or(0.0)
            .max(self.fluid_state.get_explosion_resistance())
    }

    /// `isDestroyable()` — `getExplosionResistance() < 3600000.0`.
    pub fn is_destroyable(&self) -> bool {
        self.get_explosion_resistance() < 3600000.0
    }
}
