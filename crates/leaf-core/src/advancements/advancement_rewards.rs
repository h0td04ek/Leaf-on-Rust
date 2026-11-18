//! Перенос `net.minecraft.advancements.AdvancementRewards`.

use crate::resources::identifier::Identifier;

/// `AdvancementRewards` — награды за advancement.
/// Для network-формата используется `EMPTY` (не сериализуется).
#[derive(Clone, Debug, Default)]
pub struct AdvancementRewards {
    /// `experience`.
    pub experience: i32,
    /// `loot` — ResourceKey<LootTable>.
    pub loot: Vec<Identifier>,
    /// `recipes` — ResourceKey<Recipe>.
    pub recipes: Vec<Identifier>,
    /// `function` — CacheableFunction.
    pub function: Option<Identifier>,
}

impl AdvancementRewards {
    /// `EMPTY`.
    pub const EMPTY: Self = Self {
        experience: 0,
        loot: Vec::new(),
        recipes: Vec::new(),
        function: None,
    };
}
