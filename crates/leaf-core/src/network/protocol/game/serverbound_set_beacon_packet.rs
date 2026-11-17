//! Перенос `ServerboundSetBeaconPacket`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::effect::mob_effect::MobEffect;

/// `ServerboundSetBeaconPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundSetBeaconPacket {
    /// `primary`.
    pub primary: Option<Holder<MobEffect>>,
    /// `secondary`.
    pub secondary: Option<Holder<MobEffect>>,
}

impl ServerboundSetBeaconPacket {
    /// `new`.
    pub fn new(primary: Option<Holder<MobEffect>>, secondary: Option<Holder<MobEffect>>) -> Self {
        Self { primary, secondary }
    }

    /// `primary`/`secondary`.
    pub fn primary(&self) -> Option<&Holder<MobEffect>> {
        self.primary.as_ref()
    }
    /// `secondary`.
    pub fn secondary(&self) -> Option<&Holder<MobEffect>> {
        self.secondary.as_ref()
    }

    fn read_optional_effect(buf: &mut FriendlyByteBuf) -> Option<Holder<MobEffect>> {
        buf.read_optional(|b| MobEffect::stream_codec().decode(b))
    }

    fn write_optional_effect(buf: &mut FriendlyByteBuf, value: &Option<Holder<MobEffect>>) {
        buf.write_optional(value, |b, v| {
            MobEffect::stream_codec().encode(b, v);
        });
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSetBeaconPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSetBeaconPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let primary = Self::read_optional_effect(buf);
                let secondary = Self::read_optional_effect(buf);
                ServerboundSetBeaconPacket::new(primary, secondary)
            },
        )
    }
}

impl Packet for ServerboundSetBeaconPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_set_beacon::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        Self::write_optional_effect(buf, &self.primary);
        Self::write_optional_effect(buf, &self.secondary);
    }
}
