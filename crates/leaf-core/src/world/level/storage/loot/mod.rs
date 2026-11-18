//! Перенос `net.minecraft.world.level.storage.loot` —
//! контекст, параметры, условия и числовые провайдеры.

pub mod int_range;
pub mod loot_context;
pub mod parameters;
pub mod predicates;
pub mod providers;

pub use int_range::IntRange;
pub use loot_context::{EntityTarget, ItemStackTarget, LootContext};
