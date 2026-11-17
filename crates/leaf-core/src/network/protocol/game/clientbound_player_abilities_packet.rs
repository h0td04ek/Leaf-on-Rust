//! Перенос `ClientboundPlayerAbilitiesPacket`.

use crate::entity::player::abilities::Abilities;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

const FLAG_INVULNERABLE: i32 = 1;
const FLAG_FLYING: i32 = 2;
const FLAG_CAN_FLY: i32 = 4;
const FLAG_INSTABUILD: i32 = 8;

/// `ClientboundPlayerAbilitiesPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundPlayerAbilitiesPacket {
    /// `invulnerable`.
    pub invulnerable: bool,
    /// `isFlying`.
    pub is_flying: bool,
    /// `canFly`.
    pub can_fly: bool,
    /// `instabuild`.
    pub instabuild: bool,
    /// `flyingSpeed`.
    pub flying_speed: f32,
    /// `walkingSpeed`.
    pub walking_speed: f32,
}

impl ClientboundPlayerAbilitiesPacket {
    /// `new ClientboundPlayerAbilitiesPacket(abilities)`.
    pub fn from_abilities(abilities: &Abilities) -> Self {
        Self {
            invulnerable: abilities.invulnerable,
            is_flying: abilities.flying,
            can_fly: abilities.mayfly,
            instabuild: abilities.instabuild,
            flying_speed: abilities.get_flying_speed(),
            walking_speed: abilities.get_walking_speed(),
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlayerAbilitiesPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPlayerAbilitiesPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let flags = buf.read_byte() as i32;
                let invulnerable = flags & FLAG_INVULNERABLE != 0;
                let is_flying = flags & FLAG_FLYING != 0;
                let can_fly = flags & FLAG_CAN_FLY != 0;
                let instabuild = flags & FLAG_INSTABUILD != 0;
                let flying_speed = buf.read_float();
                let walking_speed = buf.read_float();
                ClientboundPlayerAbilitiesPacket {
                    invulnerable,
                    is_flying,
                    can_fly,
                    instabuild,
                    flying_speed,
                    walking_speed,
                }
            },
        )
    }
}

impl Packet for ClientboundPlayerAbilitiesPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_player_abilities::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        let mut flags: i32 = 0;
        if self.invulnerable {
            flags |= FLAG_INVULNERABLE;
        }
        if self.is_flying {
            flags |= FLAG_FLYING;
        }
        if self.can_fly {
            flags |= FLAG_CAN_FLY;
        }
        if self.instabuild {
            flags |= FLAG_INSTABUILD;
        }
        buf.write_byte(flags);
        buf.write_float(self.flying_speed);
        buf.write_float(self.walking_speed);
    }
}
