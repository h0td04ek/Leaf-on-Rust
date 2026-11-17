//! Перенос `ServerboundKeyPacket`.
//!
//! RSA/AES-шифрование — `network::encryption::crypt` (Crypt.*).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static KEY_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("key").unwrap())
});

/// `ServerboundKeyPacket` — зашифрованный AES-ключ и challenge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundKeyPacket {
    /// `keybytes` — AES-ключ, зашифрованный публичным ключом сервера.
    pub keybytes: Vec<u8>,
    /// `encryptedChallenge` — challenge, зашифрованный тем же ключом.
    pub encrypted_challenge: Vec<u8>,
}

impl ServerboundKeyPacket {
    /// `new ServerboundKeyPacket(secretKey, publicKey, challenge)` —
    /// шифрование через [`crate::network::encryption::crypt`].
    pub fn encrypt(
        secret_key: &[u8],
        public_key: &[u8],
        challenge: &[u8],
    ) -> Result<Self, crate::network::encryption::crypt_error::CryptError> {
        use crate::network::encryption::crypt;
        let keybytes = crypt::encrypt_using_key(public_key, secret_key)?;
        let encrypted_challenge = crypt::encrypt_using_key(public_key, challenge)?;
        Ok(Self { keybytes, encrypted_challenge })
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundKeyPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundKeyPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let keybytes = buf.read_byte_array().expect("invalid key bytes");
                let encrypted_challenge =
                    buf.read_byte_array().expect("invalid encrypted challenge");
                ServerboundKeyPacket { keybytes, encrypted_challenge }
            },
        )
    }
}

impl Packet for ServerboundKeyPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        KEY_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_byte_array(&self.keybytes);
        buf.write_byte_array(&self.encrypted_challenge);
    }
}
