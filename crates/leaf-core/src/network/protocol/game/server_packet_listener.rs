//! Перенос `ServerPacketListener` — базовый интерфейс серверного
//! game-listener'а (`onPacketError` default-логирование).

use crate::network::protocol::packet::ServerboundPacketListener;

/// `ServerPacketListener`.
pub trait ServerPacketListener: ServerboundPacketListener {
    /// `onPacketError` — в Java логирует и подавляет ошибку.
    /// Здесь — no-op (логирование добавится с портом `crash_report`/log4j).
    fn on_packet_error(&mut self, _packet: &dyn crate::network::protocol::packet::Packet, _exception: &str) {}
}
