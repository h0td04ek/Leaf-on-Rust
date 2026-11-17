//! Перенос `net.minecraft.network.protocol.handshake.HandshakeProtocols`.

use super::client_intention_packet::ClientIntentionPacket;
use super::handshake_packet_types;
use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::protocol::protocol_info_builder;
use crate::network::protocol::unbound_protocol::SimpleUnboundProtocol;

/// `SERVERBOUND_TEMPLATE`.
pub fn serverbound_template() -> Box<dyn SimpleUnboundProtocol> {
    protocol_info_builder::serverbound_protocol(ConnectionProtocol::Handshaking, |b| {
        b.add_packet(
            handshake_packet_types::client_intention(),
            ClientIntentionPacket::stream_codec(),
        );
    })
}

/// `SERVERBOUND` — bind с `FriendlyByteBuf::new` (здесь контекст `()`).
pub fn serverbound() -> Box<dyn crate::network::protocol::protocol_info::ProtocolInfo> {
    serverbound_template().bind()
}
