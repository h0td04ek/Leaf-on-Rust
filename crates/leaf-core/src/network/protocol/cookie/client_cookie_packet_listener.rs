//! Перенос `net.minecraft.network.protocol.cookie.ClientCookiePacketListener`.

use crate::network::protocol::packet::{ClientboundPacketListener, PacketListener};
use super::clientbound_cookie_request_packet::ClientboundCookieRequestPacket;

/// `ClientCookiePacketListener`.
pub trait ClientCookiePacketListener: ClientboundPacketListener {
    /// `handleRequestCookie`.
    fn handle_request_cookie(&mut self, packet: &ClientboundCookieRequestPacket);
}

