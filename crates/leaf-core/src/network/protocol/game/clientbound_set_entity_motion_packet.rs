//! Перенос `ClientboundSetEntityMotionPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::phys::vec3::Vec3;

/// `ClientboundSetEntityMotionPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundSetEntityMotionPacket {
    /// `id`.
    pub id: i32,
    /// `movement` (`readLpVec3`/`writeLpVec3`).
    pub movement: Vec3,
}

impl ClientboundSetEntityMotionPacket {
    /// `ClientboundSetEntityMotionPacket::new(id, movement)`.
    pub fn new(id: i32, movement: Vec3) -> Self {
        Self { id, movement }
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// `getMovement`.
    pub fn get_movement(&self) -> Vec3 {
        self.movement
    }

    /// `STREAM_CODEC` — VAR_INT + lpVec3.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetEntityMotionPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetEntityMotionPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let movement = buf.read_lp_vec3();
                ClientboundSetEntityMotionPacket::new(id, movement)
            },
        )
    }
}

impl Packet for ClientboundSetEntityMotionPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_entity_motion::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_lp_vec3(&self.movement);
    }
}
