//! Перенос `net.minecraft.world.level.storage.loot.predicates.LootItemCondition`
//! и стандартных условий.
//!
//! Трейт + типы условий портируются полностью; `test` принимает
//! `&LootContext` (в Java `Predicate<LootContext>`).

use crate::world::level::storage::loot::loot_context::LootContext;

/// `LootItemCondition` — условие проверки loot-контекста.
pub trait LootItemCondition: Send + Sync {
    /// `test(context)`.
    fn test(&self, context: &LootContext) -> bool;
}

/// `LootItemCondition.Builder`.
pub trait LootItemConditionBuilder {
    /// `build()`.
    fn build(self: Box<Self>) -> Box<dyn LootItemCondition>;
}

/// `AnyOfCondition` (`any_of`).
pub struct AnyOfCondition {
    /// `terms`.
    pub terms: Vec<Box<dyn LootItemCondition>>,
}
impl LootItemCondition for AnyOfCondition {
    fn test(&self, context: &LootContext) -> bool {
        self.terms.iter().any(|t| t.test(context))
    }
}

/// `AllOfCondition` (`all_of`).
pub struct AllOfCondition {
    /// `terms`.
    pub terms: Vec<Box<dyn LootItemCondition>>,
}
impl LootItemCondition for AllOfCondition {
    fn test(&self, context: &LootContext) -> bool {
        self.terms.iter().all(|t| t.test(context))
    }
}

/// `InvertedLootItemCondition`.
pub struct InvertedLootItemCondition {
    /// `term`.
    pub term: Box<dyn LootItemCondition>,
}
impl LootItemCondition for InvertedLootItemCondition {
    fn test(&self, context: &LootContext) -> bool {
        !self.term.test(context)
    }
}
