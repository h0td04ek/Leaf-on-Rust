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
}

impl BlockState {
    /// `BlockStateBase` конструктор.
    pub fn new(block: Arc<Block>, values: StateHolder) -> Self {
        let definition = block
            .state_definition()
            .expect("block without state_definition")
            .clone();
        Self { block, definition, values }
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
