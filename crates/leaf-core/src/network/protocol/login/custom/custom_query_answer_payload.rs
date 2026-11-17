//! Перенос `CustomQueryAnswerPayload` — интерфейс → enum.
//! `DiscardedQueryAnswerPayload`, Paper `QueryAnswerPayload`.

use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `CustomQueryAnswerPayload`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CustomQueryAnswerPayload {
    /// `DiscardedQueryAnswerPayload` — пустое тело.
    #[default]
    Discarded,
    /// Paper `QueryAnswerPayload(buffer)` — сырые байты ответа.
    QueryAnswer { data: Vec<u8> },
}

impl CustomQueryAnswerPayload {
    /// `DiscardedQueryAnswerPayload.INSTANCE`.
    pub fn discarded() -> Self {
        Self::Discarded
    }

    /// `write`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        if let CustomQueryAnswerPayload::QueryAnswer { data } = self {
            buffer.write_bytes(data);
        }
    }
}
