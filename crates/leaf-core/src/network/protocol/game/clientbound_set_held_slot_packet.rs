//! Перенос `ClientboundSetHeldSlotPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetHeldSlotPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundSetHeldSlotPacket {
    /// `slot`.
    pub slot: i32,
}

impl ClientboundSetHeldSlotPacket {
    pub fn new(slot: i32) -> Self {
        Self { slot }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetHeldSlotPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetHeldSlotPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| ClientboundSetHeldSlotPacket::new(buf.read_var_int()),
        )
    }
}

impl Packet for ClientboundSetHeldSlotPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_held_slot::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.slot);
    }
}
