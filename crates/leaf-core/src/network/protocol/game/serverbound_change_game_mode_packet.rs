//! Перенос `ServerboundChangeGameModePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::level::game_type::GameType;

/// `ServerboundChangeGameModePacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundChangeGameModePacket {
    /// `mode`.
    pub mode: GameType,
}

impl ServerboundChangeGameModePacket {
    pub fn new(mode: GameType) -> Self {
        Self { mode }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundChangeGameModePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundChangeGameModePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundChangeGameModePacket::new(GameType::stream_codec().decode(buf))
            },
        )
    }
}

impl Packet for ServerboundChangeGameModePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_change_game_mode::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        GameType::stream_codec().encode(buf, &self.mode);
    }
}
