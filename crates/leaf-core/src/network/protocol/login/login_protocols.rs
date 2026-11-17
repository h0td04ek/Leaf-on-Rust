//! Перенос `net.minecraft.network.protocol.login.LoginProtocols`.

use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::protocol::cookie::cookie_packet_types;
use crate::network::protocol::cookie::{clientbound_cookie_request_packet, serverbound_cookie_response_packet};
use crate::network::protocol::protocol_info_builder;
use crate::network::protocol::unbound_protocol::SimpleUnboundProtocol;
use super::clientbound_custom_query_packet::ClientboundCustomQueryPacket;
use super::clientbound_hello_packet::ClientboundHelloPacket;
use super::clientbound_login_compression_packet::ClientboundLoginCompressionPacket;
use super::clientbound_login_disconnect_packet::ClientboundLoginDisconnectPacket;
use super::clientbound_login_finished_packet::ClientboundLoginFinishedPacket;
use super::serverbound_custom_query_answer_packet::ServerboundCustomQueryAnswerPacket;
use super::serverbound_hello_packet::ServerboundHelloPacket;
use super::serverbound_key_packet::ServerboundKeyPacket;
use super::serverbound_login_acknowledged_packet::ServerboundLoginAcknowledgedPacket;
use super::login_packet_types::LoginPacketTypes;

/// `SERVERBOUND_TEMPLATE`.
pub fn serverbound_template() -> Box<dyn SimpleUnboundProtocol> {
    protocol_info_builder::serverbound_protocol(ConnectionProtocol::Login, |b| {
        b.add_packet(
            LoginPacketTypes::serverbound_hello::<ServerboundHelloPacket>(),
            ServerboundHelloPacket::stream_codec(),
        )
        .add_packet(
            LoginPacketTypes::serverbound_key::<ServerboundKeyPacket>(),
            ServerboundKeyPacket::stream_codec(),
        )
        .add_packet(
            LoginPacketTypes::serverbound_custom_query_answer::<ServerboundCustomQueryAnswerPacket>(),
            ServerboundCustomQueryAnswerPacket::stream_codec(),
        )
        .add_packet(
            LoginPacketTypes::serverbound_login_acknowledged::<ServerboundLoginAcknowledgedPacket>(),
            ServerboundLoginAcknowledgedPacket::stream_codec(),
        )
        .add_packet(
            cookie_packet_types::serverbound_cookie_response::<serverbound_cookie_response_packet::ServerboundCookieResponsePacket>(),
            serverbound_cookie_response_packet::ServerboundCookieResponsePacket::stream_codec(),
        );
    })
}

/// `SERVERBOUND`.
pub fn serverbound() -> Box<dyn crate::network::protocol::protocol_info::ProtocolInfo> {
    serverbound_template().bind()
}

/// `CLIENTBOUND_TEMPLATE`.
pub fn clientbound_template() -> Box<dyn SimpleUnboundProtocol> {
    protocol_info_builder::clientbound_protocol(ConnectionProtocol::Login, |b| {
        b.add_packet(
            LoginPacketTypes::clientbound_login_disconnect::<ClientboundLoginDisconnectPacket>(),
            ClientboundLoginDisconnectPacket::stream_codec(),
        )
        .add_packet(
            LoginPacketTypes::clientbound_hello::<ClientboundHelloPacket>(),
            ClientboundHelloPacket::stream_codec(),
        )
        .add_packet(
            LoginPacketTypes::clientbound_login_finished::<ClientboundLoginFinishedPacket>(),
            ClientboundLoginFinishedPacket::stream_codec(),
        )
        .add_packet(
            LoginPacketTypes::clientbound_login_compression::<ClientboundLoginCompressionPacket>(),
            ClientboundLoginCompressionPacket::stream_codec(),
        )
        .add_packet(
            LoginPacketTypes::clientbound_custom_query::<ClientboundCustomQueryPacket>(),
            ClientboundCustomQueryPacket::stream_codec(),
        )
        .add_packet(
            cookie_packet_types::clientbound_cookie_request::<clientbound_cookie_request_packet::ClientboundCookieRequestPacket>(),
            clientbound_cookie_request_packet::ClientboundCookieRequestPacket::stream_codec(),
        );
    })
}

/// `CLIENTBOUND`.
pub fn clientbound() -> Box<dyn crate::network::protocol::protocol_info::ProtocolInfo> {
    clientbound_template().bind()
}
