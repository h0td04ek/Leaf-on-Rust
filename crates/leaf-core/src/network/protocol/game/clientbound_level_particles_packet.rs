//! `ClientboundLevelParticlesPacket` — спавн частиц.

use crate::core::particles::particle_options::ParticleOptions;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundLevelParticlesPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundLevelParticlesPacket {
    /// `x`.
    pub x: f64,
    /// `y`.
    pub y: f64,
    /// `z`.
    pub z: f64,
    /// `xDist`.
    pub x_dist: f32,
    /// `yDist`.
    pub y_dist: f32,
    /// `zDist`.
    pub z_dist: f32,
    /// `maxSpeed`.
    pub max_speed: f32,
    /// `count`.
    pub count: i32,
    /// `overrideLimiter`.
    pub override_limiter: bool,
    /// `alwaysShow`.
    pub always_show: bool,
    /// `particle`.
    pub particle: ParticleOptions,
}

impl ClientboundLevelParticlesPacket {
    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        particle: ParticleOptions,
        override_limiter: bool,
        always_show: bool,
        x: f64,
        y: f64,
        z: f64,
        x_dist: f32,
        y_dist: f32,
        z_dist: f32,
        max_speed: f32,
        count: i32,
    ) -> Self {
        Self {
            x, y, z, x_dist, y_dist, z_dist, max_speed, count,
            override_limiter, always_show, particle,
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundLevelParticlesPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundLevelParticlesPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let override_limiter = buf.read_bool();
                let always_show = buf.read_bool();
                let x = buf.read_double();
                let y = buf.read_double();
                let z = buf.read_double();
                let x_dist = buf.read_float();
                let y_dist = buf.read_float();
                let z_dist = buf.read_float();
                let max_speed = buf.read_float();
                let count = buf.read_int();
                let particle = ParticleOptions::stream_codec().decode(buf);
                ClientboundLevelParticlesPacket::new(
                    particle, override_limiter, always_show, x, y, z,
                    x_dist, y_dist, z_dist, max_speed, count,
                )
            },
        )
    }
}

impl Packet for ClientboundLevelParticlesPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_level_particles::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_bool(self.override_limiter);
        buf.write_bool(self.always_show);
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_float(self.x_dist);
        buf.write_float(self.y_dist);
        buf.write_float(self.z_dist);
        buf.write_float(self.max_speed);
        buf.write_int(self.count);
        ParticleOptions::stream_codec().encode(buf, &self.particle);
    }
}
