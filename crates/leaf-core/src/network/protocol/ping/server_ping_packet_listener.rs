//! Перенос `net.minecraft.network.protocol.ping.ServerPingPacketListener`.

use crate::network::protocol::packet::ServerboundPacketListener;
use super::serverbound_ping_request_packet::ServerboundPingRequestPacket;

/// `ServerPingPacketListener`.
pub trait ServerPingPacketListener: ServerboundPacketListener {
    /// `handlePingRequest`.
    fn handle_ping_request(&mut self, packet: &ServerboundPingRequestPacket);
}

