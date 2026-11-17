//! Перенос `ClientboundUpdateMobEffectPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::effect::mob_effect::MobEffect;

/// `ClientboundUpdateMobEffectPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundUpdateMobEffectPacket {
    /// `entityId`.
    pub entity_id: i32,
    /// `effect`.
    pub effect: crate::core::holder::Holder<crate::world::effect::mob_effect::MobEffect>,
    /// `effectAmplifier`.
    pub effect_amplifier: i32,
    /// `effectDurationTicks`.
    pub effect_duration_ticks: i32,
    /// `flags`.
    pub flags: i8,
}

impl ClientboundUpdateMobEffectPacket {
    /// `ClientboundUpdateMobEffectPacket::new`.
    pub fn new(entity_id: i32, effect: crate::core::holder::Holder<crate::world::effect::mob_effect::MobEffect>, effect_amplifier: i32, effect_duration_ticks: i32, flags: i8) -> Self {
        Self { entity_id, effect, effect_amplifier, effect_duration_ticks, flags }
    }
    /// `getEntityId`.
    pub fn get_entity_id(&self) -> i32 { self.entity_id }
    /// `getEffect`.
    pub fn get_effect(&self) -> &crate::core::holder::Holder<MobEffect> { &self.effect }
    /// `getEffectAmplifier`.
    pub fn get_effect_amplifier(&self) -> i32 { self.effect_amplifier }
    /// `getEffectDurationTicks`.
    pub fn get_effect_duration_ticks(&self) -> i32 { self.effect_duration_ticks }
    /// `isEffectAmbient` — flag 1.
    pub fn is_effect_ambient(&self) -> bool { self.flags & 1 != 0 }
    /// `isEffectVisible` — flag 2.
    pub fn is_effect_visible(&self) -> bool { self.flags & 2 != 0 }
    /// `effectShowsIcon` — flag 4.
    pub fn effect_shows_icon(&self) -> bool { self.flags & 4 != 0 }
    /// `shouldBlend` — flag 8.
    pub fn should_blend(&self) -> bool { self.flags & 8 != 0 }
    /// Флаги из `MobEffectInstance` + `blend`.
    pub fn flags_from(ambient: bool, visible: bool, show_icon: bool, blend: bool) -> i8 {
        (if ambient { 1 } else { 0 })
            | (if visible { 2 } else { 0 })
            | (if show_icon { 4 } else { 0 })
            | (if blend { 8 } else { 0 })
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundUpdateMobEffectPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundUpdateMobEffectPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let effect = crate::world::effect::mob_effect::MobEffect::stream_codec().decode(buf);
                let effect_amplifier = buf.read_var_int();
                let effect_duration_ticks = buf.read_var_int();
                let flags = buf.read_byte();
                ClientboundUpdateMobEffectPacket::new(entity_id, effect, effect_amplifier, effect_duration_ticks, flags)            },
        )
    }
}

impl Packet for ClientboundUpdateMobEffectPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_update_mob_effect::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        crate::world::effect::mob_effect::MobEffect::stream_codec().encode(buf, &self.effect);
        buf.write_var_int(self.effect_amplifier);
        buf.write_var_int(self.effect_duration_ticks);
        buf.write_byte(self.flags as i32);    }
}
