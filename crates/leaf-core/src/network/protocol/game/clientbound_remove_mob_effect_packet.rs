//! Перенос `ClientboundRemoveMobEffectPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::effect::mob_effect::MobEffect;

/// `ClientboundRemoveMobEffectPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundRemoveMobEffectPacket {
    /// `entityId`.
    pub entity_id: i32,
    /// `effect`.
    pub effect: crate::core::holder::Holder<crate::world::effect::mob_effect::MobEffect>,
}

impl ClientboundRemoveMobEffectPacket {
    /// `ClientboundRemoveMobEffectPacket::new`.
    pub fn new(entity_id: i32, effect: crate::core::holder::Holder<crate::world::effect::mob_effect::MobEffect>) -> Self {
        Self { entity_id, effect }
    }
    /// `entityId`.
    pub fn entity_id(&self) -> i32 { self.entity_id }
    /// `effect`.
    pub fn effect(&self) -> &crate::core::holder::Holder<MobEffect> { &self.effect }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundRemoveMobEffectPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundRemoveMobEffectPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let effect = crate::world::effect::mob_effect::MobEffect::stream_codec().decode(buf);
                ClientboundRemoveMobEffectPacket::new(entity_id, effect)            },
        )
    }
}

impl Packet for ClientboundRemoveMobEffectPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_remove_mob_effect::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        crate::world::effect::mob_effect::MobEffect::stream_codec().encode(buf, &self.effect);    }
}
