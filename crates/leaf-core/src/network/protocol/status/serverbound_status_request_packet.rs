//! Перенос `ServerboundStatusRequestPacket`.

use crate::network::codec::stream_codec::{unit, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static STATUS_REQUEST_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("status_request").unwrap())
});

/// `ServerboundStatusRequestPacket` — синглтон `INSTANCE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundStatusRequestPacket;

impl ServerboundStatusRequestPacket {
    /// `INSTANCE`.
    pub const INSTANCE: ServerboundStatusRequestPacket = ServerboundStatusRequestPacket;

    /// `STREAM_CODEC` (`StreamCodec.unit(INSTANCE)`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundStatusRequestPacket> {
        unit(ServerboundStatusRequestPacket)
    }
}

impl Packet for ServerboundStatusRequestPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        STATUS_REQUEST_TYPE.clone()
    }

    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}
}
