//! `net.minecraft.world.level.levelgen.structure.structures` —
//! конкретные типы структур (vanilla + Leaf).
//!
//! Каждый файл: `*_structure(id, settings) -> Structure` —
//! фабрика `Structure` с `find_generation_point` (аналог Java-
//! наследника) + куски (`StructurePiece`-реализации).

pub mod buried_treasure;
pub mod desert_pyramid;
pub mod end_city;
pub mod igloo;
pub mod jigsaw;
pub mod jungle_temple;
pub mod mineshaft;
pub mod ocean_ruin;
pub mod ruined_portal;
pub mod shipwreck;
pub mod swamp_hut;
pub mod nether_fossil;

use crate::resources::identifier::Identifier;
use crate::world::level::block::state::block_state::BlockState;

/// `Blocks.X.defaultBlockState()` по пути (`"spruce_planks"`).
/// Блок не зарегистрирован → air (тестовое окружение без
/// bootstrap реестра).
pub fn block_state(path: &str) -> BlockState {
    block_state_named(&Identifier::minecraft(path))
}

/// `block_state` по полному identifier.
pub fn block_state_named(id: &Identifier) -> BlockState {
    crate::world::level::block::block_by_identifier(id)
        .and_then(|b| b.default_block_state())
        .unwrap_or_else(BlockState::air)
}

/// `state.setValue(Property, value)` для строковых свойств
/// (`facing`, `shape`, `half` и т.п.).
pub fn with_string_property(mut state: BlockState, name: &str, value: &str) -> BlockState {
    state.values.set_erased(name, crate::world::level::block::state::property::StateValue::Str(value.to_string()));
    state
}

/// `state.setValue(BooleanProperty, value)`.
pub fn with_bool_property(mut state: BlockState, name: &str, value: bool) -> BlockState {
    state.values.set_erased(name, crate::world::level::block::state::property::StateValue::Bool(value));
    state
}

/// `state.setValue(IntegerProperty, value)`.
pub fn with_int_property(mut state: BlockState, name: &str, value: i32) -> BlockState {
    state.values.set_erased(name, crate::world::level::block::state::property::StateValue::Int(value));
    state
}
