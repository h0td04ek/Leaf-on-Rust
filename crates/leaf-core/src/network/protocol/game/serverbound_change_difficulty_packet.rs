//! Перенос `ServerboundChangeDifficultyPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::difficulty::Difficulty;

/// `ServerboundChangeDifficultyPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundChangeDifficultyPacket {
    /// `difficulty`.
    pub difficulty: Difficulty,
}

impl ServerboundChangeDifficultyPacket {
    pub fn new(difficulty: Difficulty) -> Self {
        Self { difficulty }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundChangeDifficultyPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundChangeDifficultyPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundChangeDifficultyPacket::new(Difficulty::stream_codec().decode(buf))
            },
        )
    }
}

impl Packet for ServerboundChangeDifficultyPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_change_difficulty::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        Difficulty::stream_codec().encode(buf, &self.difficulty);
    }
}
