//! Перенос `net.minecraft.network.protocol.status.StatusPacketTypes`.

use crate::network::protocol::packet::Packet;
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;

fn id(name: &str) -> Identifier {
    Identifier::with_default_namespace(name).unwrap()
}

/// `CLIENTBOUND_STATUS_RESPONSE`.
pub fn clientbound_status_response<P: Packet>() -> PacketType<P> {
    PacketType::new(PacketFlow::CLIENTBOUND, id("status_response"))
}

/// `SERVERBOUND_STATUS_REQUEST`.
pub fn serverbound_status_request<P: Packet>() -> PacketType<P> {
    PacketType::new(PacketFlow::SERVERBOUND, id("status_request"))
}
