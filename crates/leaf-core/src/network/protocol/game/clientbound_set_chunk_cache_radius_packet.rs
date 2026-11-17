//! Перенос `ClientboundSetChunkCacheRadiusPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetChunkCacheRadiusPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundSetChunkCacheRadiusPacket {
    /// `radius`.
    pub radius: i32,
}

impl ClientboundSetChunkCacheRadiusPacket {
    pub fn new(radius: i32) -> Self {
        Self { radius }
    }

    /// `getRadius`.
    pub fn get_radius(&self) -> i32 {
        self.radius
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetChunkCacheRadiusPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetChunkCacheRadiusPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundSetChunkCacheRadiusPacket::new(buf.read_var_int())
            },
        )
    }
}

impl Packet for ClientboundSetChunkCacheRadiusPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_chunk_cache_radius::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.radius);
    }
}
