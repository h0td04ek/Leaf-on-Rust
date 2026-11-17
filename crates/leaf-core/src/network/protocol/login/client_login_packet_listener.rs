//! Перенос `ClientLoginPacketListener`.

use crate::network::protocol::cookie::server_cookie_packet_listener::ServerCookiePacketListener;
use crate::network::protocol::packet::ClientboundPacketListener;
use crate::network::connection_protocol::ConnectionProtocol;
use super::clientbound_custom_query_packet::ClientboundCustomQueryPacket;
use super::clientbound_hello_packet::ClientboundHelloPacket;
use super::clientbound_login_compression_packet::ClientboundLoginCompressionPacket;
use super::clientbound_login_disconnect_packet::ClientboundLoginDisconnectPacket;
use super::clientbound_login_finished_packet::ClientboundLoginFinishedPacket;

/// `ClientLoginPacketListener`.
pub trait ClientLoginPacketListener: ClientboundPacketListener + ServerCookiePacketListener {
    /// `protocol()` — `ConnectionProtocol.LOGIN`.
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Login
    }
    /// `handleHello`.
    fn handle_hello(&mut self, _packet: &ClientboundHelloPacket) {}
    /// `handleLoginFinished`.
    fn handle_login_finished(&mut self, _packet: &ClientboundLoginFinishedPacket) {}
    /// `handleDisconnect`.
    fn handle_disconnect(&mut self, _packet: &ClientboundLoginDisconnectPacket) {}
    /// `handleCompression`.
    fn handle_compression(&mut self, _packet: &ClientboundLoginCompressionPacket) {}
    /// `handleCustomQuery`.
    fn handle_custom_query(&mut self, _packet: &ClientboundCustomQueryPacket) {}
}
