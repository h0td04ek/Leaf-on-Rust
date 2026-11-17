//! Перенос `ClientboundUpdateAttributesPacket`.

use crate::core::holder::Holder;
use crate::entity::ai::attributes::attribute::Attribute;
use crate::entity::ai::attributes::attribute_modifier::AttributeModifier;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundUpdateAttributesPacket.AttributeSnapshot`.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeSnapshot {
    /// `attribute` — `Holder<Attribute>`.
    pub attribute: Holder<Attribute>,
    /// `base`.
    pub base: f64,
    /// `modifiers`.
    pub modifiers: Vec<AttributeModifier>,
}

impl AttributeSnapshot {
    /// `AttributeSnapshot::new`.
    pub fn new(attribute: Holder<Attribute>, base: f64, modifiers: Vec<AttributeModifier>) -> Self {
        Self { attribute, base, modifiers }
    }

    /// `MODIFIER_STREAM_CODEC` — коллекция модификаторов.
    fn modifiers_stream_codec() -> impl StreamCodec<FriendlyByteBuf, Vec<AttributeModifier>> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, list: &Vec<AttributeModifier>| {
                buf.write_collection(list, |b, m| {
                    AttributeModifier::stream_codec().encode(b, m);
                });
            },
            |buf: &mut FriendlyByteBuf| {
                buf.read_list(|b| AttributeModifier::stream_codec().decode(b))
            },
        )
    }

    /// `STREAM_CODEC` — Holder<Attribute> + DOUBLE + modifier list.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, AttributeSnapshot> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, s: &AttributeSnapshot| {
                Attribute::stream_codec().encode(buf, &s.attribute);
                buf.write_double(s.base);
                Self::modifiers_stream_codec().encode(buf, &s.modifiers);
            },
            |buf: &mut FriendlyByteBuf| {
                let attribute = Attribute::stream_codec().decode(buf);
                let base = buf.read_double();
                let modifiers = Self::modifiers_stream_codec().decode(buf);
                AttributeSnapshot::new(attribute, base, modifiers)
            },
        )
    }
}

/// `ClientboundUpdateAttributesPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundUpdateAttributesPacket {
    /// `entityId`.
    pub entity_id: i32,
    /// `attributes`.
    pub attributes: Vec<AttributeSnapshot>,
}

impl ClientboundUpdateAttributesPacket {
    /// `ClientboundUpdateAttributesPacket::new`.
    pub fn new(entity_id: i32, attributes: Vec<AttributeSnapshot>) -> Self {
        Self { entity_id, attributes }
    }

    /// `getEntityId`.
    pub fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    /// `getValues`.
    pub fn get_values(&self) -> &[AttributeSnapshot] {
        &self.attributes
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundUpdateAttributesPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundUpdateAttributesPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let attributes = buf.read_list(|b| AttributeSnapshot::stream_codec().decode(b));
                ClientboundUpdateAttributesPacket::new(entity_id, attributes)
            },
        )
    }
}

impl Packet for ClientboundUpdateAttributesPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_update_attributes::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        buf.write_collection(&self.attributes, |b, a| {
            AttributeSnapshot::stream_codec().encode(b, a);
        });
    }
}
