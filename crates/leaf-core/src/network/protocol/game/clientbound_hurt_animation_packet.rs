//! Перенос `ClientboundHurtAnimationPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundHurtAnimationPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundHurtAnimationPacket {
    /// `id` — entity id.
    pub id: i32,
    /// `yaw` — направление урона.
    pub yaw: f32,
}

impl ClientboundHurtAnimationPacket {
    /// `new ClientboundHurtAnimationPacket(id, yaw)`.
    pub fn new(id: i32, yaw: f32) -> Self {
        Self { id, yaw }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundHurtAnimationPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundHurtAnimationPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundHurtAnimationPacket::new(buf.read_var_int(), buf.read_float())
            },
        )
    }
}

impl Packet for ClientboundHurtAnimationPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_hurt_animation::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_float(self.yaw);
    }
}
