//! Перенос `ServerboundTeleportToEntityPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::util::uuid::Uuid;

/// `ServerboundTeleportToEntityPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundTeleportToEntityPacket {
    /// `uuid` — entity uuid.
    pub uuid: Uuid,
}

impl ServerboundTeleportToEntityPacket {
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }

    /// `STREAM_CODEC` (`readUUID`/`writeUUID` = 16 байт).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundTeleportToEntityPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundTeleportToEntityPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundTeleportToEntityPacket::new(buf.read_uuid())
            },
        )
    }
}

impl Packet for ServerboundTeleportToEntityPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_teleport_to_entity::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_uuid(&self.uuid);
    }
}
