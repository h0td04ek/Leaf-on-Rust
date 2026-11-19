//! Перенос `net.minecraft.world.item.enchantment.TargetedConditionalEffect`.

use crate::world::level::storage::loot::loot_context::LootContext;
use crate::world::level::storage::loot::predicates::loot_item_condition::LootItemCondition;

use super::enchantment_target::EnchantmentTarget;

/// `TargetedConditionalEffect<T>` — эффект с целями
/// (`enchanted`/`affected`) и опциональным условием.
pub struct TargetedConditionalEffect<T> {
    /// `enchanted` — сторона, на которой висит зачарование.
    pub enchanted: EnchantmentTarget,
    /// `affected` — сторона, на которую действует эффект.
    pub affected: EnchantmentTarget,
    /// `effect`.
    pub effect: T,
    /// `requirements` — `Optional<LootItemCondition>`.
    pub requirements: Option<Box<dyn LootItemCondition>>,
}

impl<T> TargetedConditionalEffect<T> {
    /// `new`.
    pub fn new(
        enchanted: EnchantmentTarget,
        affected: EnchantmentTarget,
        effect: T,
        requirements: Option<Box<dyn LootItemCondition>>,
    ) -> Self {
        Self { enchanted, affected, effect, requirements }
    }

    /// `matches(lootContext)`.
    pub fn matches(&self, context: &mut LootContext) -> bool {
        self.requirements
            .as_ref()
            .map(|c| c.test(context))
            .unwrap_or(true)
    }
}
