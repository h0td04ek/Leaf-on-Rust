//! Перенос `net.minecraft.util.context` — типизированные ключи
//! параметров контекста (loot/enchantment predicates).

pub mod context_key;
pub mod context_key_set;
pub mod context_map;

pub use context_key::ContextKey;
pub use context_key_set::ContextKeySet;
pub use context_map::ContextMap;
