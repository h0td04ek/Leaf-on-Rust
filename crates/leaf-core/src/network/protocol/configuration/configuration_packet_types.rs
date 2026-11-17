//! Перенос `ConfigurationPacketTypes`.

use crate::network::protocol::packet::Packet;
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;

fn id(name: &str) -> Identifier {
    Identifier::with_default_namespace(name).unwrap()
}

/// `ConfigurationPacketTypes`.
pub struct ConfigurationPacketTypes;

impl ConfigurationPacketTypes {
    /// `CLIENTBOUND_CODE_OF_CONDUCT`.
    pub fn clientbound_code_of_conduct<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("code_of_conduct"))
    }
    /// `CLIENTBOUND_FINISH_CONFIGURATION`.
    pub fn clientbound_finish_configuration<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("finish_configuration"))
    }
    /// `CLIENTBOUND_REGISTRY_DATA`.
    pub fn clientbound_registry_data<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("registry_data"))
    }
    /// `CLIENTBOUND_RESET_CHAT`.
    pub fn clientbound_reset_chat<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("reset_chat"))
    }
    /// `CLIENTBOUND_SELECT_KNOWN_PACKS`.
    pub fn clientbound_select_known_packs<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("select_known_packs"))
    }
    /// `CLIENTBOUND_UPDATE_ENABLED_FEATURES`.
    pub fn clientbound_update_enabled_features<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("update_enabled_features"))
    }
    /// `SERVERBOUND_ACCEPT_CODE_OF_CONDUCT`.
    pub fn serverbound_accept_code_of_conduct<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("accept_code_of_conduct"))
    }
    /// `SERVERBOUND_FINISH_CONFIGURATION`.
    pub fn serverbound_finish_configuration<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("finish_configuration"))
    }
    /// `SERVERBOUND_SELECT_KNOWN_PACKS`.
    pub fn serverbound_select_known_packs<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("select_known_packs"))
    }
}
