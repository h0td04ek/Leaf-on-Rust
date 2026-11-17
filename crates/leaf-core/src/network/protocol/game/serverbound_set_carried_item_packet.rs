//! Перенос `ServerboundSetCarriedItemPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundSetCarriedItemPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundSetCarriedItemPacket {
    /// `slot` — short (0-8 hotbar).
    pub slot: i16,
}

impl ServerboundSetCarriedItemPacket {
    pub fn new(slot: i16) -> Self {
        Self { slot }
    }

    /// `getSlot`.
    pub fn get_slot(&self) -> i16 {
        self.slot
    }

    /// `STREAM_CODEC` (`readShort`/`writeShort`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSetCarriedItemPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSetCarriedItemPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundSetCarriedItemPacket::new(buf.read_short())
            },
        )
    }
}

impl Packet for ServerboundSetCarriedItemPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_set_carried_item::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_short(self.slot as i32);
    }
}
