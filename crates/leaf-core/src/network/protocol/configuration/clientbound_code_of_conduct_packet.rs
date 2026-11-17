//! Перенос `ClientboundCodeOfConductPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static CODE_OF_CONDUCT_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("code_of_conduct").unwrap())
});

/// `ClientboundCodeOfConductPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundCodeOfConductPacket {
    /// `codeOfConduct`.
    pub code_of_conduct: String,
}

impl ClientboundCodeOfConductPacket {
    pub fn new(code_of_conduct: String) -> Self {
        Self { code_of_conduct }
    }

    /// `STREAM_CODEC` (`STRING_UTF8`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundCodeOfConductPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundCodeOfConductPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundCodeOfConductPacket::new(buf.read_utf().expect("invalid code of conduct"))
            },
        )
    }
}

impl Packet for ClientboundCodeOfConductPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        CODE_OF_CONDUCT_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.code_of_conduct).expect("code of conduct too long");
    }
}
