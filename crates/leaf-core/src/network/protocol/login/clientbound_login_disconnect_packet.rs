//! Перенос `ClientboundLoginDisconnectPacket`.
//! Paper: localized codec (adventure$locale) — используем стандартный
//! component stream codec (locale-aware вариант добавим при порте locale).

use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static LOGIN_DISCONNECT_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::CLIENTBOUND,
        Identifier::with_default_namespace("login_disconnect").unwrap(),
    )
});

/// `ClientboundLoginDisconnectPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundLoginDisconnectPacket {
    /// `reason`.
    pub reason: Component,
}

impl ClientboundLoginDisconnectPacket {
    pub fn new(reason: Component) -> Self {
        Self { reason }
    }

    /// `reason()`.
    pub fn reason(&self) -> &Component {
        &self.reason
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundLoginDisconnectPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundLoginDisconnectPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundLoginDisconnectPacket::new(
                    component_json::component_stream_codec().decode(buf),
                )
            },
        )
    }
}

impl Packet for ClientboundLoginDisconnectPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        LOGIN_DISCONNECT_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        component_json::component_stream_codec().encode(buf, &self.reason);
    }

    fn is_terminal(&self) -> bool {
        true
    }
}
