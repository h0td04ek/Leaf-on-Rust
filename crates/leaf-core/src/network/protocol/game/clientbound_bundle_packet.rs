//! Перенос `ClientboundBundlePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundBundlePacket` — bundle из под-пакетов.
///
/// Тело на проводе пустое: содержимое разворачивается конвейером
/// (`BundlerInfo`/`CreateForPacketBundlerInfo`).
#[derive(Default)]
pub struct ClientboundBundlePacket {
    /// `packets` — под-пакеты (`Iterable<Packet<? super ClientGamePacketListener>>`).
    pub packets: Vec<Box<dyn Packet>>,
}

impl ClientboundBundlePacket {
    /// `ClientboundBundlePacket::new`.
    pub fn new(packets: Vec<Box<dyn Packet>>) -> Self {
        Self { packets }
    }

    /// `subPackets()` — ссылка на под-пакеты.
    pub fn sub_packets(&self) -> &[Box<dyn Packet>] {
        &self.packets
    }

    /// `STREAM_CODEC` — тело пустое.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundBundlePacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ClientboundBundlePacket| {},
            |_buf: &mut FriendlyByteBuf| ClientboundBundlePacket::new(Vec::new()),
        )
    }
}

impl Packet for ClientboundBundlePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_bundle_sub_packets(self: Box<Self>) -> Option<Vec<Box<dyn Packet>>> {
        Some((*self).packets)
    }

    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_bundle::<Self>().erase()
    }

    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}
}
