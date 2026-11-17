//! Перенос `net.minecraft.network.chat.SignedMessageLink`.

use crate::util::uuid::Uuid;

/// `SignedMessageLink` — ссылка на сообщение в цепочке (index + sender + session).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignedMessageLink {
    /// `index`.
    pub index: i32,
    /// `sender`.
    pub sender: Uuid,
    /// `sessionId`.
    pub session_id: Uuid,
}

impl SignedMessageLink {
    /// `unsigned(sender)` — корневая ссылка без сессии.
    pub fn unsigned(sender: Uuid) -> Self {
        Self::root(sender, Uuid::NIL)
    }

    /// `root(sender, sessionId)`.
    pub fn root(sender: Uuid, session_id: Uuid) -> Self {
        Self { index: 0, sender, session_id }
    }

    /// `isDescendantOf`.
    pub fn is_descendant_of(&self, other: &SignedMessageLink) -> bool {
        self.index > other.index
            && self.sender == other.sender
            && self.session_id == other.session_id
    }

    /// `advance` — следующий index; `None` при `Integer.MAX_VALUE`.
    pub fn advance(&self) -> Option<Self> {
        if self.index == i32::MAX {
            None
        } else {
            Some(Self { index: self.index + 1, ..*self })
        }
    }

    /// `updateSignature` — UUID sender + UUID sessionId + Ints.toByteArray(index).
    pub fn update_signature<F>(&self, mut output: F) -> Result<(), crate::util::signature_validator::SignatureException>
    where
        F: FnMut(&[u8]) -> Result<(), crate::util::signature_validator::SignatureException>,
    {
        output(&self.sender_bytes())?;
        output(&self.session_id_bytes())?;
        output(&self.index.to_be_bytes())?;
        Ok(())
    }

    /// `UUIDUtil.uuidToByteArray` — big-endian 16 байт.
    fn sender_bytes(&self) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[..8].copy_from_slice(&self.sender.most_significant_bits.to_be_bytes());
        b[8..].copy_from_slice(&self.sender.least_significant_bits.to_be_bytes());
        b
    }

    fn session_id_bytes(&self) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[..8].copy_from_slice(&self.session_id.most_significant_bits.to_be_bytes());
        b[8..].copy_from_slice(&self.session_id.least_significant_bits.to_be_bytes());
        b
    }
}
