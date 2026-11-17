//! Перенос `ServerboundPaddleBoatPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundPaddleBoatPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPaddleBoatPacket {
    /// `left`.
    pub left: bool,
    /// `right`.
    pub right: bool,
}

impl ServerboundPaddleBoatPacket {
    pub fn new(left: bool, right: bool) -> Self {
        Self { left, right }
    }

    /// `getLeft`.
    pub fn get_left(&self) -> bool {
        self.left
    }

    /// `getRight`.
    pub fn get_right(&self) -> bool {
        self.right
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundPaddleBoatPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundPaddleBoatPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let left = buf.read_bool();
                let right = buf.read_bool();

                ServerboundPaddleBoatPacket::new(left, right)
            },
        )
    }
}

impl Packet for ServerboundPaddleBoatPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_paddle_boat::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_bool(self.left);
        buf.write_bool(self.right);
    }
}
