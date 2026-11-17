//! Перенос `ServerboundChatSessionUpdatePacket`.

use crate::network::chat::remote_chat_session::RemoteChatSessionData;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundChatSessionUpdatePacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundChatSessionUpdatePacket {
    /// `chatSession` — `RemoteChatSession.Data`.
    pub chat_session: RemoteChatSessionData,
}

impl ServerboundChatSessionUpdatePacket {
    pub fn new(chat_session: RemoteChatSessionData) -> Self {
        Self { chat_session }
    }

    /// `chatSession`.
    pub fn chat_session(&self) -> &RemoteChatSessionData {
        &self.chat_session
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundChatSessionUpdatePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundChatSessionUpdatePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let chat_session = RemoteChatSessionData::read(buf).expect("invalid chat session");

                ServerboundChatSessionUpdatePacket::new(chat_session)
            },
        )
    }
}

impl Packet for ServerboundChatSessionUpdatePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_chat_session_update::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        self.chat_session.write(buf).expect("chat session write");
    }
}
