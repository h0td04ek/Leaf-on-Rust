//! Перенос `net.minecraft.network.protocol.cookie.ServerCookiePacketListener`.

use crate::network::protocol::packet::ServerboundPacketListener;
use super::serverbound_cookie_response_packet::ServerboundCookieResponsePacket;

/// `ServerCookiePacketListener`.
pub trait ServerCookiePacketListener: ServerboundPacketListener {
    /// `handleCookieResponse`.
    fn handle_cookie_response(&mut self, packet: &ServerboundCookieResponsePacket);
}

