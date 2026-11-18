//! Перенос `net.minecraft.world.item.crafting.display.RecipeDisplayEntry`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::crafting::display::recipe_display::RecipeDisplay;
use crate::world::item::crafting::display::recipe_display_id::RecipeDisplayId;
use crate::world::item::crafting::ingredient::Ingredient;
use crate::world::item::crafting::recipe_book_category::RecipeBookCategory;

/// `RecipeDisplayEntry` — `{ id, display, group, category, craftingRequirements }`.
#[derive(Clone, Debug)]
pub struct RecipeDisplayEntry {
    /// `id`.
    pub id: RecipeDisplayId,
    /// `display`.
    pub display: RecipeDisplay,
    /// `group` — OptionalInt.
    pub group: Option<i32>,
    /// `category`.
    pub category: RecipeBookCategory,
    /// `craftingRequirements` — Optional<List<Ingredient>>.
    pub crafting_requirements: Option<Vec<Ingredient>>,
}

impl RecipeDisplayEntry {
    /// `new(id, display, group, category, craftingRequirements)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RecipeDisplayId,
        display: RecipeDisplay,
        group: Option<i32>,
        category: RecipeBookCategory,
        crafting_requirements: Option<Vec<Ingredient>>,
    ) -> Self {
        Self { id, display, group, category, crafting_requirements }
    }

    /// `STREAM_CODEC` — encode.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        RecipeDisplayId::stream_codec().encode(buf, &self.id);
        self.display.encode(buf);
        // OPTIONAL_VAR_INT
        match self.group {
            Some(g) => {
                buf.write_var_int(g + 1);
            }
            None => {
                buf.write_var_int(0);
            }
        }
        // registry(Registries.RECIPE_BOOK_CATEGORY) — VarInt id
        buf.write_var_int(self.category as i32);
        // OPTIONAL list of Ingredient
        match &self.crafting_requirements {
            Some(reqs) => {
                buf.write_bool(true);
                buf.write_collection(reqs, |b, i| i.encode(b));
            }
            None => {
                buf.write_bool(false);
            }
        }
    }

    /// `STREAM_CODEC` — decode.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let id = RecipeDisplayId::stream_codec().decode(buf);
        let display = RecipeDisplay::decode(buf);
        let group_raw = buf.read_var_int();
        let group = if group_raw == 0 { None } else { Some(group_raw - 1) };
        let category_id = buf.read_var_int();
        let category = RecipeBookCategory::from_id(category_id).unwrap_or(RecipeBookCategory::CraftingMisc);
        let crafting_requirements = if buf.read_bool() {
            Some(buf.read_collection(|n| Vec::with_capacity(n), |b| Ingredient::decode(b)))
        } else {
            None
        };
        Self::new(id, display, group, category, crafting_requirements)
    }
}
