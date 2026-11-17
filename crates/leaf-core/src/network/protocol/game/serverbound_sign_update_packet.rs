//! Перенос `ServerboundSignUpdatePacket`.

use crate::core::block_pos::BlockPos;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `MAX_STRING_LENGTH`.
const MAX_STRING_LENGTH: usize = 384;
/// `LINES_COUNT`.
const LINES_COUNT: usize = 4;

/// `ServerboundSignUpdatePacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundSignUpdatePacket {
    /// `pos`.
    pub pos: BlockPos,
    /// `lines` — ровно 4 строки.
    pub lines: [String; 4],
    /// `isFrontText`.
    pub is_front_text: bool,
}

impl ServerboundSignUpdatePacket {
    /// `new ServerboundSignUpdatePacket(pos, isFrontText, l1..l4)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pos: BlockPos,
        is_front_text: bool,
        line1: String,
        line2: String,
        line3: String,
        line4: String,
    ) -> Self {
        Self { pos, lines: [line1, line2, line3, line4], is_front_text }
    }

    /// `getPos`.
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    /// `isFrontText`.
    pub fn is_front_text(&self) -> bool {
        self.is_front_text
    }

    /// `getLines`.
    pub fn get_lines(&self) -> &[String; 4] {
        &self.lines
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSignUpdatePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSignUpdatePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = buf.read_block_pos();
                let is_front_text = buf.read_bool();
                let mut lines: [String; 4] = Default::default();
                for line in lines.iter_mut() {
                    *line = buf.read_utf_n(MAX_STRING_LENGTH).expect("invalid sign line");
                }
                ServerboundSignUpdatePacket { pos, lines, is_front_text }
            },
        )
    }
}

impl Packet for ServerboundSignUpdatePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_sign_update::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.pos);
        buf.write_bool(self.is_front_text);
        for line in &self.lines {
            buf.write_utf_n(line, MAX_STRING_LENGTH).expect("sign line too long");
        }
    }
}
