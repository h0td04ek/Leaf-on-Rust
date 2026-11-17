//! Перенос `ClientConfigurationPacketListener`.

use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use super::clientbound_code_of_conduct_packet::ClientboundCodeOfConductPacket;
use super::clientbound_finish_configuration_packet::ClientboundFinishConfigurationPacket;
use super::clientbound_registry_data_packet::ClientboundRegistryDataPacket;
use super::clientbound_reset_chat_packet::ClientboundResetChatPacket;
use super::clientbound_select_known_packs::ClientboundSelectKnownPacks;
use super::clientbound_update_enabled_features_packet::ClientboundUpdateEnabledFeaturesPacket;

/// `ClientConfigurationPacketListener`.
pub trait ClientConfigurationPacketListener: ClientCommonPacketListener {
    /// `protocol()` — `ConnectionProtocol.CONFIGURATION`.
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Configuration
    }
    /// `handleCodeOfConduct`.
    fn handle_code_of_conduct(&mut self, _packet: &ClientboundCodeOfConductPacket) {}
    /// `handleConfigurationFinished`.
    fn handle_configuration_finished(&mut self, _packet: &ClientboundFinishConfigurationPacket) {}
    /// `handleRegistryData`.
    fn handle_registry_data(&mut self, _packet: &ClientboundRegistryDataPacket) {}
    /// `handleEnabledFeatures`.
    fn handle_enabled_features(&mut self, _packet: &ClientboundUpdateEnabledFeaturesPacket) {}
    /// `handleSelectKnownPacks`.
    fn handle_select_known_packs(&mut self, _packet: &ClientboundSelectKnownPacks) {}
    /// `handleResetChat`.
    fn handle_reset_chat(&mut self, _packet: &ClientboundResetChatPacket) {}
}
