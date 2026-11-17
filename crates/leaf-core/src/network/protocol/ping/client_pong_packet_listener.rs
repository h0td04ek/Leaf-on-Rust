//! Перенос `net.minecraft.network.protocol.ping.ClientPongPacketListener`.

use crate::network::protocol::packet::{ClientboundPacketListener, PacketListener};
use super::clientbound_pong_response_packet::ClientboundPongResponsePacket;

/// `ClientPongPacketListener`.
pub trait ClientPongPacketListener: ClientboundPacketListener {
    /// `handlePongResponse`.
    fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket);
}

