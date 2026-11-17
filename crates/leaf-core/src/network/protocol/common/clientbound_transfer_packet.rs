//! Перенос `ClientboundTransferPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::network::var_int;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static TRANSFER_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("transfer").unwrap())
});

/// `ClientboundTransferPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundTransferPacket {
    pub host: String,
    pub port: i32,
}

impl ClientboundTransferPacket {
    pub fn new(host: impl Into<String>, port: i32) -> Self {
        Self { host: host.into(), port }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundTransferPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundTransferPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let host = buf.read_utf().expect("invalid host");
                let port = var_int::read(buf);
                ClientboundTransferPacket::new(host, port)
            },
        )
    }
}

impl Packet for ClientboundTransferPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        TRANSFER_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.host).expect("host too long");
        var_int::write(buf, self.port);
    }
}
