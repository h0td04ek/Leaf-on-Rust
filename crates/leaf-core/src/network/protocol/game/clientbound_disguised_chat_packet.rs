//! Перенос `ClientboundDisguisedChatPacket`.

use crate::network::chat::chat_type::ChatTypeBound;
use crate::network::chat::component::Component;
use crate::network::chat::component_json::component_stream_codec;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundDisguisedChatPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundDisguisedChatPacket {
    /// `message`.
    pub message: Component,
    /// `chatType`.
    pub chat_type: ChatTypeBound,
}

impl ClientboundDisguisedChatPacket {
    pub fn new(message: Component, chat_type: ChatTypeBound) -> Self {
        Self { message, chat_type }
    }

    /// `isSkippable` → true.
    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundDisguisedChatPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundDisguisedChatPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let message = component_stream_codec().decode(buf);
                let chat_type = ChatTypeBound::stream_codec().decode(buf);

                ClientboundDisguisedChatPacket::new(message, chat_type)
            },
        )
    }
}

impl Packet for ClientboundDisguisedChatPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_disguised_chat::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        component_stream_codec().encode(buf, &self.message);
        ChatTypeBound::stream_codec().encode(buf, &self.chat_type);
    }

    fn is_skippable(&self) -> bool {
        true
    }
}
