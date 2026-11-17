//! Перенос `ClientboundSoundEntityPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::sounds::sound_event::SoundEvent;
use crate::sounds::sound_source::SoundSource;

/// `ClientboundSoundEntityPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSoundEntityPacket {
    /// `sound`.
    pub sound: crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>,
    /// `source`.
    pub source: crate::sounds::sound_source::SoundSource,
    /// `id`.
    pub id: i32,
    /// `volume`.
    pub volume: f32,
    /// `pitch`.
    pub pitch: f32,
    /// `seed`.
    pub seed: i64,
}

impl ClientboundSoundEntityPacket {
    /// `ClientboundSoundEntityPacket::new`.
    pub fn new(sound: crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>, source: SoundSource, id: i32, volume: f32, pitch: f32, seed: i64) -> Self {
        Self { sound, source, id, volume, pitch, seed }
    }
    /// `getSound`.
    pub fn get_sound(&self) -> &crate::core::holder::Holder<SoundEvent> { &self.sound }
    /// `getSource`.
    pub fn get_source(&self) -> SoundSource { self.source }
    /// `getId`.
    pub fn get_id(&self) -> i32 { self.id }
    /// `getVolume`.
    pub fn get_volume(&self) -> f32 { self.volume }
    /// `getPitch`.
    pub fn get_pitch(&self) -> f32 { self.pitch }
    /// `getSeed`.
    pub fn get_seed(&self) -> i64 { self.seed }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSoundEntityPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSoundEntityPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let sound = crate::sounds::sound_event::SoundEvent::stream_codec().decode(buf);
                let source = crate::sounds::sound_source::SoundSource::by_id(buf.read_var_int());
                let id = buf.read_var_int();
                let volume = buf.read_float();
                let pitch = buf.read_float();
                let seed = buf.read_long();
                ClientboundSoundEntityPacket::new(sound, source, id, volume, pitch, seed)            },
        )
    }
}

impl Packet for ClientboundSoundEntityPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_sound_entity::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        crate::sounds::sound_event::SoundEvent::stream_codec().encode(buf, &self.sound);
        buf.write_var_int(self.source.ordinal());
        buf.write_var_int(self.id);
        buf.write_float(self.volume);
        buf.write_float(self.pitch);
        buf.write_long(self.seed);    }
}
