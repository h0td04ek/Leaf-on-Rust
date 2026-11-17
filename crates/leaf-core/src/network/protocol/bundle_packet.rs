//! Перенос `net.minecraft.network.protocol.BundlePacket`.
//!
//! Реализация — через `Packet::into_bundle_sub_packets`
//! (см. `packet.rs`); трейт-маркер для bundle-пакетов.

use crate::network::protocol::packet::Packet;

/// `BundlePacket<T>` — пакет, содержащий под-пакеты.
/// Конкретные bundle-пакеты реализуют `Packet` и возвращают
/// `Some(sub_packets)` из `into_bundle_sub_packets`.
pub trait BundlePacket: Packet {
    /// `subPackets()` — итерация под-пакетов (для &self-доступа
    /// используется `Packet::bundle_sub_packets_ref` при необходимости).
    fn bundle_sub_packets_vec(&self) -> Vec<Box<dyn Packet>>;
}

/// `BundleDelimiterPacket<T>` — маркер границы bundle.
/// `handle` бросает AssertionError — обрабатывается pipeline'ом.
pub trait BundleDelimiterPacket: Packet {}
