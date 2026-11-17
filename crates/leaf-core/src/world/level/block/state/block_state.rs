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
