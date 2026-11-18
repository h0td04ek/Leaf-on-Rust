//! Перенос `net.minecraft.world.item.enchantment.ConditionalEffect`.

use crate::world::level::storage::loot::loot_context::LootContext;
use crate::world::level::storage::loot::predicates::loot_item_condition::LootItemCondition;

/// `ConditionalEffect<T>` — эффект + опциональное loot-условие.
pub struct ConditionalEffect<T> {
    /// `effect`.
    pub effect: T,
    /// `requirements`.
    pub requirements: Option<Box<dyn LootItemCondition>>,
}

impl<T> ConditionalEffect<T> {
    pub fn new(effect: T, requirements: Option<Box<dyn LootItemCondition>>) -> Self {
        Self { effect, requirements }
    }

    /// `matches(lootContext)`.
    pub fn matches(&self, context: &LootContext) -> bool {
        self.requirements
            .as_ref()
            .map(|c| c.test(context))
            .unwrap_or(true)
    }
}
