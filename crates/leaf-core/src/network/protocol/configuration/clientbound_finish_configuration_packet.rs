//! Перенос `ClientboundFinishConfigurationPacket` — singleton.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static FINISH_CONFIGURATION_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::CLIENTBOUND,
        Identifier::with_default_namespace("finish_configuration").unwrap(),
    )
});

/// `ClientboundFinishConfigurationPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundFinishConfigurationPacket;

/// `INSTANCE`.
pub const CLIENTBOUND_FINISH_CONFIGURATION_PACKET: ClientboundFinishConfigurationPacket =
    ClientboundFinishConfigurationPacket;

impl ClientboundFinishConfigurationPacket {
    /// `INSTANCE`.
    pub fn instance() -> Self {
        Self
    }

    /// `STREAM_CODEC` (`StreamCodec.unit`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundFinishConfigurationPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ClientboundFinishConfigurationPacket| {},
            |_buf: &mut FriendlyByteBuf| ClientboundFinishConfigurationPacket,
        )
    }
}

impl Packet for ClientboundFinishConfigurationPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        FINISH_CONFIGURATION_TYPE.clone()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}

    fn is_terminal(&self) -> bool {
        true
    }
}
