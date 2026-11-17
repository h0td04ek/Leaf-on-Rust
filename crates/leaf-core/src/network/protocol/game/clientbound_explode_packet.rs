//! `ClientboundExplodePacket` — взрыв.

use crate::core::holder::Holder;
use crate::core::particles::explosion_particle_info::ExplosionParticleInfo;
use crate::core::particles::particle_options::ParticleOptions;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::sounds::sound_event::SoundEvent;
use crate::util::random::weighted_list::WeightedList;
use crate::world::phys::vec3::Vec3;

/// `ClientboundExplodePacket`.
#[derive(Clone, Debug)]
pub struct ClientboundExplodePacket {
    /// `center`.
    pub center: Vec3,
    /// `radius`.
    pub radius: f32,
    /// `blockCount`.
    pub block_count: i32,
    /// `playerKnockback`.
    pub player_knockback: Option<Vec3>,
    /// `explosionParticle`.
    pub explosion_particle: ParticleOptions,
    /// `explosionSound`.
    pub explosion_sound: Holder<SoundEvent>,
    /// `blockParticles`.
    pub block_particles: WeightedList<ExplosionParticleInfo>,
}

impl ClientboundExplodePacket {
    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        center: Vec3,
        radius: f32,
        block_count: i32,
        player_knockback: Option<Vec3>,
        explosion_particle: ParticleOptions,
        explosion_sound: Holder<SoundEvent>,
        block_particles: WeightedList<ExplosionParticleInfo>,
    ) -> Self {
        Self {
            center, radius, block_count, player_knockback,
            explosion_particle, explosion_sound, block_particles,
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundExplodePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundExplodePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let center = Vec3::stream_codec().decode(buf);
                let radius = buf.read_float();
                let block_count = buf.read_int();
                let player_knockback = if buf.read_bool() {
                    Some(Vec3::stream_codec().decode(buf))
                } else { None };
                let explosion_particle = ParticleOptions::stream_codec().decode(buf);
                let explosion_sound = SoundEvent::stream_codec().decode(buf);
                let block_particles = WeightedList::<ExplosionParticleInfo>::stream_codec(ExplosionParticleInfo::stream_codec()).decode(buf);
                ClientboundExplodePacket::new(
                    center, radius, block_count, player_knockback,
                    explosion_particle, explosion_sound, block_particles,
                )
            },
        )
    }
}

impl Packet for ClientboundExplodePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_explode::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        Vec3::stream_codec().encode(buf, &self.center);
        buf.write_float(self.radius);
        buf.write_int(self.block_count);
        match &self.player_knockback {
            Some(v) => { buf.write_bool(true); Vec3::stream_codec().encode(buf, v); }
            None => { buf.write_bool(false); }
        }
        ParticleOptions::stream_codec().encode(buf, &self.explosion_particle);
        SoundEvent::stream_codec().encode(buf, &self.explosion_sound);
        WeightedList::<ExplosionParticleInfo>::stream_codec(ExplosionParticleInfo::stream_codec()).encode(buf, &self.block_particles);
    }
}
