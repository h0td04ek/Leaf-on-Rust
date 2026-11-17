//! Перенос `net.minecraft.stats.RecipeBookSettings` — wire-часть
//! (4×TypeSettings: open+filtering bools).

use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::inventory::recipe_book_type::RecipeBookType;

/// `RecipeBookSettings.TypeSettings`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeSettings {
    /// `open`.
    pub open: bool,
    /// `filtering`.
    pub filtering: bool,
}

impl TypeSettings {
    /// `DEFAULT`.
    pub const DEFAULT: TypeSettings = TypeSettings { open: false, filtering: false };

    /// `new`.
    pub fn new(open: bool, filtering: bool) -> Self {
        Self { open, filtering }
    }

    /// `setOpen`.
    pub fn set_open(&self, open: bool) -> Self {
        Self { open, ..*self }
    }

    /// `setFiltering`.
    pub fn set_filtering(&self, filtering: bool) -> Self {
        Self { filtering, ..*self }
    }

    /// `STREAM_CODEC` — BOOL + BOOL.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, TypeSettings> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, s: &TypeSettings| {
                buf.write_bool(s.open);
                buf.write_bool(s.filtering);
            },
            |buf: &mut FriendlyByteBuf| {
                let open = buf.read_bool();
                let filtering = buf.read_bool();
                TypeSettings::new(open, filtering)
            },
        )
    }
}

/// `RecipeBookSettings`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeBookSettings {
    /// `crafting`.
    pub crafting: TypeSettings,
    /// `furnace`.
    pub furnace: TypeSettings,
    /// `blastFurnace`.
    pub blast_furnace: TypeSettings,
    /// `smoker`.
    pub smoker: TypeSettings,
}

impl RecipeBookSettings {
    /// `new()` — все DEFAULT.
    pub fn new() -> Self {
        Self {
            crafting: TypeSettings::DEFAULT,
            furnace: TypeSettings::DEFAULT,
            blast_furnace: TypeSettings::DEFAULT,
            smoker: TypeSettings::DEFAULT,
        }
    }

    /// Приватный конструктор из 4 настроек.
    pub fn from_parts(
        crafting: TypeSettings,
        furnace: TypeSettings,
        blast_furnace: TypeSettings,
        smoker: TypeSettings,
    ) -> Self {
        Self { crafting, furnace, blast_furnace, smoker }
    }

    /// `getSettings(type)`.
    pub fn get_settings(&self, book_type: RecipeBookType) -> TypeSettings {
        match book_type {
            RecipeBookType::CRAFTING => self.crafting,
            RecipeBookType::FURNACE => self.furnace,
            RecipeBookType::BLAST_FURNACE => self.blast_furnace,
            RecipeBookType::SMOKER => self.smoker,
        }
    }

    /// `isOpen`.
    pub fn is_open(&self, book_type: RecipeBookType) -> bool {
        self.get_settings(book_type).open
    }

    /// `setOpen`.
    pub fn set_open(&mut self, book_type: RecipeBookType, open: bool) {
        self.update_settings(book_type, |s| s.set_open(open));
    }

    /// `isFiltering`.
    pub fn is_filtering(&self, book_type: RecipeBookType) -> bool {
        self.get_settings(book_type).filtering
    }

    /// `setFiltering`.
    pub fn set_filtering(&mut self, book_type: RecipeBookType, filtering: bool) {
        self.update_settings(book_type, |s| s.set_filtering(filtering));
    }

    /// `updateSettings`.
    fn update_settings(&mut self, book_type: RecipeBookType, updater: impl FnOnce(TypeSettings) -> TypeSettings) {
        match book_type {
            RecipeBookType::CRAFTING => self.crafting = updater(self.crafting),
            RecipeBookType::FURNACE => self.furnace = updater(self.furnace),
            RecipeBookType::BLAST_FURNACE => self.blast_furnace = updater(self.blast_furnace),
            RecipeBookType::SMOKER => self.smoker = updater(self.smoker),
        }
    }

    /// `copy`.
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// `replaceFrom`.
    pub fn replace_from(&mut self, other: &RecipeBookSettings) {
        *self = other.clone();
    }

    /// `STREAM_CODEC` — 4×TypeSettings.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, RecipeBookSettings> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, s: &RecipeBookSettings| {
                TypeSettings::stream_codec().encode(buf, &s.crafting);
                TypeSettings::stream_codec().encode(buf, &s.furnace);
                TypeSettings::stream_codec().encode(buf, &s.blast_furnace);
                TypeSettings::stream_codec().encode(buf, &s.smoker);
            },
            |buf: &mut FriendlyByteBuf| {
                let crafting = TypeSettings::stream_codec().decode(buf);
                let furnace = TypeSettings::stream_codec().decode(buf);
                let blast_furnace = TypeSettings::stream_codec().decode(buf);
                let smoker = TypeSettings::stream_codec().decode(buf);
                RecipeBookSettings::from_parts(crafting, furnace, blast_furnace, smoker)
            },
        )
    }
}

impl Default for RecipeBookSettings {
    fn default() -> Self {
        Self::new()
    }
}
