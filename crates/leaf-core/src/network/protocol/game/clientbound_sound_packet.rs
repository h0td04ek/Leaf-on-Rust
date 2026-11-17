//! Перенос `ClientboundSoundPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::sounds::sound_event::SoundEvent;
use crate::sounds::sound_source::SoundSource;

/// `LOCATION_ACCURACY`.
pub const LOCATION_ACCURACY: f64 = 8.0;

/// `ClientboundSoundPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSoundPacket {
    /// `sound`.
    pub sound: crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>,
    /// `source`.
    pub source: crate::sounds::sound_source::SoundSource,
    /// `x`.
    pub x: i32,
    /// `y`.
    pub y: i32,
    /// `z`.
    pub z: i32,
    /// `volume`.
    pub volume: f32,
    /// `pitch`.
    pub pitch: f32,
    /// `seed`.
    pub seed: i64,
}

impl ClientboundSoundPacket {
    /// `ClientboundSoundPacket::new`.
    pub fn new(sound: crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>, source: SoundSource, x: i32, y: i32, z: i32, volume: f32, pitch: f32, seed: i64) -> Self {
        Self { sound, source, x, y, z, volume, pitch, seed }
    }
    /// `getSound`.
    pub fn get_sound(&self) -> &crate::core::holder::Holder<SoundEvent> { &self.sound }
    /// `getSource`.
    pub fn get_source(&self) -> SoundSource { self.source }
    /// `getX` — fixed-point /8.
    pub fn get_x(&self) -> f64 { self.x as f64 / LOCATION_ACCURACY }
    /// `getY`.
    pub fn get_y(&self) -> f64 { self.y as f64 / LOCATION_ACCURACY }
    /// `getZ`.
    pub fn get_z(&self) -> f64 { self.z as f64 / LOCATION_ACCURACY }
    /// `getVolume`.
    pub fn get_volume(&self) -> f32 { self.volume }
    /// `getPitch`.
    pub fn get_pitch(&self) -> f32 { self.pitch }
    /// `getSeed`.
    pub fn get_seed(&self) -> i64 { self.seed }
    /// Координаты для конструктора из double (×8, как в Java).
    pub fn pack_position(x: f64, y: f64, z: f64) -> (i32, i32, i32) {
        ((x * LOCATION_ACCURACY) as i32, (y * LOCATION_ACCURACY) as i32, (z * LOCATION_ACCURACY) as i32)
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSoundPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSoundPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let sound = crate::sounds::sound_event::SoundEvent::stream_codec().decode(buf);
                let source = crate::sounds::sound_source::SoundSource::by_id(buf.read_var_int());
                let x = buf.read_int();
                let y = buf.read_int();
                let z = buf.read_int();
                let volume = buf.read_float();
                let pitch = buf.read_float();
                let seed = buf.read_long();
                ClientboundSoundPacket::new(sound, source, x, y, z, volume, pitch, seed)            },
        )
    }
}

impl Packet for ClientboundSoundPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_sound::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        crate::sounds::sound_event::SoundEvent::stream_codec().encode(buf, &self.sound);
        buf.write_var_int(self.source.ordinal());
        buf.write_int(self.x);
        buf.write_int(self.y);
        buf.write_int(self.z);
        buf.write_float(self.volume);
        buf.write_float(self.pitch);
        buf.write_long(self.seed);    }
}
