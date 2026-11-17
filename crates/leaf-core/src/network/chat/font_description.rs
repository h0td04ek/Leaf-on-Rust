//! Перенос `net.minecraft.network.chat.FontDescription`.

use crate::resources::identifier::Identifier;

/// `FontDescription` — описание шрифта компонента.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontDescription {
    /// `Resource(id)` — по умолчанию `minecraft:default`.
    Resource(Identifier),
    /// `AtlasSprite(atlasId, spriteId)`.
    AtlasSprite { atlas_id: Identifier, sprite_id: Identifier },
    /// `PlayerSprite(profile, hat)` — профиль пока хранится как имя+uuid.
    PlayerSprite { profile_name: String, profile_id: Option<crate::util::uuid::Uuid>, hat: bool },
}

impl Default for FontDescription {
    fn default() -> Self {
        FontDescription::Resource(
            Identifier::with_default_namespace("default").expect("default font id is valid"),
        )
    }
}

impl FontDescription {
    /// `FontDescription.DEFAULT`.
    pub fn resource_default() -> FontDescription {
        FontDescription::default()
    }
}
