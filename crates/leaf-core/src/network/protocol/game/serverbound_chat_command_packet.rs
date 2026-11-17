//! Перенос `ServerboundChatCommandPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `MAX_CHAT_PACKET_INPUT_SIZE` (Paper, default 256).
pub const MAX_CHAT_PACKET_INPUT_SIZE: usize = 256;

/// `ServerboundChatCommandPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundChatCommandPacket {
    /// `command`.
    pub command: String,
}

impl ServerboundChatCommandPacket {
    pub fn new(command: String) -> Self {
        Self { command }
    }

    /// `command`.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// `STREAM_CODEC` — utf(MAX_CHAT_PACKET_INPUT_SIZE).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundChatCommandPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundChatCommandPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundChatCommandPacket::new(
                    buf.read_utf_n(MAX_CHAT_PACKET_INPUT_SIZE).expect("invalid command"),
                )
            },
        )
    }
}

impl Packet for ServerboundChatCommandPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_chat_command::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf_n(&self.command, MAX_CHAT_PACKET_INPUT_SIZE).expect("command too long");
    }
}
