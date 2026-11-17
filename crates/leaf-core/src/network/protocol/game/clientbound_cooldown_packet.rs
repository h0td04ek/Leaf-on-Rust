//! Перенос `ClientboundCooldownPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::resources::identifier::Identifier;

/// `ClientboundCooldownPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundCooldownPacket {
    /// `cooldownGroup`.
    pub cooldown_group: Identifier,
    /// `duration` — тики.
    pub duration: i32,
}

impl ClientboundCooldownPacket {
    pub fn new(cooldown_group: Identifier, duration: i32) -> Self {
        Self { cooldown_group, duration }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundCooldownPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundCooldownPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let cooldown_group = buf.read_identifier().expect("invalid cooldown group");
                let duration = buf.read_var_int();
                ClientboundCooldownPacket::new(cooldown_group, duration)
            },
        )
    }
}

impl Packet for ClientboundCooldownPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_cooldown::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_identifier(&self.cooldown_group).expect("invalid identifier");
        buf.write_var_int(self.duration);
    }
}
