//! Перенос `CookiePacketTypes`.

use crate::network::protocol::packet::Packet;
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;

fn id(name: &str) -> Identifier {
    Identifier::with_default_namespace(name).unwrap()
}

/// `CLIENTBOUND_COOKIE_REQUEST`.
pub fn clientbound_cookie_request<P: Packet>() -> PacketType<P> {
    PacketType::new(PacketFlow::CLIENTBOUND, id("cookie_request"))
}

/// `SERVERBOUND_COOKIE_RESPONSE`.
pub fn serverbound_cookie_response<P: Packet>() -> PacketType<P> {
    PacketType::new(PacketFlow::SERVERBOUND, id("cookie_response"))
}
