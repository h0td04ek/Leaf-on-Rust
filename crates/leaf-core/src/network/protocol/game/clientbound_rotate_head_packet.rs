//! Перенос `ClientboundRotateHeadPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::util::mth;

/// `ClientboundRotateHeadPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundRotateHeadPacket {
    /// `entityId`.
    pub entity_id: i32,
    /// `yHeadRot` — packed byte.
    pub y_head_rot: i8,
}

impl ClientboundRotateHeadPacket {
    pub fn new(entity_id: i32, y_head_rot: i8) -> Self {
        Self { entity_id, y_head_rot }
    }

    /// `getYHeadRot` — `Mth.unpackDegrees`.
    pub fn get_y_head_rot(&self) -> f32 {
        mth::unpack_degrees(self.y_head_rot)
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundRotateHeadPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundRotateHeadPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let y_head_rot = buf.read_byte();

                ClientboundRotateHeadPacket::new(entity_id, y_head_rot)
            },
        )
    }
}

impl Packet for ClientboundRotateHeadPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_rotate_head::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        buf.write_byte(self.y_head_rot as i32);
    }
}
