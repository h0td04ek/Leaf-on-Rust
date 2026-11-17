//! Перенос `ClientboundOpenBookPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::interaction_hand::InteractionHand;

/// `ClientboundOpenBookPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundOpenBookPacket {
    /// `hand`.
    pub hand: InteractionHand,
}

impl ClientboundOpenBookPacket {
    pub fn new(hand: InteractionHand) -> Self {
        Self { hand }
    }

    /// `getHand`.
    pub fn get_hand(&self) -> InteractionHand {
        self.hand
    }

    /// `STREAM_CODEC` (`readEnum`/`writeEnum` = VarInt ordinal).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundOpenBookPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundOpenBookPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundOpenBookPacket::new(InteractionHand::by_id(buf.read_var_int()))
            },
        )
    }
}

impl Packet for ClientboundOpenBookPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_open_book::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.hand.ordinal());
    }
}
