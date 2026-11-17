//! Перенос `ServerboundChatAckPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundChatAckPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundChatAckPacket {
    /// `offset` — последний подтверждённый offset подписи.
    pub offset: i32,
}

impl ServerboundChatAckPacket {
    pub fn new(offset: i32) -> Self {
        Self { offset }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundChatAckPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundChatAckPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| ServerboundChatAckPacket::new(buf.read_var_int()),
        )
    }
}

impl Packet for ServerboundChatAckPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_chat_ack::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.offset);
    }
}
