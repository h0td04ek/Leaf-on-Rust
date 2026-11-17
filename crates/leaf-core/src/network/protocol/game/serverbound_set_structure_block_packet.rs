//! Перенос `ServerboundSetStructureBlockPacket`.

use crate::core::block_pos::BlockPos;
use crate::core::vec3i::Vec3i;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::util::mth;
use crate::world::level::block::entity::structure_block_entity::StructureBlockUpdateType;
use crate::world::level::block::{Mirror, Rotation, StructureMode};

const FLAG_IGNORE_ENTITIES: i32 = 1;
const FLAG_SHOW_AIR: i32 = 2;
const FLAG_SHOW_BOUNDING_BOX: i32 = 4;
const FLAG_STRICT: i32 = 8;
const OFFSET_SIZE: i32 = 48;
const SIZE_MAX: i32 = 48;

/// `ServerboundSetStructureBlockPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundSetStructureBlockPacket {
    /// `pos`.
    pub pos: BlockPos,
    /// `updateType`.
    pub update_type: StructureBlockUpdateType,
    /// `mode`.
    pub mode: StructureMode,
    /// `name`.
    pub name: String,
    /// `offset`.
    pub offset: BlockPos,
    /// `size`.
    pub size: Vec3i,
    /// `mirror`.
    pub mirror: Mirror,
    /// `rotation`.
    pub rotation: Rotation,
    /// `data`.
    pub data: String,
    /// `ignoreEntities`.
    pub ignore_entities: bool,
    /// `strict`.
    pub strict: bool,
    /// `showAir`.
    pub show_air: bool,
    /// `showBoundingBox`.
    pub show_bounding_box: bool,
    /// `integrity`.
    pub integrity: f32,
    /// `seed`.
    pub seed: i64,
}

impl ServerboundSetStructureBlockPacket {
    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pos: BlockPos,
        update_type: StructureBlockUpdateType,
        mode: StructureMode,
        name: String,
        offset: BlockPos,
        size: Vec3i,
        mirror: Mirror,
        rotation: Rotation,
        data: String,
        ignore_entities: bool,
        strict: bool,
        show_air: bool,
        show_bounding_box: bool,
        integrity: f32,
        seed: i64,
    ) -> Self {
        Self {
            pos,
            update_type,
            mode,
            name,
            offset,
            size,
            mirror,
            rotation,
            data,
            ignore_entities,
            strict,
            show_air,
            show_bounding_box,
            integrity,
            seed,
        }
    }

    fn enum_by_ordinal<T: Copy>(values: &[T], ordinal: i32) -> T {
        values[ordinal.rem_euclid(values.len() as i32) as usize]
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSetStructureBlockPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSetStructureBlockPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = buf.read_block_pos();
                let update_type = Self::enum_by_ordinal(
                    &StructureBlockUpdateType::VALUES,
                    buf.read_var_int(),
                );
                let mode = Self::enum_by_ordinal(&StructureMode::VALUES, buf.read_var_int());
                let name = buf.read_utf().expect("invalid utf");
                let offset = BlockPos::new(
                    mth::clamp_i32(buf.read_byte() as i32, -OFFSET_SIZE, OFFSET_SIZE),
                    mth::clamp_i32(buf.read_byte() as i32, -OFFSET_SIZE, OFFSET_SIZE),
                    mth::clamp_i32(buf.read_byte() as i32, -OFFSET_SIZE, OFFSET_SIZE),
                );
                let size = Vec3i::new(
                    mth::clamp_i32(buf.read_byte() as i32, 0, SIZE_MAX),
                    mth::clamp_i32(buf.read_byte() as i32, 0, SIZE_MAX),
                    mth::clamp_i32(buf.read_byte() as i32, 0, SIZE_MAX),
                );
                let mirror = Self::enum_by_ordinal(&Mirror::VALUES, buf.read_var_int());
                let rotation = Self::enum_by_ordinal(&Rotation::VALUES, buf.read_var_int());
                let data = buf.read_utf_n(128).expect("invalid utf");
                let integrity = mth::clamp_f32(buf.read_float(), 0.0, 1.0);
                let seed = buf.read_var_long();
                let flags = buf.read_byte() as i32;
                let ignore_entities = (flags & FLAG_IGNORE_ENTITIES) != 0;
                let strict = (flags & FLAG_STRICT) != 0;
                let show_air = (flags & FLAG_SHOW_AIR) != 0;
                let show_bounding_box = (flags & FLAG_SHOW_BOUNDING_BOX) != 0;
                ServerboundSetStructureBlockPacket::new(
                    pos,
                    update_type,
                    mode,
                    name,
                    offset,
                    size,
                    mirror,
                    rotation,
                    data,
                    ignore_entities,
                    strict,
                    show_air,
                    show_bounding_box,
                    integrity,
                    seed,
                )
            },
        )
    }
}

impl Packet for ServerboundSetStructureBlockPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_set_structure_block::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.pos);
        buf.write_var_int(self.update_type as i32);
        buf.write_var_int(self.mode as i32);
        buf.write_utf(&self.name).expect("invalid utf");
        buf.write_byte(self.offset.get_x());
        buf.write_byte(self.offset.get_y());
        buf.write_byte(self.offset.get_z());
        buf.write_byte(self.size.x);
        buf.write_byte(self.size.y);
        buf.write_byte(self.size.z);
        buf.write_var_int(self.mirror as i32);
        buf.write_var_int(self.rotation as i32);
        buf.write_utf_n(&self.data, 128).expect("invalid utf");
        buf.write_float(self.integrity);
        buf.write_var_long(self.seed);
        let mut i = 0;
        if self.ignore_entities {
            i |= FLAG_IGNORE_ENTITIES;
        }
        if self.show_air {
            i |= FLAG_SHOW_AIR;
        }
        if self.show_bounding_box {
            i |= FLAG_SHOW_BOUNDING_BOX;
        }
        if self.strict {
            i |= FLAG_STRICT;
        }
        buf.write_byte(i);
    }
}
