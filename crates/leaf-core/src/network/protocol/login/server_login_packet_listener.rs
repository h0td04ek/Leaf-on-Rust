//! Перенос `ServerLoginPacketListener`.

use crate::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener;
use crate::network::protocol::packet::ServerboundPacketListener;
use crate::network::connection_protocol::ConnectionProtocol;
use super::serverbound_custom_query_answer_packet::ServerboundCustomQueryAnswerPacket;
use super::serverbound_hello_packet::ServerboundHelloPacket;
use super::serverbound_key_packet::ServerboundKeyPacket;
use super::serverbound_login_acknowledged_packet::ServerboundLoginAcknowledgedPacket;

/// `ServerLoginPacketListener`.
pub trait ServerLoginPacketListener: ServerboundPacketListener + ClientCookiePacketListener {
    /// `protocol()` — `ConnectionProtocol.LOGIN`.
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Login
    }
    /// `handleHello`.
    fn handle_hello(&mut self, _packet: &ServerboundHelloPacket) {}
    /// `handleKey`.
    fn handle_key(&mut self, _packet: &ServerboundKeyPacket) {}
    /// `handleCustomQueryPacket`.
    fn handle_custom_query_packet(&mut self, _packet: &ServerboundCustomQueryAnswerPacket) {}
    /// `handleLoginAcknowledgement`.
    fn handle_login_acknowledgement(&mut self, _packet: &ServerboundLoginAcknowledgedPacket) {}
}
