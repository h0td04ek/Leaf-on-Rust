//! Перенос `ServerboundSwingPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::interaction_hand::InteractionHand;

/// `ServerboundSwingPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundSwingPacket {
    /// `hand`.
    pub hand: InteractionHand,
}

impl ServerboundSwingPacket {
    pub fn new(hand: InteractionHand) -> Self {
        Self { hand }
    }

    /// `getHand`.
    pub fn get_hand(&self) -> InteractionHand {
        self.hand
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSwingPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSwingPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundSwingPacket::new(InteractionHand::by_id(buf.read_var_int()))
            },
        )
    }
}

impl Packet for ServerboundSwingPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_swing::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.hand.ordinal());
    }
}
