//! Перенос `ClientboundSetBorderSizePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetBorderSizePacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundSetBorderSizePacket {
    /// `size`.
    pub size: f64,
}

impl ClientboundSetBorderSizePacket {
    /// `ClientboundSetBorderSizePacket::new`.
    pub fn new(size: f64) -> Self {
        Self { size }
    }
    /// `size` getter.
    pub fn get_size(&self) -> f64 {
        self.size
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetBorderSizePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetBorderSizePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let size = buf.read_double();

                ClientboundSetBorderSizePacket::new(size)
            },
        )
    }
}

impl Packet for ClientboundSetBorderSizePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_border_size::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_double(self.size);
    }
}
