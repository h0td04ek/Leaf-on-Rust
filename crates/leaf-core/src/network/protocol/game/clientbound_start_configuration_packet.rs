//! Перенос `ClientboundStartConfigurationPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{Packet, PacketTypePtr};

/// `ClientboundStartConfigurationPacket` — синглтон `INSTANCE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundStartConfigurationPacket;

/// `INSTANCE`.
pub static INSTANCE: ClientboundStartConfigurationPacket = ClientboundStartConfigurationPacket;

impl ClientboundStartConfigurationPacket {
    /// `INSTANCE`-конструктор.
    pub fn instance() -> Self {
        INSTANCE
    }

    /// `STREAM_CODEC` — `StreamCodec.unit`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundStartConfigurationPacket> {
        crate::network::codec::stream_codec::FnCodec::new(
            |_buf: &mut FriendlyByteBuf, _p: &ClientboundStartConfigurationPacket| {},
            |_buf: &mut FriendlyByteBuf| INSTANCE,
        )
    }
}

impl Packet for ClientboundStartConfigurationPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_start_configuration::<Self>().erase()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}

    /// `isTerminal`.
    fn is_terminal(&self) -> bool {
        true
    }
}
