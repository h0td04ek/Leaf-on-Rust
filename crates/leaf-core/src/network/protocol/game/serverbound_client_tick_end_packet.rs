//! Перенос `ServerboundClientTickEndPacket` — singleton.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundClientTickEndPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundClientTickEndPacket;

/// `INSTANCE`.
pub const SERVERBOUND_CLIENT_TICK_END_PACKET: ServerboundClientTickEndPacket =
    ServerboundClientTickEndPacket;

impl ServerboundClientTickEndPacket {
    /// `INSTANCE`.
    pub fn instance() -> Self {
        Self
    }

    /// `STREAM_CODEC` (`StreamCodec.unit`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundClientTickEndPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ServerboundClientTickEndPacket| {},
            |_buf: &mut FriendlyByteBuf| ServerboundClientTickEndPacket,
        )
    }
}

impl Packet for ServerboundClientTickEndPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_client_tick_end::<Self>().erase()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}
}
