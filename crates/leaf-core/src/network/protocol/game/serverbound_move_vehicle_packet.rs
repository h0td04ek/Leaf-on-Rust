//! Перенос `ServerboundMoveVehiclePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::phys::vec3::Vec3;

/// `ServerboundMoveVehiclePacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ServerboundMoveVehiclePacket {
    /// `position`.
    pub position: Vec3,
    /// `yRot`.
    pub y_rot: f32,
    /// `xRot`.
    pub x_rot: f32,
    /// `onGround`.
    pub on_ground: bool,
}

impl ServerboundMoveVehiclePacket {
    pub fn new(position: Vec3, y_rot: f32, x_rot: f32, on_ground: bool) -> Self {
        Self { position, y_rot, x_rot, on_ground }
    }

    /// `position`.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// `STREAM_CODEC` (`Vec3.STREAM_CODEC` + FLOAT×2 + BOOL).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundMoveVehiclePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundMoveVehiclePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let position = buf.read_vec3();
                let y_rot = buf.read_float();
                let x_rot = buf.read_float();
                let on_ground = buf.read_bool();

                ServerboundMoveVehiclePacket::new(position, y_rot, x_rot, on_ground)
            },
        )
    }
}

impl Packet for ServerboundMoveVehiclePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_move_vehicle::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_vec3(&self.position);
        buf.write_float(self.y_rot);
        buf.write_float(self.x_rot);
        buf.write_bool(self.on_ground);
    }
}
