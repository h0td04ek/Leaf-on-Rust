//! Перенос `ClientboundResetChatPacket` — singleton.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static RESET_CHAT_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("reset_chat").unwrap())
});

/// `ClientboundResetChatPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundResetChatPacket;

/// `INSTANCE`.
pub const CLIENTBOUND_RESET_CHAT_PACKET: ClientboundResetChatPacket = ClientboundResetChatPacket;

impl ClientboundResetChatPacket {
    /// `INSTANCE`.
    pub fn instance() -> Self {
        Self
    }

    /// `STREAM_CODEC` (`StreamCodec.unit`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundResetChatPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ClientboundResetChatPacket| {},
            |_buf: &mut FriendlyByteBuf| ClientboundResetChatPacket,
        )
    }
}

impl Packet for ClientboundResetChatPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        RESET_CHAT_TYPE.clone()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}
}
