//! Перенос `ClientboundBlockEventPacket`.

use crate::core::block_pos::BlockPos;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::resources::identifier::Identifier;

/// `ClientboundBlockEventPacket`.
///
/// `block` — `ResourceKey` блока в реестре `Registries.BLOCK`
/// (на проводе — `ResourceLocation`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundBlockEventPacket {
    /// `pos`.
    pub pos: BlockPos,
    /// `b0`.
    pub b0: u8,
    /// `b1`.
    pub b1: u8,
    /// `block`.
    pub block: Identifier,
}

impl ClientboundBlockEventPacket {
    /// `ClientboundBlockEventPacket::new`.
    pub fn new(pos: BlockPos, block: Identifier, b0: u8, b1: u8) -> Self {
        Self { pos, b0, b1, block }
    }

    /// `getPos`.
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    /// `getB0`.
    pub fn get_b0(&self) -> u8 {
        self.b0
    }

    /// `getB1`.
    pub fn get_b1(&self) -> u8 {
        self.b1
    }

    /// `getBlock`.
    pub fn get_block(&self) -> &Identifier {
        &self.block
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundBlockEventPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundBlockEventPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = buf.read_block_pos();
                let b0 = buf.read_byte() as u8;
                let b1 = buf.read_byte() as u8;
                let block = buf.read_identifier().expect("invalid identifier");

                ClientboundBlockEventPacket::new(pos, block, b0, b1)
            },
        )
    }
}

impl Packet for ClientboundBlockEventPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_block_event::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.pos);
        buf.write_byte(self.b0 as i32);
        buf.write_byte(self.b1 as i32);
        buf.write_identifier(&self.block).expect("invalid identifier");
    }
}
