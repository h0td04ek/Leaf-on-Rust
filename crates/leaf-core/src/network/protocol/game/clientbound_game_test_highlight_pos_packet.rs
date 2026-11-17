//! Перенос `ClientboundGameTestHighlightPosPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ClientboundGameTestHighlightPosPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundGameTestHighlightPosPacket {
    /// `absolutePos`.
    pub absolute_pos: crate::core::block_pos::BlockPos,
    /// `relativePos`.
    pub relative_pos: crate::core::block_pos::BlockPos,
}

impl ClientboundGameTestHighlightPosPacket {
    /// `ClientboundGameTestHighlightPosPacket::new`.
    pub fn new(absolute_pos: crate::core::block_pos::BlockPos, relative_pos: crate::core::block_pos::BlockPos) -> Self {
        Self { absolute_pos, relative_pos }
    }
    /// `absolutePos`.
    pub fn absolute_pos(&self) -> crate::core::block_pos::BlockPos {
        self.absolute_pos
    }
    /// `relativePos`.
    pub fn relative_pos(&self) -> crate::core::block_pos::BlockPos {
        self.relative_pos
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundGameTestHighlightPosPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundGameTestHighlightPosPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let absolute_pos = buf.read_block_pos();
                let relative_pos = buf.read_block_pos();
                ClientboundGameTestHighlightPosPacket::new(absolute_pos, relative_pos)
            },
        )
    }
}

impl Packet for ClientboundGameTestHighlightPosPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_game_test_highlight_pos::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.absolute_pos);
        buf.write_block_pos(&self.relative_pos);    }
}
