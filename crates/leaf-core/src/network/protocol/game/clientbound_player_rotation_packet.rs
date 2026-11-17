//! Перенос `ClientboundPlayerRotationPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundPlayerRotationPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundPlayerRotationPacket {
    /// `yRot`.
    pub y_rot: f32,
    /// `relativeY`.
    pub relative_y: bool,
    /// `xRot`.
    pub x_rot: f32,
    /// `relativeX`.
    pub relative_x: bool,
}

impl ClientboundPlayerRotationPacket {
    /// `STREAM_CODEC` — FLOAT + BOOL + FLOAT + BOOL.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlayerRotationPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPlayerRotationPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let y_rot = buf.read_float();
                let relative_y = buf.read_bool();
                let x_rot = buf.read_float();
                let relative_x = buf.read_bool();
                ClientboundPlayerRotationPacket { y_rot, relative_y, x_rot, relative_x }
            },
        )
    }
}

impl Packet for ClientboundPlayerRotationPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_player_rotation::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_float(self.y_rot);
        buf.write_bool(self.relative_y);
        buf.write_float(self.x_rot);
        buf.write_bool(self.relative_x);
    }
}
