//! Перенос `ClientboundSetBorderCenterPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetBorderCenterPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundSetBorderCenterPacket {
    /// `newCenterX`.
    pub new_center_x: f64,
    /// `newCenterZ`.
    pub new_center_z: f64,
}

impl ClientboundSetBorderCenterPacket {
    /// `ClientboundSetBorderCenterPacket::new`.
    pub fn new(new_center_x: f64, new_center_z: f64) -> Self {
        Self { new_center_x, new_center_z }
    }
    /// `newCenterX` getter.
    pub fn get_new_center_x(&self) -> f64 {
        self.new_center_x
    }
    /// `newCenterZ` getter.
    pub fn get_new_center_z(&self) -> f64 {
        self.new_center_z
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetBorderCenterPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetBorderCenterPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let new_center_x = buf.read_double();
                let new_center_z = buf.read_double();

                ClientboundSetBorderCenterPacket::new(new_center_x, new_center_z)
            },
        )
    }
}

impl Packet for ClientboundSetBorderCenterPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_border_center::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_double(self.new_center_x);
        buf.write_double(self.new_center_z);
    }
}
