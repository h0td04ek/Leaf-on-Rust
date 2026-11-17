//! `net.minecraft.core.particles` — типы и опции частиц.

pub mod particle_type;
pub mod particle_options;
pub mod particle_types;
pub mod explosion_particle_info;

pub use particle_type::{ParticleOptionsKind, ParticleType};
pub use particle_options::{ParticleOptions, ParticleOptionsData};
pub use particle_types::{
    particle_type_by_id, particle_type_by_identifier, particle_type_count, particle_type_stream_codec,
    particle_types,
};
pub use explosion_particle_info::ExplosionParticleInfo;
