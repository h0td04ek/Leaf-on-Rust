//! Перенос `net.minecraft.world.item.crafting.display.RecipeDisplay`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::crafting::display::slot_display::SlotDisplay;
use crate::world::item::crafting::recipe_book_category::RecipeBookCategory;

/// `RecipeDisplay` — отображение рецепта (registry-dispatched).
#[derive(Clone, Debug)]
pub enum RecipeDisplay {
    /// `ShapelessCraftingRecipeDisplay`.
    ShapelessCrafting {
        /// `group`.
        group: Option<String>,
        /// `ingredients`.
        ingredients: Vec<SlotDisplay>,
        /// `result`.
        result: SlotDisplay,
        /// `craftingStation`.
        crafting_station: SlotDisplay,
        /// `category`.
        category: Option<RecipeBookCategory>,
    },
    /// `ShapedCraftingRecipeDisplay`.
    ShapedCrafting {
        /// `group`.
        group: Option<String>,
        /// `width`.
        width: i32,
        /// `height`.
        height: i32,
        /// `ingredients`.
        ingredients: Vec<SlotDisplay>,
        /// `result`.
        result: SlotDisplay,
        /// `craftingStation`.
        crafting_station: SlotDisplay,
        /// `category`.
        category: Option<RecipeBookCategory>,
    },
    /// `FurnaceRecipeDisplay`.
    Furnace {
        /// `recipeCategory`.
        recipe_category: Option<RecipeBookCategory>,
        /// `ingredient`.
        ingredient: SlotDisplay,
        /// `fuel`.
        fuel: SlotDisplay,
        /// `result`.
        result: SlotDisplay,
        /// `cookingTime`.
        cooking_time: i32,
        /// `experience`.
        experience: f32,
    },
    /// `StonecutterRecipeDisplay`.
    Stonecutter {
        /// `ingredient`.
        ingredient: SlotDisplay,
        /// `result`.
        result: SlotDisplay,
    },
    /// `SmithingRecipeDisplay`.
    Smithing {
        /// `template`.
        template: SlotDisplay,
        /// `base`.
        base: SlotDisplay,
        /// `addition`.
        addition: SlotDisplay,
        /// `result`.
        result: SlotDisplay,
    },
}

impl RecipeDisplay {
    /// `type()` — registry id.
    pub fn type_id(&self) -> i32 {
        match self {
            RecipeDisplay::ShapelessCrafting { .. } => 0,
            RecipeDisplay::ShapedCrafting { .. } => 1,
            RecipeDisplay::Furnace { .. } => 2,
            RecipeDisplay::Stonecutter { .. } => 3,
            RecipeDisplay::Smithing { .. } => 4,
        }
    }

    /// `result()` — SlotDisplay результата.
    pub fn result(&self) -> &SlotDisplay {
        match self {
            RecipeDisplay::ShapelessCrafting { result, .. } => result,
            RecipeDisplay::ShapedCrafting { result, .. } => result,
            RecipeDisplay::Furnace { result, .. } => result,
            RecipeDisplay::Stonecutter { result, .. } => result,
            RecipeDisplay::Smithing { result, .. } => result,
        }
    }

    /// `craftingStation()` — SlotDisplay станции крафта.
    pub fn crafting_station(&self) -> Option<&SlotDisplay> {
        match self {
            RecipeDisplay::ShapelessCrafting { crafting_station, .. } => Some(crafting_station),
            RecipeDisplay::ShapedCrafting { crafting_station, .. } => Some(crafting_station),
            RecipeDisplay::Furnace { .. } => None,
            RecipeDisplay::Stonecutter { .. } => None,
            RecipeDisplay::Smithing { .. } => None,
        }
    }

    /// `STREAM_CODEC` — encode.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.type_id());
        match self {
            RecipeDisplay::ShapelessCrafting { group, ingredients, result, crafting_station, category } => {
                Self::write_optional_string(buf, group);
                buf.write_collection(ingredients, |b, i| i.encode(b));
                result.encode(buf);
                crafting_station.encode(buf);
                Self::write_optional_category(buf, category);
            }
            RecipeDisplay::ShapedCrafting { group, width, height, ingredients, result, crafting_station, category } => {
                Self::write_optional_string(buf, group);
                buf.write_var_int(*width);
                buf.write_var_int(*height);
                buf.write_collection(ingredients, |b, i| i.encode(b));
                result.encode(buf);
                crafting_station.encode(buf);
                Self::write_optional_category(buf, category);
            }
            RecipeDisplay::Furnace { recipe_category, ingredient, fuel, result, cooking_time, experience } => {
                Self::write_optional_category(buf, recipe_category);
                ingredient.encode(buf);
                fuel.encode(buf);
                result.encode(buf);
                buf.write_var_int(*cooking_time);
                buf.write_float(*experience);
            }
            RecipeDisplay::Stonecutter { ingredient, result } => {
                ingredient.encode(buf);
                result.encode(buf);
            }
            RecipeDisplay::Smithing { template, base, addition, result } => {
                template.encode(buf);
                base.encode(buf);
                addition.encode(buf);
                result.encode(buf);
            }
        }
    }

    /// `STREAM_CODEC` — decode.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let type_id = buf.read_var_int();
        match type_id {
            0 => {
                let group = Self::read_optional_string(buf);
                let ingredients = buf.read_collection(|n| Vec::with_capacity(n), |b| SlotDisplay::decode(b));
                let result = SlotDisplay::decode(buf);
                let crafting_station = SlotDisplay::decode(buf);
                let category = Self::read_optional_category(buf);
                RecipeDisplay::ShapelessCrafting { group, ingredients, result, crafting_station, category }
            }
            1 => {
                let group = Self::read_optional_string(buf);
                let width = buf.read_var_int();
                let height = buf.read_var_int();
                let ingredients = buf.read_collection(|n| Vec::with_capacity(n), |b| SlotDisplay::decode(b));
                let result = SlotDisplay::decode(buf);
                let crafting_station = SlotDisplay::decode(buf);
                let category = Self::read_optional_category(buf);
                RecipeDisplay::ShapedCrafting { group, width, height, ingredients, result, crafting_station, category }
            }
            2 => {
                let recipe_category = Self::read_optional_category(buf);
                let ingredient = SlotDisplay::decode(buf);
                let fuel = SlotDisplay::decode(buf);
                let result = SlotDisplay::decode(buf);
                let cooking_time = buf.read_var_int();
                let experience = buf.read_float();
                RecipeDisplay::Furnace { recipe_category, ingredient, fuel, result, cooking_time, experience }
            }
            3 => {
                let ingredient = SlotDisplay::decode(buf);
                let result = SlotDisplay::decode(buf);
                RecipeDisplay::Stonecutter { ingredient, result }
            }
            4 => {
                let template = SlotDisplay::decode(buf);
                let base = SlotDisplay::decode(buf);
                let addition = SlotDisplay::decode(buf);
                let result = SlotDisplay::decode(buf);
                RecipeDisplay::Smithing { template, base, addition, result }
            }
            _ => RecipeDisplay::Stonecutter {
                ingredient: SlotDisplay::Empty,
                result: SlotDisplay::Empty,
            },
        }
    }

    fn write_optional_string(buf: &mut FriendlyByteBuf, s: &Option<String>) {
        match s {
            Some(s) => {
                buf.write_bool(true);
                let _ = buf.write_utf(s);
            }
            None => {
                buf.write_bool(false);
            }
        }
    }

    fn read_optional_string(buf: &mut FriendlyByteBuf) -> Option<String> {
        if buf.read_bool() {
            buf.read_utf().ok()
        } else {
            None
        }
    }

    fn write_optional_category(buf: &mut FriendlyByteBuf, c: &Option<RecipeBookCategory>) {
        match c {
            Some(c) => {
                buf.write_bool(true);
                buf.write_var_int(*c as i32);
            }
            None => {
                buf.write_bool(false);
            }
        }
    }

    fn read_optional_category(buf: &mut FriendlyByteBuf) -> Option<RecipeBookCategory> {
        if buf.read_bool() {
            let id = buf.read_var_int();
            RecipeBookCategory::from_id(id)
        } else {
            None
        }
    }
}
