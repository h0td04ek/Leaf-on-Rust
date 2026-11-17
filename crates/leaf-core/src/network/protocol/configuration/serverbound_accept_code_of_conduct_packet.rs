//! Перенос `ServerboundAcceptCodeOfConductPacket` — singleton.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static ACCEPT_CODE_OF_CONDUCT_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::SERVERBOUND,
        Identifier::with_default_namespace("accept_code_of_conduct").unwrap(),
    )
});

/// `ServerboundAcceptCodeOfConductPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundAcceptCodeOfConductPacket;

/// `INSTANCE`.
pub const SERVERBOUND_ACCEPT_CODE_OF_CONDUCT_PACKET: ServerboundAcceptCodeOfConductPacket =
    ServerboundAcceptCodeOfConductPacket;

impl ServerboundAcceptCodeOfConductPacket {
    /// `INSTANCE`.
    pub fn instance() -> Self {
        Self
    }

    /// `STREAM_CODEC` (`StreamCodec.unit`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundAcceptCodeOfConductPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ServerboundAcceptCodeOfConductPacket| {},
            |_buf: &mut FriendlyByteBuf| ServerboundAcceptCodeOfConductPacket,
        )
    }
}

impl Packet for ServerboundAcceptCodeOfConductPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        ACCEPT_CODE_OF_CONDUCT_TYPE.clone()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}
}
