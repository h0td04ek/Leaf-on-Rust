//! Перенос `net.minecraft.network.protocol.status.ServerStatusPacketListener`.

use crate::network::protocol::packet::{PacketListener, ServerboundPacketListener};
use crate::network::protocol::ping::server_ping_packet_listener::ServerPingPacketListener;
use super::serverbound_status_request_packet::ServerboundStatusRequestPacket;

/// `ServerStatusPacketListener`.
/// (`ServerPacketListener` — game-пакет, расширение при порте game-протокола.)
pub trait ServerStatusPacketListener: ServerboundPacketListener + ServerPingPacketListener {
    /// `handleStatusRequest`.
    fn handle_status_request(&mut self, packet: &ServerboundStatusRequestPacket);
}
