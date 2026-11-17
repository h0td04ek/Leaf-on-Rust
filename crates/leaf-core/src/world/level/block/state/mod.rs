//! `world.level.block.state` — система состояний блоков:
//! `Property`, `StateHolder`, `StateDefinition`, `BlockState`,
//! `FluidState`, `pattern` (BlockInWorld и др.).

pub mod property;
pub mod state_holder;
pub mod state_definition;
pub mod block_state;
pub mod fluid_state;
pub mod pattern;

pub use property::{Property, PropertyTrait, PropertyValue, StateValue};
pub use state_holder::StateHolder;
pub use state_definition::StateDefinition;
pub use block_state::BlockState;
pub use fluid_state::FluidState;
pub mod block_state_registry;
pub use block_state_registry::{block_state_by_id, block_state_count, block_state_id_map, register_block_states};
