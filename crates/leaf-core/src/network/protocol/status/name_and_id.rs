//! Перенос `NameAndId` (`net.minecraft.server` / Used by ServerStatus.Players).

use crate::util::uuid::Uuid;

/// `NameAndId` — (uuid, name) запись для status-ответа.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameAndId {
    pub id: Uuid,
    pub name: String,
}

impl NameAndId {
    pub fn new(id: Uuid, name: impl Into<String>) -> Self {
        Self { id, name: name.into() }
    }
}
