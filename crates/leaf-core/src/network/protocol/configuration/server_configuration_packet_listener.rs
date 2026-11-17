//! Перенос `ServerConfigurationPacketListener`.

use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use super::serverbound_accept_code_of_conduct_packet::ServerboundAcceptCodeOfConductPacket;
use super::serverbound_finish_configuration_packet::ServerboundFinishConfigurationPacket;
use super::serverbound_select_known_packs::ServerboundSelectKnownPacks;

/// `ServerConfigurationPacketListener`.
pub trait ServerConfigurationPacketListener: ServerCommonPacketListener {
    /// `protocol()` — `ConnectionProtocol.CONFIGURATION`.
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Configuration
    }
    /// `handleConfigurationFinished`.
    fn handle_configuration_finished(&mut self, _packet: &ServerboundFinishConfigurationPacket) {}
    /// `handleSelectKnownPacks`.
    fn handle_select_known_packs(&mut self, _packet: &ServerboundSelectKnownPacks) {}
    /// `handleAcceptCodeOfConduct`.
    fn handle_accept_code_of_conduct(&mut self, _packet: &ServerboundAcceptCodeOfConductPacket) {}
}
