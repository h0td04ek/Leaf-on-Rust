//! Перенос `net.minecraft.network.protocol.status.ClientStatusPacketListener`.

use crate::network::protocol::packet::{ClientboundPacketListener, PacketListener};
use crate::network::protocol::ping::client_pong_packet_listener::ClientPongPacketListener;
use super::clientbound_status_response_packet::ClientboundStatusResponsePacket;

/// `ClientStatusPacketListener`.
pub trait ClientStatusPacketListener: ClientPongPacketListener + ClientboundPacketListener {
    /// `handleStatusResponse`.
    fn handle_status_response(&mut self, packet: &ClientboundStatusResponsePacket);
}
