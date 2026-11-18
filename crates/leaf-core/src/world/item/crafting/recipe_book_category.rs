//! Перенос `net.minecraft.world.item.crafting.RecipeBookCategory`.

/// `RecipeBookCategory` — категория книги рецептов.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecipeBookCategory {
    /// `crafting_building_blocks`.
    CraftingBuildingBlocks,
    /// `crafting_redstone`.
    CraftingRedstone,
    /// `crafting_equipment`.
    CraftingEquipment,
    /// `crafting_misc`.
    CraftingMisc,
    /// `furnace_food`.
    FurnaceFood,
    /// `furnace_blocks`.
    FurnaceBlocks,
    /// `furnace_misc`.
    FurnaceMisc,
    /// `blast_furnace_blocks`.
    BlastFurnaceBlocks,
    /// `blast_furnace_misc`.
    BlastFurnaceMisc,
    /// `smoker_food`.
    SmokerFood,
    /// `stonecutter`.
    Stonecutter,
    /// `smithing`.
    Smithing,
    /// `campfire`.
    Campfire,
}

impl RecipeBookCategory {
    /// `values()` — все категории в порядке регистрации.
    pub const VALUES: [RecipeBookCategory; 13] = [
        RecipeBookCategory::CraftingBuildingBlocks,
        RecipeBookCategory::CraftingRedstone,
        RecipeBookCategory::CraftingEquipment,
        RecipeBookCategory::CraftingMisc,
        RecipeBookCategory::FurnaceFood,
        RecipeBookCategory::FurnaceBlocks,
        RecipeBookCategory::FurnaceMisc,
        RecipeBookCategory::BlastFurnaceBlocks,
        RecipeBookCategory::BlastFurnaceMisc,
        RecipeBookCategory::SmokerFood,
        RecipeBookCategory::Stonecutter,
        RecipeBookCategory::Smithing,
        RecipeBookCategory::Campfire,
    ];

    /// `from_id` — registry id → category.
    pub fn from_id(id: i32) -> Option<Self> {
        Self::VALUES.get(id as usize).copied()
    }

    /// `get_serialized_name` — имя для registry.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            RecipeBookCategory::CraftingBuildingBlocks => "crafting_building_blocks",
            RecipeBookCategory::CraftingRedstone => "crafting_redstone",
            RecipeBookCategory::CraftingEquipment => "crafting_equipment",
            RecipeBookCategory::CraftingMisc => "crafting_misc",
            RecipeBookCategory::FurnaceFood => "furnace_food",
            RecipeBookCategory::FurnaceBlocks => "furnace_blocks",
            RecipeBookCategory::FurnaceMisc => "furnace_misc",
            RecipeBookCategory::BlastFurnaceBlocks => "blast_furnace_blocks",
            RecipeBookCategory::BlastFurnaceMisc => "blast_furnace_misc",
            RecipeBookCategory::SmokerFood => "smoker_food",
            RecipeBookCategory::Stonecutter => "stonecutter",
            RecipeBookCategory::Smithing => "smithing",
            RecipeBookCategory::Campfire => "campfire",
        }
    }
}
