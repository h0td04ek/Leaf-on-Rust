//! Перенос `ClientboundOpenSignEditorPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ClientboundOpenSignEditorPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundOpenSignEditorPacket {
    /// `pos`.
    pub pos: crate::core::block_pos::BlockPos,
    /// `isFrontText`.
    pub is_front_text: bool,
}

impl ClientboundOpenSignEditorPacket {
    /// `ClientboundOpenSignEditorPacket::new`.
    pub fn new(pos: crate::core::block_pos::BlockPos, is_front_text: bool) -> Self {
        Self { pos, is_front_text }
    }
    /// `getPos`.
    pub fn get_pos(&self) -> crate::core::block_pos::BlockPos { self.pos }
    /// `isFrontText`.
    pub fn is_front_text(&self) -> bool { self.is_front_text }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundOpenSignEditorPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundOpenSignEditorPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = buf.read_block_pos();
                let is_front_text = buf.read_bool();
                ClientboundOpenSignEditorPacket::new(pos, is_front_text)
            },
        )
    }
}

impl Packet for ClientboundOpenSignEditorPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_open_sign_editor::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.pos);
        buf.write_bool(self.is_front_text);    }
}
