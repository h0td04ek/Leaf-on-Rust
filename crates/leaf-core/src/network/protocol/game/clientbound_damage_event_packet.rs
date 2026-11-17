//! Перенос `ClientboundDamageEventPacket`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::damagesource::damage_type::DamageType;
use crate::world::phys::vec3::Vec3;

/// `ClientboundDamageEventPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundDamageEventPacket {
    /// `entityId`.
    pub entity_id: i32,
    /// `sourceType` — `Holder<DamageType>`.
    pub source_type: Holder<DamageType>,
    /// `sourceCauseId` — -1 если нет.
    pub source_cause_id: i32,
    /// `sourceDirectId` — -1 если нет.
    pub source_direct_id: i32,
    /// `sourcePosition`.
    pub source_position: Option<Vec3>,
}

impl ClientboundDamageEventPacket {
    /// `ClientboundDamageEventPacket::new`.
    pub fn new(
        entity_id: i32,
        source_type: Holder<DamageType>,
        source_cause_id: i32,
        source_direct_id: i32,
        source_position: Option<Vec3>,
    ) -> Self {
        Self { entity_id, source_type, source_cause_id, source_direct_id, source_position }
    }

    /// `entityId`/`sourceType`/`sourceCauseId`/`sourceDirectId`/`sourcePosition`.
    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }
    /// `sourceType`.
    pub fn source_type(&self) -> &Holder<DamageType> {
        &self.source_type
    }
    /// `sourceCauseId`.
    pub fn source_cause_id(&self) -> i32 {
        self.source_cause_id
    }
    /// `sourceDirectId`.
    pub fn source_direct_id(&self) -> i32 {
        self.source_direct_id
    }
    /// `sourcePosition`.
    pub fn source_position(&self) -> Option<&Vec3> {
        self.source_position.as_ref()
    }

    fn read_optional_entity_id(buf: &mut FriendlyByteBuf) -> i32 {
        buf.read_var_int() - 1
    }

    fn write_optional_entity_id(buf: &mut FriendlyByteBuf, optional_entity_id: i32) {
        buf.write_var_int(optional_entity_id + 1);
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundDamageEventPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundDamageEventPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let source_type = DamageType::stream_codec().decode(buf);
                let source_cause_id = Self::read_optional_entity_id(buf);
                let source_direct_id = Self::read_optional_entity_id(buf);
                let source_position = buf.read_optional(|b| {
                    Vec3::new(b.read_double(), b.read_double(), b.read_double())
                });
                ClientboundDamageEventPacket::new(
                    entity_id,
                    source_type,
                    source_cause_id,
                    source_direct_id,
                    source_position,
                )
            },
        )
    }
}

impl Packet for ClientboundDamageEventPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_damage_event::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        DamageType::stream_codec().encode(buf, &self.source_type);
        Self::write_optional_entity_id(buf, self.source_cause_id);
        Self::write_optional_entity_id(buf, self.source_direct_id);
        buf.write_optional(&self.source_position, |b, v| {
            b.write_double(v.x);
            b.write_double(v.y);
            b.write_double(v.z);
        });
    }
}
