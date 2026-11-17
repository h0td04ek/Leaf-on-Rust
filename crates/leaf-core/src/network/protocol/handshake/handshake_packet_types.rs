//! Перенос `net.minecraft.network.protocol.handshake.HandshakePacketTypes`.

use crate::network::protocol::packet::Packet;
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;

/// `CLIENT_INTENTION` packet type.
pub fn client_intention<P: Packet>() -> PacketType<P> {
    PacketType::new(
        PacketFlow::SERVERBOUND,
        Identifier::with_default_namespace("intention").unwrap(),
    )
}
