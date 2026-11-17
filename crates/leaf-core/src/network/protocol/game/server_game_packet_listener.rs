//! Перенос `ServerGamePacketListener` (+ `ServerPingPacketListener`).

use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use crate::network::protocol::ping::server_ping_packet_listener::ServerPingPacketListener;
use super::serverbound_set_creative_mode_slot_packet::ServerboundSetCreativeModeSlotPacket;
use super::server_packet_listener::ServerPacketListener;

/// `ServerGamePacketListener`.
pub trait ServerGamePacketListener:
    ServerPacketListener + ServerCommonPacketListener + ServerPingPacketListener
{
    /// `protocol()` — `ConnectionProtocol.PLAY`.
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Play
    }

    fn handle_set_creative_mode_slot(&mut self, _packet: &ServerboundSetCreativeModeSlotPacket) {}
}
