//! Перенос `ServerboundSetTestBlockPacket`.

use crate::core::block_pos::BlockPos;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::level::block::test_block_mode::TestBlockMode;

/// `ServerboundSetTestBlockPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundSetTestBlockPacket {
    /// `position`.
    pub position: BlockPos,
    /// `mode`.
    pub mode: TestBlockMode,
    /// `message`.
    pub message: String,
}

impl ServerboundSetTestBlockPacket {
    /// `new`.
    pub fn new(position: BlockPos, mode: TestBlockMode, message: String) -> Self {
        Self { position, mode, message }
    }

    /// `position`/`mode`/`message`.
    pub fn position(&self) -> BlockPos {
        self.position
    }
    /// `mode`.
    pub fn mode(&self) -> TestBlockMode {
        self.mode
    }
    /// `message`.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSetTestBlockPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSetTestBlockPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let position = buf.read_block_pos();
                let mode = TestBlockMode::stream_codec().decode(buf);
                let message = buf.read_utf().expect("invalid utf");
                ServerboundSetTestBlockPacket::new(position, mode, message)
            },
        )
    }
}

impl Packet for ServerboundSetTestBlockPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_set_test_block::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.position);
        TestBlockMode::stream_codec().encode(buf, &self.mode);
        buf.write_utf(&self.message).expect("invalid utf");
    }
}
