//! Перенос `net.minecraft.network.protocol.handshake.ServerHandshakePacketListener`.

use crate::network::protocol::packet::ServerboundPacketListener;
use super::client_intention_packet::ClientIntentionPacket;

/// `ServerHandshakePacketListener`.
pub trait ServerHandshakePacketListener: ServerboundPacketListener {
    /// `handleIntention`.
    fn handle_intention(&mut self, packet: &ClientIntentionPacket);
}

