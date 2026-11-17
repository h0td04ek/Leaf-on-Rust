//! Перенос `CustomQueryPayload` — интерфейс → enum.
//! `DiscardedQueryPayload`, Paper `PlayerInfoChannelPayload`.

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `CustomQueryPayload`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CustomQueryPayload {
    /// `DiscardedQueryPayload(id)` — неизвестный payload, тело пропускается.
    Discarded { id: Identifier },
    /// Paper `PlayerInfoChannelPayload(id, buffer)` — сырые байты.
    PlayerInfoChannel { id: Identifier, data: Vec<u8> },
}

impl CustomQueryPayload {
    /// `id()`.
    pub fn id(&self) -> &Identifier {
        match self {
            CustomQueryPayload::Discarded { id } => id,
            CustomQueryPayload::PlayerInfoChannel { id, .. } => id,
        }
    }

    /// `write`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        match self {
            CustomQueryPayload::Discarded { .. } => {}
            CustomQueryPayload::PlayerInfoChannel { data, .. } => {
                buffer.write_bytes(data);
            }
        }
    }
}
