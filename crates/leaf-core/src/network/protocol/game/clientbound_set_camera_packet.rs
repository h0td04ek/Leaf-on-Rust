//! Перенос `ClientboundSetCameraPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetCameraPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundSetCameraPacket {
    /// `cameraId`.
    pub camera_id: i32,
}

impl ClientboundSetCameraPacket {
    /// `ClientboundSetCameraPacket::new`.
    pub fn new(camera_id: i32) -> Self {
        Self { camera_id }
    }
    /// `cameraId` getter.
    pub fn get_camera_id(&self) -> i32 {
        self.camera_id
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetCameraPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetCameraPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let camera_id = buf.read_var_int();

                ClientboundSetCameraPacket::new(camera_id)
            },
        )
    }
}

impl Packet for ClientboundSetCameraPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_camera::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.camera_id);
    }
}
