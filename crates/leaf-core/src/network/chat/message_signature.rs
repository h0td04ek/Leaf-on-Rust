//! Перенос `net.minecraft.network.chat.MessageSignature` (+ `Packed`).

use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `MessageSignature` — 256-байтовая RSA-подпись сообщения.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MessageSignature {
    /// `bytes` — ровно 256 байт.
    pub bytes: [u8; 256],
}

impl MessageSignature {
    /// `SIGNATURE_BYTES`.
    pub const SIGNATURE_BYTES: usize = 256;

    /// `new MessageSignature(bytes)` — проверяет длину.
    pub fn new(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != Self::SIGNATURE_BYTES {
            return Err(format!(
                "Invalid message signature size: expected {}, got {}",
                Self::SIGNATURE_BYTES,
                bytes.len()
            ));
        }
        let mut arr = [0u8; 256];
        arr.copy_from_slice(bytes);
        Ok(Self { bytes: arr })
    }

    /// `read` — 256 байт из буфера.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Self {
        let data = buffer.read_bytes_vec(Self::SIGNATURE_BYTES);
        Self::new(&data).expect("invalid signature length")
    }

    /// `write` — 256 байт в буфер.
    pub fn write(buffer: &mut FriendlyByteBuf, signature: &MessageSignature) {
        buffer.write_bytes(&signature.bytes);
    }

    /// `bytes()` — ссылка на байты.
    pub fn bytes(&self) -> &[u8; 256] {
        &self.bytes
    }
}

/// `MessageSignature.Packed` — ссылка на подпись по id из кеша
/// либо полная подпись (`FULL_SIGNATURE` = -1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedSignature {
    /// `id` — id в `MessageSignatureCache` (-1 = полная подпись).
    pub id: i32,
    /// `fullSignature` — `Some` при `id == -1`.
    pub full_signature: Option<MessageSignature>,
}

/// `Packed.FULL_SIGNATURE`.
pub const FULL_SIGNATURE: i32 = -1;

impl PackedSignature {
    /// `new Packed(fullSignature)`.
    pub fn new_full(full_signature: MessageSignature) -> Self {
        Self { id: FULL_SIGNATURE, full_signature: Some(full_signature) }
    }

    /// `new Packed(id)`.
    pub fn new_id(id: i32) -> Self {
        Self { id, full_signature: None }
    }

    /// `Packed.read`.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Self {
        let i = buffer.read_var_int() - 1;
        if i == -1 {
            Self::new_full(MessageSignature::read(buffer))
        } else {
            Self::new_id(i)
        }
    }

    /// `Packed.write`.
    pub fn write(buffer: &mut FriendlyByteBuf, packed: &PackedSignature) {
        buffer.write_var_int(packed.id + 1);
        if let Some(sig) = &packed.full_signature {
            MessageSignature::write(buffer, sig);
        }
    }
}
