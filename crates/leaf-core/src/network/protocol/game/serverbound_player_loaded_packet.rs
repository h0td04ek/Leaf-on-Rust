//! Перенос `ServerboundPlayerLoadedPacket` — singleton.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundPlayerLoadedPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPlayerLoadedPacket;

/// `INSTANCE`.
pub const SERVERBOUND_PLAYER_LOADED_PACKET: ServerboundPlayerLoadedPacket =
    ServerboundPlayerLoadedPacket;

impl ServerboundPlayerLoadedPacket {
    /// `INSTANCE`.
    pub fn instance() -> Self {
        Self
    }

    /// `STREAM_CODEC` (`StreamCodec.unit`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundPlayerLoadedPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ServerboundPlayerLoadedPacket| {},
            |_buf: &mut FriendlyByteBuf| ServerboundPlayerLoadedPacket,
        )
    }
}

impl Packet for ServerboundPlayerLoadedPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_player_loaded::<Self>().erase()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}
}
