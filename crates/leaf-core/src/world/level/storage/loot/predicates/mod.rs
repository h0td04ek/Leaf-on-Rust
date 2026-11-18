//! Перенос `net.minecraft.world.level.storage.loot.predicates` —
//! условия loot-таблиц.

pub mod loot_item_condition;

pub use loot_item_condition::{
    AllOfCondition, AnyOfCondition, InvertedLootItemCondition,
    LootItemCondition, LootItemConditionBuilder,
};
