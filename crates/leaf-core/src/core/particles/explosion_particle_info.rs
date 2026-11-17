//! Перенос `net.minecraft.core.particles.ExplosionParticleInfo` —
//! `record(ParticleOptions particle, float scaling, float speed)`.

use super::particle_options::ParticleOptions;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ExplosionParticleInfo` — частица + scaling + speed для взрыва.
#[derive(Clone, Debug, PartialEq)]
pub struct ExplosionParticleInfo {
    /// `particle`.
    pub particle: ParticleOptions,
    /// `scaling`.
    pub scaling: f32,
    /// `speed`.
    pub speed: f32,
}

impl ExplosionParticleInfo {
    /// Конструктор.
    pub fn new(particle: ParticleOptions, scaling: f32, speed: f32) -> Self {
        Self { particle, scaling, speed }
    }

    /// `STREAM_CODEC` — `ParticleTypes.STREAM_CODEC` + FLOAT + FLOAT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ExplosionParticleInfo> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, v: &ExplosionParticleInfo| {
                ParticleOptions::stream_codec().encode(buf, &v.particle);
                buf.write_float(v.scaling);
                buf.write_float(v.speed);
            },
            |buf: &mut FriendlyByteBuf| {
                let particle = ParticleOptions::stream_codec().decode(buf);
                let scaling = buf.read_float();
                let speed = buf.read_float();
                ExplosionParticleInfo::new(particle, scaling, speed)
            },
        )
    }
}
