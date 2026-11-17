//! Перенос `ClientboundRespawnPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::common_player_spawn_info::CommonPlayerSpawnInfo;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `KEEP_ATTRIBUTE_MODIFIERS`.
pub const KEEP_ATTRIBUTE_MODIFIERS: i8 = 1;
/// `KEEP_ENTITY_DATA`.
pub const KEEP_ENTITY_DATA: i8 = 2;
/// `KEEP_ALL_DATA`.
pub const KEEP_ALL_DATA: i8 = 3;

/// `ClientboundRespawnPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundRespawnPacket {
    /// `commonPlayerSpawnInfo`.
    pub common_player_spawn_info: CommonPlayerSpawnInfo,
    /// `dataToKeep` (битовая маска KEEP_*).
    pub data_to_keep: i8,
}

impl ClientboundRespawnPacket {
    /// `ClientboundRespawnPacket::new`.
    pub fn new(common_player_spawn_info: CommonPlayerSpawnInfo, data_to_keep: i8) -> Self {
        Self { common_player_spawn_info, data_to_keep }
    }

    /// `shouldKeep`.
    pub fn should_keep(&self, data: i8) -> bool {
        (self.data_to_keep & data) != 0
    }

    /// `STREAM_CODEC` — `Packet.codec` (ручной read/write).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundRespawnPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundRespawnPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let info = CommonPlayerSpawnInfo::read(buf);
                let data_to_keep = buf.read_byte();

                ClientboundRespawnPacket::new(info, data_to_keep)
            },
        )
    }
}

impl Packet for ClientboundRespawnPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_respawn::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        self.common_player_spawn_info.write(buf);
        buf.write_byte(self.data_to_keep as i32);
    }
}
