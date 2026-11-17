//! Перенос `LoginPacketTypes`.

use crate::network::protocol::packet::Packet;
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;

fn id(name: &str) -> Identifier {
    Identifier::with_default_namespace(name).unwrap()
}

/// `LoginPacketTypes` — статические `PacketType` логин-фазы.
pub struct LoginPacketTypes;

impl LoginPacketTypes {
    /// `CLIENTBOUND_CUSTOM_QUERY`.
    pub fn clientbound_custom_query<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("custom_query"))
    }
    /// `CLIENTBOUND_LOGIN_FINISHED`.
    pub fn clientbound_login_finished<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("login_finished"))
    }
    /// `CLIENTBOUND_HELLO`.
    pub fn clientbound_hello<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("hello"))
    }
    /// `CLIENTBOUND_LOGIN_COMPRESSION`.
    pub fn clientbound_login_compression<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("login_compression"))
    }
    /// `CLIENTBOUND_LOGIN_DISCONNECT`.
    pub fn clientbound_login_disconnect<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("login_disconnect"))
    }
    /// `SERVERBOUND_CUSTOM_QUERY_ANSWER`.
    pub fn serverbound_custom_query_answer<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("custom_query_answer"))
    }
    /// `SERVERBOUND_HELLO`.
    pub fn serverbound_hello<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("hello"))
    }
    /// `SERVERBOUND_KEY`.
    pub fn serverbound_key<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("key"))
    }
    /// `SERVERBOUND_LOGIN_ACKNOWLEDGED`.
    pub fn serverbound_login_acknowledged<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("login_acknowledged"))
    }
}
