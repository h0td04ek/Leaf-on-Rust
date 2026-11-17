//! Перенос `ClientIntentionPacket`.

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::handshake::client_intent::ClientIntent;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::network::var_int;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

/// `MAX_HOST_LENGTH` (Spigot: Short.MAX_VALUE).
const MAX_HOST_LENGTH: usize = i16::MAX as usize;

static CLIENT_INTENTION_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::SERVERBOUND,
        Identifier::with_default_namespace("intention").unwrap(),
    )
});

/// `ClientIntentionPacket` — handshake-запрос клиента.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientIntentionPacket {
    pub protocol_version: i32,
    pub host_name: String,
    pub port: i32,
    pub intention: ClientIntent,
}

impl ClientIntentionPacket {
    pub fn new(
        protocol_version: i32,
        host_name: String,
        port: i32,
        intention: ClientIntent,
    ) -> Self {
        Self { protocol_version, host_name, port, intention }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<FriendlyByteBuf, ClientIntentionPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientIntentionPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let protocol_version = var_int::read(buf);
                let host_name = buf
                    .read_utf_n(MAX_HOST_LENGTH)
                    .expect("invalid hostname");
                let port = buf.read_unsigned_short();
                let intention =
                    ClientIntent::by_id(var_int::read(buf)).expect("unknown connection intent");
                ClientIntentionPacket::new(protocol_version, host_name, port, intention)
            },
        )
    }
}

impl Packet for ClientIntentionPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        CLIENT_INTENTION_TYPE.clone()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        var_int::write(buf, self.protocol_version);
        buf.write_utf_n(&self.host_name, MAX_HOST_LENGTH)
            .expect("hostname too long");
        buf.write_short(self.port);
        var_int::write(buf, self.intention.id());
    }

    fn is_terminal(&self) -> bool {
        true
    }
}
