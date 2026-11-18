//! Перенос `net.minecraft.world.item.crafting.SelectableRecipe`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::world::item::crafting::ingredient::Ingredient;
use crate::world::item::crafting::display::SlotDisplay;
use crate::world::item::item_stack::ItemStack;

/// `SelectableRecipe` — выбираемый рецепт с отображением.
#[derive(Debug)]
pub struct SelectableRecipe {
    /// `optionDisplay` — SlotDisplay для отображения.
    pub option_display: SlotDisplay,
    /// `recipe` — Optional<RecipeHolder>.
    pub recipe: Option<RecipeHolder>,
}

/// `RecipeHolder` — `{ id, value }` для рецепта.
#[derive(Debug)]
pub struct RecipeHolder {
    /// `id` — ResourceKey<Recipe>.
    pub id: Identifier,
    /// `value` — type-erased рецепт.
    pub value: Box<dyn std::any::Any + Send + Sync>,
}

impl SelectableRecipe {
    /// `new(optionDisplay, recipe)`.
    pub fn new(option_display: SlotDisplay, recipe: Option<RecipeHolder>) -> Self {
        Self { option_display, recipe }
    }

    /// `noRecipeCodec` — encode без recipe.
    pub fn encode_no_recipe(&self, buf: &mut FriendlyByteBuf) {
        self.option_display.encode(buf);
    }

    /// `noRecipeCodec` — decode без recipe.
    pub fn decode_no_recipe(buf: &mut FriendlyByteBuf) -> Self {
        let option_display = SlotDisplay::decode(buf);
        Self::new(option_display, None)
    }
}

/// `SingleInputEntry` — `{ input, recipe }`.
#[derive(Debug)]
pub struct SingleInputEntry {
    /// `input` — Ingredient.
    pub input: Ingredient,
    /// `recipe` — SelectableRecipe.
    pub recipe: SelectableRecipe,
}

impl SingleInputEntry {
    /// `new(input, recipe)`.
    pub fn new(input: Ingredient, recipe: SelectableRecipe) -> Self {
        Self { input, recipe }
    }

    /// `noRecipeCodec` — encode.
    pub fn encode_no_recipe(&self, buf: &mut FriendlyByteBuf) {
        self.input.encode(buf);
        self.recipe.encode_no_recipe(buf);
    }

    /// `noRecipeCodec` — decode.
    pub fn decode_no_recipe(buf: &mut FriendlyByteBuf) -> Self {
        let input = Ingredient::decode(buf);
        let recipe = SelectableRecipe::decode_no_recipe(buf);
        Self::new(input, recipe)
    }
}

/// `SingleInputSet` — список `SingleInputEntry`.
#[derive(Debug, Default)]
pub struct SingleInputSet {
    /// `entries`.
    pub entries: Vec<SingleInputEntry>,
}

impl SingleInputSet {
    /// `empty()`.
    pub fn empty() -> Self {
        Self { entries: Vec::new() }
    }

    /// `new(entries)`.
    pub fn new(entries: Vec<SingleInputEntry>) -> Self {
        Self { entries }
    }

    /// `noRecipeCodec` — encode.
    pub fn encode_no_recipe(&self, buf: &mut FriendlyByteBuf) {
        buf.write_collection(&self.entries, |b, e| e.encode_no_recipe(b));
    }

    /// `noRecipeCodec` — decode.
    pub fn decode_no_recipe(buf: &mut FriendlyByteBuf) -> Self {
        let entries = buf.read_collection(|n| Vec::with_capacity(n), |b| SingleInputEntry::decode_no_recipe(b));
        Self::new(entries)
    }

    /// `acceptsInput(stack)`.
    pub fn accepts_input(&self, stack: &ItemStack) -> bool {
        self.entries.iter().any(|e| e.input.test(stack))
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.entries.len()
    }
}
