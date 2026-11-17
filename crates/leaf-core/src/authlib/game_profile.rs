//! Перенос `com.mojang.authlib.GameProfile`.

use super::property_map::PropertyMap;
use crate::util::uuid::Uuid;

/// `GameProfile` — (uuid, name, properties).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameProfile {
    /// `id`.
    pub id: Uuid,
    /// `name`.
    pub name: String,
    /// `properties`.
    pub properties: PropertyMap,
}

impl GameProfile {
    /// `new GameProfile(id, name)`.
    pub fn new(id: Uuid, name: impl Into<String>) -> Self {
        Self { id, name: name.into(), properties: PropertyMap::new() }
    }

    /// `new GameProfile(id, name, properties)`.
    pub fn with_properties(id: Uuid, name: impl Into<String>, properties: PropertyMap) -> Self {
        Self { id, name: name.into(), properties }
    }
}
