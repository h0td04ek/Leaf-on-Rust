//! `ClientboundLevelChunkWithLightPacket` — пакет с данными чанка и освещением.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::clientbound_level_chunk_packet_data::ClientboundLevelChunkPacketData;
use crate::network::protocol::game::clientbound_light_update_packet_data::ClientboundLightUpdatePacketData;
use crate::network::protocol::packet::{packet_codec, Packet, PacketListenerObj, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;

/// `ClientboundLevelChunkWithLightPacket`.
pub struct ClientboundLevelChunkWithLightPacket {
    /// `x` — X координата чанка.
    pub x: i32,
    /// `z` — Z координата чанка.
    pub z: i32,
    /// `chunkData` — данные чанка.
    pub chunk_data: ClientboundLevelChunkPacketData,
    /// `lightData` — данные освещения.
    pub light_data: ClientboundLightUpdatePacketData,
    /// `ready` — флаг готовности (Paper).
    pub ready: bool,
}

impl ClientboundLevelChunkWithLightPacket {
    /// `new(x, z, chunkData, lightData)`.
    pub fn new(x: i32, z: i32, chunk_data: ClientboundLevelChunkPacketData, light_data: ClientboundLightUpdatePacketData) -> Self {
        Self { x, z, chunk_data, light_data, ready: true }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundLevelChunkWithLightPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundLevelChunkWithLightPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let x = buf.read_int();
                let z = buf.read_int();
                let chunk_data = ClientboundLevelChunkPacketData::read(buf, x, z);
                let light_data = ClientboundLightUpdatePacketData::read(buf, x, z);
                Self::new(x, z, chunk_data, light_data)
            },
        )
    }
}

impl Packet for ClientboundLevelChunkWithLightPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_level_chunk_with_light::<Self>().erase()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int(self.x);
        buf.write_int(self.z);
        self.chunk_data.write(buf);
        self.light_data.write(buf);
    }

    fn handle(&self, _handler: &mut dyn PacketListenerObj) {}
}
