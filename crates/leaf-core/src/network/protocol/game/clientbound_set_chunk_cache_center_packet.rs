//! Перенос `ClientboundSetChunkCacheCenterPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetChunkCacheCenterPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundSetChunkCacheCenterPacket {
    /// `x` — chunk x центра.
    pub x: i32,
    /// `z` — chunk z центра.
    pub z: i32,
}

impl ClientboundSetChunkCacheCenterPacket {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// `getX`.
    pub fn get_x(&self) -> i32 {
        self.x
    }

    /// `getZ`.
    pub fn get_z(&self) -> i32 {
        self.z
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetChunkCacheCenterPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetChunkCacheCenterPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let x = buf.read_var_int();
                let z = buf.read_var_int();
                ClientboundSetChunkCacheCenterPacket::new(x, z)
            },
        )
    }
}

impl Packet for ClientboundSetChunkCacheCenterPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_chunk_cache_center::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.x);
        buf.write_var_int(self.z);
    }
}
