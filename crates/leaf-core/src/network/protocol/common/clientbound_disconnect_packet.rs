//! Перенос `ClientboundDisconnectPacket`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static DISCONNECT_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("disconnect").unwrap())
});

/// `ClientboundDisconnectPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundDisconnectPacket {
    pub reason: Component,
}

impl ClientboundDisconnectPacket {
    pub fn new(reason: Component) -> Self {
        Self { reason }
    }

    /// `STREAM_CODEC` (`ComponentSerialization.TRUSTED_CONTEXT_FREE_STREAM_CODEC`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundDisconnectPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundDisconnectPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundDisconnectPacket::new(component_json::component_stream_codec().decode(buf))
            },
        )
    }
}

impl Packet for ClientboundDisconnectPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        DISCONNECT_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        component_json::component_stream_codec().encode(buf, &self.reason);
    }

    fn is_terminal(&self) -> bool {
        true
    }
}
