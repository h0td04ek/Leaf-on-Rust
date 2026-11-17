//! Перенос `ClientboundSystemChatPacket`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSystemChatPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSystemChatPacket {
    /// `content`.
    pub content: Component,
    /// `overlay` — actionbar vs chat.
    pub overlay: bool,
}

impl ClientboundSystemChatPacket {
    pub fn new(content: Component, overlay: bool) -> Self {
        Self { content, overlay }
    }

    /// `STREAM_CODEC` (`ComponentSerialization.TRUSTED_STREAM_CODEC` + bool).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSystemChatPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSystemChatPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let content = component_json::component_stream_codec().decode(buf);
                let overlay = buf.read_bool();
                ClientboundSystemChatPacket::new(content, overlay)
            },
        )
    }
}

impl Packet for ClientboundSystemChatPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_system_chat::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        component_json::component_stream_codec().encode(buf, &self.content);
        buf.write_bool(self.overlay);
    }

    fn is_skippable(&self) -> bool {
        true
    }
}
