//! Перенос `ClientboundSetDefaultSpawnPositionPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::level::RespawnData;

/// `ClientboundSetDefaultSpawnPositionPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetDefaultSpawnPositionPacket {
    /// `respawnData`.
    pub respawn_data: RespawnData,
}

impl ClientboundSetDefaultSpawnPositionPacket {
    /// `ClientboundSetDefaultSpawnPositionPacket::new`.
    pub fn new(respawn_data: RespawnData) -> Self {
        Self { respawn_data }
    }

    /// `STREAM_CODEC` — `LevelData.RespawnData.STREAM_CODEC`.
    pub fn stream_codec()
    -> impl StreamCodec<FriendlyByteBuf, ClientboundSetDefaultSpawnPositionPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetDefaultSpawnPositionPacket| {
                p.respawn_data.write(buf)
            },
            |buf: &mut FriendlyByteBuf| {
                ClientboundSetDefaultSpawnPositionPacket::new(RespawnData::read(buf))
            },
        )
    }
}

impl Packet for ClientboundSetDefaultSpawnPositionPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_default_spawn_position::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        self.respawn_data.write(buf);
    }
}
