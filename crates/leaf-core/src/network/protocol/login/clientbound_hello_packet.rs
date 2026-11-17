//! Перенос `ClientboundHelloPacket` — серверный public key + challenge.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::encryption::crypt;
use crate::network::encryption::crypt_error::CryptError;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

/// `MAX_SERVER_ID_LENGTH` (readUtf(20)).
const MAX_SERVER_ID_LENGTH: usize = 20;

static HELLO_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("hello").unwrap())
});

/// `ClientboundHelloPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundHelloPacket {
    /// `serverId`.
    pub server_id: String,
    /// `publicKey` — X509 DER RSA-ключ сервера.
    pub public_key: Vec<u8>,
    /// `challenge` — случайные байты для проверки клиента.
    pub challenge: Vec<u8>,
    /// `shouldAuthenticate`.
    pub should_authenticate: bool,
}

impl ClientboundHelloPacket {
    pub fn new(
        server_id: String,
        public_key: Vec<u8>,
        challenge: Vec<u8>,
        should_authenticate: bool,
    ) -> Self {
        Self { server_id, public_key, challenge, should_authenticate }
    }

    /// `getPublicKey()` — парсит X509 DER → RsaPublicKey.
    pub fn get_public_key(&self) -> Result<rsa::RsaPublicKey, CryptError> {
        crypt::byte_to_public_key(&self.public_key)
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundHelloPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundHelloPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let server_id = buf
                    .read_utf_n(MAX_SERVER_ID_LENGTH)
                    .expect("invalid server id");
                let public_key = buf.read_byte_array().expect("invalid public key");
                let challenge = buf.read_byte_array().expect("invalid challenge");
                let should_authenticate = buf.read_bool();
                ClientboundHelloPacket::new(server_id, public_key, challenge, should_authenticate)
            },
        )
    }
}

impl Packet for ClientboundHelloPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        HELLO_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf_n(&self.server_id, MAX_SERVER_ID_LENGTH)
            .expect("server id too long");
        buf.write_byte_array(&self.public_key);
        buf.write_byte_array(&self.challenge);
        buf.write_bool(self.should_authenticate);
    }
}
