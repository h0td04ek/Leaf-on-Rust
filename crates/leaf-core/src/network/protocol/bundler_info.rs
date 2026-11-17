//! Перенос `net.minecraft.network.protocol.BundlerInfo` (+ `Bundler`).

use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::packet_type::PacketType;
use std::sync::Arc;

/// `BundlerInfo.BUNDLE_SIZE_LIMIT`.
pub const BUNDLE_SIZE_LIMIT: usize = 4096;

/// `BundlerInfo.Bundler` — накопитель bundle на приёме.
pub trait PacketBundler {
    /// `addPacket(subPacket)` → Some(готовый bundle) или None.
    fn add_packet(&mut self, sub_packet: Box<dyn Packet>) -> Option<Box<dyn Packet>>;
}

/// `BundlerInfo` — описание bundle-протокола направления.
pub trait BundlerInfo {
    /// `unbundlePacket(packet, consumer)` — раскрыть bundle в поток
    /// (delimiter + sub-packets + delimiter), обычный пакет → как есть.
    fn unbundle_packet(&self, packet: Box<dyn Packet>, out: &mut Vec<Box<dyn Packet>>);

    /// `startPacketBundling(packet)` → Some(bundler) если это delimiter.
    /// В Java сравнение по ссылке; здесь — по `packet_type`
    /// (эквивалентно: delimiter — синглтон конкретного типа).
    fn start_packet_bundling(
        &self,
        packet_type: PacketTypePtr,
    ) -> Option<Box<dyn PacketBundler>>;
}

/// Реализация `BundlerInfo.createForPacket`.
pub struct CreateForPacketBundlerInfo {
    bundle_type: PacketTypePtr,
    delimiter_type: PacketTypePtr,
    bundler: Arc<dyn Fn(Vec<Box<dyn Packet>>) -> Box<dyn Packet> + Send + Sync>,
    delimiter_factory: Arc<dyn Fn() -> Box<dyn Packet> + Send + Sync>,
}

impl CreateForPacketBundlerInfo {
    /// `BundlerInfo.createForPacket(type, bundler, delimiterPacket)`.
    /// `delimiter_factory` создаёт экземпляр delimiter-пакета
    /// (в Java переиспользуется один объект — у нас пакеты boxed,
    /// поэтому тип и экземпляры задаются фабрикой).
    pub fn new_with_delimiter_type(
        bundle_type: PacketTypePtr,
        delimiter_type: PacketTypePtr,
        bundler: impl Fn(Vec<Box<dyn Packet>>) -> Box<dyn Packet> + Send + Sync + 'static,
        delimiter_factory: impl Fn() -> Box<dyn Packet> + Send + Sync + 'static,
    ) -> Self {
        Self {
            bundle_type,
            delimiter_type,
            bundler: Arc::new(bundler),
            delimiter_factory: Arc::new(delimiter_factory),
        }
    }
}

/// Bundler-аккумулятор для `createForPacket`.
struct CreateForPacketBundler {
    bundle_packets: Vec<Box<dyn Packet>>,
    bundler: Arc<dyn Fn(Vec<Box<dyn Packet>>) -> Box<dyn Packet> + Send + Sync>,
    delimiter_type: PacketTypePtr,
}

impl PacketBundler for CreateForPacketBundler {
    fn add_packet(&mut self, sub_packet: Box<dyn Packet>) -> Option<Box<dyn Packet>> {
        if sub_packet.packet_type() == self.delimiter_type {
            Some((self.bundler)(std::mem::take(&mut self.bundle_packets)))
        } else {
            if self.bundle_packets.len() >= BUNDLE_SIZE_LIMIT {
                panic!("Too many packets in a bundle");
            }
            self.bundle_packets.push(sub_packet);
            None
        }
    }
}

impl BundlerInfo for CreateForPacketBundlerInfo {
    fn unbundle_packet(&self, packet: Box<dyn Packet>, out: &mut Vec<Box<dyn Packet>>) {
        if packet.packet_type() == self.bundle_type {
            out.push((self.delimiter_factory)());
            if let Some(subs) = packet.into_bundle_sub_packets() {
                for sub in subs {
                    out.push(sub);
                }
            }
            out.push((self.delimiter_factory)());
        } else {
            out.push(packet);
        }
    }

    fn start_packet_bundling(
        &self,
        packet_type: PacketTypePtr,
    ) -> Option<Box<dyn PacketBundler>> {
        if packet_type == self.delimiter_type {
            Some(Box::new(CreateForPacketBundler {
                bundle_packets: Vec::new(),
                bundler: self.bundler.clone(),
                delimiter_type: self.delimiter_type.clone(),
            }))
        } else {
            None
        }
    }
}
