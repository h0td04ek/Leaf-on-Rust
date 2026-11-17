//! Перенос `net.minecraft.world.inventory.RecipeBookType`.

/// `RecipeBookType` — 4 значения в порядке объявления.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecipeBookType {
    /// `CRAFTING`.
    CRAFTING,
    /// `FURNACE`.
    FURNACE,
    /// `BLAST_FURNACE`.
    BLAST_FURNACE,
    /// `SMOKER`.
    SMOKER,
}

impl RecipeBookType {
    /// `values()`.
    pub const VALUES: [RecipeBookType; 4] = [
        RecipeBookType::CRAFTING,
        RecipeBookType::FURNACE,
        RecipeBookType::BLAST_FURNACE,
        RecipeBookType::SMOKER,
    ];

    /// `byOrdinal` — `readEnum`/`writeEnum` (VarInt ordinal).
    pub fn by_ordinal(ordinal: i32) -> RecipeBookType {
        Self::VALUES[ordinal.rem_euclid(4) as usize]
    }
}
