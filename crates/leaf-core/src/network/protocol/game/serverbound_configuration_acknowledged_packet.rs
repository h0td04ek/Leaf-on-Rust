//! Перенос `ServerboundConfigurationAcknowledgedPacket` — singleton, terminal.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundConfigurationAcknowledgedPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundConfigurationAcknowledgedPacket;

/// `INSTANCE`.
pub const SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET: ServerboundConfigurationAcknowledgedPacket =
    ServerboundConfigurationAcknowledgedPacket;

impl ServerboundConfigurationAcknowledgedPacket {
    /// `INSTANCE`.
    pub fn instance() -> Self {
        Self
    }

    /// `STREAM_CODEC` (`StreamCodec.unit`).
    pub fn stream_codec()
    -> impl StreamCodec<FriendlyByteBuf, ServerboundConfigurationAcknowledgedPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ServerboundConfigurationAcknowledgedPacket| {},
            |_buf: &mut FriendlyByteBuf| ServerboundConfigurationAcknowledgedPacket,
        )
    }
}

impl Packet for ServerboundConfigurationAcknowledgedPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_configuration_acknowledged::<Self>().erase()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}

    fn is_terminal(&self) -> bool {
        true
    }
}
