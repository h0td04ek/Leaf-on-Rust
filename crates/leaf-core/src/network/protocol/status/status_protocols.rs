//! Перенос `net.minecraft.network.protocol.status.StatusProtocols`.

use super::clientbound_status_response_packet::ClientboundStatusResponsePacket;
use super::serverbound_status_request_packet::ServerboundStatusRequestPacket;
use super::{status_packet_types};
use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::protocol::ping::{clientbound_pong_response_packet::ClientboundPongResponsePacket, serverbound_ping_request_packet::ServerboundPingRequestPacket};
use crate::network::protocol::protocol_info_builder;
use crate::network::protocol::unbound_protocol::SimpleUnboundProtocol;

/// `SERVERBOUND_TEMPLATE`.
pub fn serverbound_template() -> Box<dyn SimpleUnboundProtocol> {
    protocol_info_builder::serverbound_protocol(ConnectionProtocol::Status, |b| {
        b.add_packet(
            status_packet_types::serverbound_status_request(),
            ServerboundStatusRequestPacket::stream_codec(),
        )
        .add_packet(
            crate::network::protocol::ping::ping_packet_types::serverbound_ping_request(),
            ServerboundPingRequestPacket::stream_codec(),
        );
    })
}

/// `SERVERBOUND`.
pub fn serverbound() -> Box<dyn crate::network::protocol::protocol_info::ProtocolInfo> {
    serverbound_template().bind()
}

/// `CLIENTBOUND_TEMPLATE`.
pub fn clientbound_template() -> Box<dyn SimpleUnboundProtocol> {
    protocol_info_builder::clientbound_protocol(ConnectionProtocol::Status, |b| {
        b.add_packet(
            status_packet_types::clientbound_status_response(),
            ClientboundStatusResponsePacket::stream_codec(),
        )
        .add_packet(
            crate::network::protocol::ping::ping_packet_types::clientbound_pong_response(),
            ClientboundPongResponsePacket::stream_codec(),
        );
    })
}

/// `CLIENTBOUND`.
pub fn clientbound() -> Box<dyn crate::network::protocol::protocol_info::ProtocolInfo> {
    clientbound_template().bind()
}
