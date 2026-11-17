//! Перенос `ClientboundMoveVehiclePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::phys::vec3::Vec3;

/// `ClientboundMoveVehiclePacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundMoveVehiclePacket {
    /// `position`.
    pub position: Vec3,
    /// `yRot`.
    pub y_rot: f32,
    /// `xRot`.
    pub x_rot: f32,
}

impl ClientboundMoveVehiclePacket {
    pub fn new(position: Vec3, y_rot: f32, x_rot: f32) -> Self {
        Self { position, y_rot, x_rot }
    }

    /// `STREAM_CODEC` (`Vec3.STREAM_CODEC` + 2 float).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundMoveVehiclePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundMoveVehiclePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let position = Vec3::stream_codec().decode(buf);
                let y_rot = buf.read_float();
                let x_rot = buf.read_float();
                ClientboundMoveVehiclePacket::new(position, y_rot, x_rot)
            },
        )
    }
}

impl Packet for ClientboundMoveVehiclePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_move_vehicle::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        Vec3::stream_codec().encode(buf, &self.position);
        buf.write_float(self.y_rot);
        buf.write_float(self.x_rot);
    }
}
