//! Перенос `ClientboundBlockChangedAckPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundBlockChangedAckPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundBlockChangedAckPacket {
    /// `sequence`.
    pub sequence: i32,
}

impl ClientboundBlockChangedAckPacket {
    /// `ClientboundBlockChangedAckPacket::new`.
    pub fn new(sequence: i32) -> Self {
        Self { sequence }
    }
    /// `sequence` getter.
    pub fn get_sequence(&self) -> i32 {
        self.sequence
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundBlockChangedAckPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundBlockChangedAckPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let sequence = buf.read_var_int();

                ClientboundBlockChangedAckPacket::new(sequence)
            },
        )
    }
}

impl Packet for ClientboundBlockChangedAckPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_block_changed_ack::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.sequence);
    }
}
