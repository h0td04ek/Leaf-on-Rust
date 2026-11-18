//! Перенос `net.minecraft.world.item.crafting`.

pub mod ingredient;
pub mod recipe_book_category;
pub mod recipe_property_set;
pub mod selectable_recipe;
pub mod display;

pub use ingredient::Ingredient;
pub use recipe_book_category::RecipeBookCategory;
pub use recipe_property_set::RecipePropertySet;
pub use selectable_recipe::{SelectableRecipe, SingleInputEntry, SingleInputSet, RecipeHolder};
pub use display::SlotDisplay;
