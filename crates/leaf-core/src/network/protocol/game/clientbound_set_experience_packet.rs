//! Перенос `ClientboundSetExperiencePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetExperiencePacket`.
///
/// ВАЖНО: порядок полей на проводе — `experienceProgress`,
/// `experienceLevel`, `totalExperience` (не в порядке объявления).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundSetExperiencePacket {
    /// `experienceProgress`.
    pub experience_progress: f32,
    /// `totalExperience`.
    pub total_experience: i32,
    /// `experienceLevel`.
    pub experience_level: i32,
}

impl ClientboundSetExperiencePacket {
    /// `ClientboundSetExperiencePacket::new`.
    pub fn new(experience_progress: f32, total_experience: i32, experience_level: i32) -> Self {
        Self { experience_progress, total_experience, experience_level }
    }

    /// `getExperienceProgress`.
    pub fn get_experience_progress(&self) -> f32 {
        self.experience_progress
    }

    /// `getTotalExperience`.
    pub fn get_total_experience(&self) -> i32 {
        self.total_experience
    }

    /// `getExperienceLevel`.
    pub fn get_experience_level(&self) -> i32 {
        self.experience_level
    }

    /// `STREAM_CODEC` — `Packet.codec`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetExperiencePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetExperiencePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let experience_progress = buf.read_float();
                let experience_level = buf.read_var_int();
                let total_experience = buf.read_var_int();

                ClientboundSetExperiencePacket::new(experience_progress, total_experience, experience_level)
            },
        )
    }
}

impl Packet for ClientboundSetExperiencePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_experience::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_float(self.experience_progress);
        buf.write_var_int(self.experience_level);
        buf.write_var_int(self.total_experience);
    }
}
