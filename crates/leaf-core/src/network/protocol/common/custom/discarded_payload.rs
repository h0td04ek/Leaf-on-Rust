//! Перенос `net.minecraft.network.protocol.common.custom.DiscardedPayload`.

use crate::resources::identifier::Identifier;

/// `DiscardedPayload` — сырые байты неизвестного payload'а.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscardedPayload {
    /// `id` — тип payload'а.
    pub id: Identifier,
    /// `data` — сырые байты.
    pub data: Vec<u8>,
}

impl DiscardedPayload {
    /// `new DiscardedPayload(id, data)`.
    pub fn new(id: Identifier, data: Vec<u8>) -> Self {
        Self { id, data }
    }

    /// `type()`.
    pub fn payload_type(&self) -> Identifier {
        self.id.clone()
    }
}
