//! Перенос `net.minecraft.network.ProtocolInfo` (+ `Details`,
//! `DetailsProvider`).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::Packet;
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use std::sync::Arc;

/// `StreamCodec<ByteBuf, Packet<? super T>>` — type-erased кодек протокола.
pub type PacketCodec = Arc<dyn StreamCodec<FriendlyByteBuf, Box<dyn Packet>>>;

/// `ProtocolInfo<T extends PacketListener>`.
pub trait ProtocolInfo {
    /// `id()` — вид протокола.
    fn id(&self) -> ConnectionProtocol;
    /// `flow()`.
    fn flow(&self) -> PacketFlow;
    /// `codec()` — пакетный кодек (id + тело).
    fn codec(&self) -> &PacketCodec;
    /// `bundlerInfo()` — bundle-кодек, если есть.
    fn bundler_info(&self) -> Option<&dyn crate::network::protocol::bundler_info::BundlerInfo>;
    /// `details()` — метаданные для отладки/валидации.
    fn details(&self) -> &dyn ProtocolInfoDetails;
}

/// `ProtocolInfo.Details`.
pub trait ProtocolInfoDetails {
    /// `id()`.
    fn id(&self) -> ConnectionProtocol;
    /// `flow()`.
    fn flow(&self) -> PacketFlow;
    /// `listPackets(visitor)` — (packetType, index).
    fn list_packets(&self, visitor: &mut dyn FnMut(PacketType<Box<dyn Packet>>, usize));
}

/// `ProtocolInfo.DetailsProvider`.
pub trait ProtocolInfoDetailsProvider {
    /// `details()`.
    fn details(&self) -> SharedProtocolInfoDetails;
}

/// Разделяемый `ProtocolInfo.Details` (аналог ссылки на Details).
pub type SharedProtocolInfoDetails = Arc<dyn ProtocolInfoDetails>;
