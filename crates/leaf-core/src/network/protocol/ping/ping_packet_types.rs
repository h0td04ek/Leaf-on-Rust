//! Перенос `PingPacketTypes`.

use crate::network::protocol::packet::Packet;
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;

fn id(name: &str) -> Identifier {
    Identifier::with_default_namespace(name).unwrap()
}

/// `CLIENTBOUND_PONG_RESPONSE`.
pub fn clientbound_pong_response<P: Packet>() -> PacketType<P> {
    PacketType::new(PacketFlow::CLIENTBOUND, id("pong_response"))
}

/// `SERVERBOUND_PING_REQUEST`.
pub fn serverbound_ping_request<P: Packet>() -> PacketType<P> {
    PacketType::new(PacketFlow::SERVERBOUND, id("ping_request"))
}
