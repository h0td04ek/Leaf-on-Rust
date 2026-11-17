//! Перенос `ClientboundSetBorderLerpSizePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetBorderLerpSizePacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundSetBorderLerpSizePacket {
    /// `oldSize`.
    pub old_size: f64,
    /// `newSize`.
    pub new_size: f64,
    /// `lerpTime`.
    pub lerp_time: i64,
}

impl ClientboundSetBorderLerpSizePacket {
    /// `ClientboundSetBorderLerpSizePacket::new`.
    pub fn new(old_size: f64, new_size: f64, lerp_time: i64) -> Self {
        Self { old_size, new_size, lerp_time }
    }
    /// `oldSize` getter.
    pub fn get_old_size(&self) -> f64 {
        self.old_size
    }
    /// `newSize` getter.
    pub fn get_new_size(&self) -> f64 {
        self.new_size
    }
    /// `lerpTime` getter.
    pub fn get_lerp_time(&self) -> i64 {
        self.lerp_time
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetBorderLerpSizePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetBorderLerpSizePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let old_size = buf.read_double();
                let new_size = buf.read_double();
                let lerp_time = buf.read_var_long();

                ClientboundSetBorderLerpSizePacket::new(old_size, new_size, lerp_time)
            },
        )
    }
}

impl Packet for ClientboundSetBorderLerpSizePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_border_lerp_size::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_double(self.old_size);
        buf.write_double(self.new_size);
        buf.write_var_long(self.lerp_time);
    }
}
