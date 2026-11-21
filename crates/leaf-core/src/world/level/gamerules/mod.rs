//! `net.minecraft.world.level.gamerules` — система игровых правил.
//!
//! Отличия от Java:
//! - Brigadier `ArgumentType` заменён на `GameRuleArgument`
//!   (bool/int с диапазоном) — парсинг командного аргумента.
//! - DFU `Codec<T>` заменён на строковую сериализацию
//!   (`serialize`/`deserialize`); NBT-кодек `GameRuleMap`
//!   реализован напрямую (`to_nbt`/`from_nbt`) — level.dat
//!   хранит правила как compound строк.
//! - Paper `gameRuleIndex`/`idAccess` сохранён (usize-ключи).
//! - Paper per-world `ServerLevel` callback в `set` — хук
//!   `on_game_rule_changed` (plugin API отложен).

pub mod game_rule;
pub mod game_rule_category;
pub mod game_rule_map;
pub mod game_rule_type;
pub mod game_rules;

pub use game_rule::{GameRule, GameRuleArgument, game_rule_registry, game_rule_bool, game_rule_int};
pub use game_rule_category::GameRuleCategory;
pub use game_rule_map::GameRuleMap;
pub use game_rule_type::GameRuleType;
pub use game_rules::{bootstrap, GameRules, GameRuleTypeVisitor};
