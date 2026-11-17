//! `ClientboundAddEntityPacket` — спавн сущности.

use crate::core::holder::Holder;
use crate::entity::entity_type::EntityType;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::util::mth;
use crate::util::uuid::Uuid;
use crate::world::phys::vec3::Vec3;

/// `ClientboundAddEntityPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundAddEntityPacket {
    /// `id`.
    pub id: i32,
    /// `uuid`.
    pub uuid: Uuid,
    /// `type` — `Holder<EntityType>`.
    pub type_: Holder<EntityType>,
    /// `x`.
    pub x: f64,
    /// `y`.
    pub y: f64,
    /// `z`.
    pub z: f64,
    /// `movement`.
    pub movement: Vec3,
    /// `xRot` — packed degrees.
    pub x_rot: i8,
    /// `yRot` — packed degrees.
    pub y_rot: i8,
    /// `yHeadRot` — packed degrees.
    pub y_head_rot: i8,
    /// `data`.
    pub data: i32,
}

impl ClientboundAddEntityPacket {
    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i32,
        uuid: Uuid,
        type_: Holder<EntityType>,
        x: f64,
        y: f64,
        z: f64,
        movement: Vec3,
        x_rot: f32,
        y_rot: f32,
        y_head_rot: f64,
        data: i32,
    ) -> Self {
        Self {
            id, uuid, type_, x, y, z, movement,
            x_rot: mth::pack_degrees(x_rot),
            y_rot: mth::pack_degrees(y_rot),
            y_head_rot: mth::pack_degrees(y_head_rot as f32),
            data,
        }
    }

    /// `getXRot` — `Mth.unpackDegrees`.
    pub fn get_x_rot(&self) -> f32 {
        mth::unpack_degrees(self.x_rot)
    }

    /// `getYRot` — `Mth.unpackDegrees`.
    pub fn get_y_rot(&self) -> f32 {
        mth::unpack_degrees(self.y_rot)
    }

    /// `getYHeadRot` — `Mth.unpackDegrees`.
    pub fn get_y_head_rot(&self) -> f32 {
        mth::unpack_degrees(self.y_head_rot)
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundAddEntityPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundAddEntityPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let uuid = buf.read_uuid();
                let type_ = EntityType::stream_codec().decode(buf);
                let x = buf.read_double();
                let y = buf.read_double();
                let z = buf.read_double();
                let movement = buf.read_lp_vec3();
                let x_rot = buf.read_byte();
                let y_rot = buf.read_byte();
                let y_head_rot = buf.read_byte();
                let data = buf.read_var_int();
                ClientboundAddEntityPacket {
                    id, uuid, type_, x, y, z, movement,
                    x_rot, y_rot, y_head_rot, data,
                }
            },
        )
    }
}

impl Packet for ClientboundAddEntityPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_add_entity::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_uuid(&self.uuid);
        EntityType::stream_codec().encode(buf, &self.type_);
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_lp_vec3(&self.movement);
        buf.write_byte(self.x_rot as i32);
        buf.write_byte(self.y_rot as i32);
        buf.write_byte(self.y_head_rot as i32);
        buf.write_var_int(self.data);
    }
}
