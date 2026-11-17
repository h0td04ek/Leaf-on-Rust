//! Перенос `net.minecraft.world.level.block.state.pattern` —
//! предикаты/паттерны состояний блока в мире.

pub mod block_in_world;
pub use block_in_world::{has_state, BlockInWorld};
