//! Перенос `ClientboundChangeDifficultyPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::difficulty::Difficulty;

/// `ClientboundChangeDifficultyPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundChangeDifficultyPacket {
    /// `difficulty`.
    pub difficulty: Difficulty,
    /// `locked`.
    pub locked: bool,
}

impl ClientboundChangeDifficultyPacket {
    pub fn new(difficulty: Difficulty, locked: bool) -> Self {
        Self { difficulty, locked }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundChangeDifficultyPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundChangeDifficultyPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let difficulty = Difficulty::stream_codec().decode(buf);
                let locked = buf.read_bool();
                ClientboundChangeDifficultyPacket::new(difficulty, locked)
            },
        )
    }
}

impl Packet for ClientboundChangeDifficultyPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_change_difficulty::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        Difficulty::stream_codec().encode(buf, &self.difficulty);
        buf.write_bool(self.locked);
    }
}
