//! Перенос `ClientboundBundleDelimiterPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundBundleDelimiterPacket` — маркер границы bundle.
/// `handle` бросает AssertionError — обрабатывается pipeline'ом
/// (в порте: `handle` не переопределяется, как и в трейтах).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClientboundBundleDelimiterPacket;

impl ClientboundBundleDelimiterPacket {
    /// `STREAM_CODEC` — тело пустое.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundBundleDelimiterPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ClientboundBundleDelimiterPacket| {},
            |_buf: &mut FriendlyByteBuf| ClientboundBundleDelimiterPacket,
        )
    }
}

impl Packet for ClientboundBundleDelimiterPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_bundle_delimiter::<Self>().erase()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}
}
