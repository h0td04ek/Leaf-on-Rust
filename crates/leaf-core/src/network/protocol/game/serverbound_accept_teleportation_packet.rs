//! Перенос `ServerboundAcceptTeleportationPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundAcceptTeleportationPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundAcceptTeleportationPacket {
    /// `id` — id телепорта.
    pub id: i32,
}

impl ServerboundAcceptTeleportationPacket {
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundAcceptTeleportationPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundAcceptTeleportationPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| ServerboundAcceptTeleportationPacket::new(buf.read_var_int()),
        )
    }
}

impl Packet for ServerboundAcceptTeleportationPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_accept_teleportation::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
    }
}
