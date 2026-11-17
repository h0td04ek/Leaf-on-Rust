//! Перенос `net.minecraft.network.protocol.configuration.ConfigurationProtocols`.

use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::protocol::common::common_packet_types as common_types;
use crate::network::protocol::common::{
    clientbound_clear_dialog_packet,
    clientbound_custom_report_details_packet, clientbound_disconnect_packet,
    clientbound_keep_alive_packet, clientbound_ping_packet, clientbound_resource_pack_pop_packet,
    clientbound_resource_pack_push_packet, clientbound_server_links_packet,
    clientbound_store_cookie_packet, clientbound_transfer_packet,
    clientbound_update_tags_packet, custom_payload_packets, serverbound_misc_packets,
    serverbound_keep_alive_packet, serverbound_pong_packet,
};
use crate::network::protocol::cookie::cookie_packet_types;
use crate::network::protocol::cookie::{
    clientbound_cookie_request_packet, serverbound_cookie_response_packet,
};
use crate::network::protocol::protocol_info_builder;
use crate::network::protocol::unbound_protocol::SimpleUnboundProtocol;
use super::clientbound_code_of_conduct_packet::ClientboundCodeOfConductPacket;
use super::clientbound_finish_configuration_packet::ClientboundFinishConfigurationPacket;
use super::clientbound_registry_data_packet::ClientboundRegistryDataPacket;
use super::clientbound_reset_chat_packet::ClientboundResetChatPacket;
use super::clientbound_select_known_packs::ClientboundSelectKnownPacks;
use super::clientbound_update_enabled_features_packet::ClientboundUpdateEnabledFeaturesPacket;
use super::configuration_packet_types::ConfigurationPacketTypes;
use super::serverbound_accept_code_of_conduct_packet::ServerboundAcceptCodeOfConductPacket;
use super::serverbound_finish_configuration_packet::ServerboundFinishConfigurationPacket;
use super::serverbound_select_known_packs::ServerboundSelectKnownPacks;

/// `SERVERBOUND_TEMPLATE`.
pub fn serverbound_template() -> Box<dyn SimpleUnboundProtocol> {
    protocol_info_builder::serverbound_protocol(ConnectionProtocol::Configuration, |b| {
        b.add_packet(
            common_types::serverbound_client_information::<custom_payload_packets::ServerboundClientInformationPacket>(),
            custom_payload_packets::ServerboundClientInformationPacket::stream_codec(),
        )
        .add_packet(
            cookie_packet_types::serverbound_cookie_response::<serverbound_cookie_response_packet::ServerboundCookieResponsePacket>(),
            serverbound_cookie_response_packet::ServerboundCookieResponsePacket::stream_codec(),
        )
        .add_packet(
            common_types::serverbound_custom_payload::<custom_payload_packets::ServerboundCustomPayloadPacket>(),
            custom_payload_packets::ServerboundCustomPayloadPacket::stream_codec(),
        )
        .add_packet(
            ConfigurationPacketTypes::serverbound_finish_configuration::<ServerboundFinishConfigurationPacket>(),
            ServerboundFinishConfigurationPacket::stream_codec(),
        )
        .add_packet(
            common_types::serverbound_keep_alive::<serverbound_keep_alive_packet::ServerboundKeepAlivePacket>(),
            serverbound_keep_alive_packet::ServerboundKeepAlivePacket::stream_codec(),
        )
        .add_packet(
            common_types::serverbound_pong::<serverbound_pong_packet::ServerboundPongPacket>(),
            serverbound_pong_packet::ServerboundPongPacket::stream_codec(),
        )
        .add_packet(
            common_types::serverbound_resource_pack::<serverbound_misc_packets::ServerboundResourcePackPacket>(),
            serverbound_misc_packets::ServerboundResourcePackPacket::stream_codec(),
        )
        .add_packet(
            ConfigurationPacketTypes::serverbound_select_known_packs::<ServerboundSelectKnownPacks>(),
            ServerboundSelectKnownPacks::stream_codec(),
        )
        .add_packet(
            common_types::serverbound_custom_click_action::<serverbound_misc_packets::ServerboundCustomClickActionPacket>(),
            serverbound_misc_packets::ServerboundCustomClickActionPacket::stream_codec(),
        )
        .add_packet(
            ConfigurationPacketTypes::serverbound_accept_code_of_conduct::<ServerboundAcceptCodeOfConductPacket>(),
            ServerboundAcceptCodeOfConductPacket::stream_codec(),
        );
    })
}

/// `SERVERBOUND`.
pub fn serverbound() -> Box<dyn crate::network::protocol::protocol_info::ProtocolInfo> {
    serverbound_template().bind()
}

/// `CLIENTBOUND_TEMPLATE`.
///
/// `ClientboundShowDialogPacket` не регистрируется — `server.dialog`
/// ещё не портирован (см. common::client_common_packet_listener).
pub fn clientbound_template() -> Box<dyn SimpleUnboundProtocol> {
    protocol_info_builder::clientbound_protocol(ConnectionProtocol::Configuration, |b| {
        b.add_packet(
            cookie_packet_types::clientbound_cookie_request::<clientbound_cookie_request_packet::ClientboundCookieRequestPacket>(),
            clientbound_cookie_request_packet::ClientboundCookieRequestPacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_custom_payload::<custom_payload_packets::ClientboundCustomPayloadPacket>(),
            custom_payload_packets::ClientboundCustomPayloadPacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_disconnect::<clientbound_disconnect_packet::ClientboundDisconnectPacket>(),
            clientbound_disconnect_packet::ClientboundDisconnectPacket::stream_codec(),
        )
        .add_packet(
            ConfigurationPacketTypes::clientbound_finish_configuration::<ClientboundFinishConfigurationPacket>(),
            ClientboundFinishConfigurationPacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_keep_alive::<clientbound_keep_alive_packet::ClientboundKeepAlivePacket>(),
            clientbound_keep_alive_packet::ClientboundKeepAlivePacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_ping::<clientbound_ping_packet::ClientboundPingPacket>(),
            clientbound_ping_packet::ClientboundPingPacket::stream_codec(),
        )
        .add_packet(
            ConfigurationPacketTypes::clientbound_reset_chat::<ClientboundResetChatPacket>(),
            ClientboundResetChatPacket::stream_codec(),
        )
        .add_packet(
            ConfigurationPacketTypes::clientbound_registry_data::<ClientboundRegistryDataPacket>(),
            ClientboundRegistryDataPacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_resource_pack_pop::<clientbound_resource_pack_pop_packet::ClientboundResourcePackPopPacket>(),
            clientbound_resource_pack_pop_packet::ClientboundResourcePackPopPacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_resource_pack_push::<clientbound_resource_pack_push_packet::ClientboundResourcePackPushPacket>(),
            clientbound_resource_pack_push_packet::ClientboundResourcePackPushPacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_store_cookie::<clientbound_store_cookie_packet::ClientboundStoreCookiePacket>(),
            clientbound_store_cookie_packet::ClientboundStoreCookiePacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_transfer::<clientbound_transfer_packet::ClientboundTransferPacket>(),
            clientbound_transfer_packet::ClientboundTransferPacket::stream_codec(),
        )
        .add_packet(
            ConfigurationPacketTypes::clientbound_update_enabled_features::<ClientboundUpdateEnabledFeaturesPacket>(),
            ClientboundUpdateEnabledFeaturesPacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_update_tags::<clientbound_update_tags_packet::ClientboundUpdateTagsPacket>(),
            clientbound_update_tags_packet::ClientboundUpdateTagsPacket::stream_codec(),
        )
        .add_packet(
            ConfigurationPacketTypes::clientbound_select_known_packs::<ClientboundSelectKnownPacks>(),
            ClientboundSelectKnownPacks::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_custom_report_details::<clientbound_custom_report_details_packet::ClientboundCustomReportDetailsPacket>(),
            clientbound_custom_report_details_packet::ClientboundCustomReportDetailsPacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_server_links::<clientbound_server_links_packet::ClientboundServerLinksPacket>(),
            clientbound_server_links_packet::ClientboundServerLinksPacket::stream_codec(),
        )
        .add_packet(
            common_types::clientbound_clear_dialog::<clientbound_clear_dialog_packet::ClientboundClearDialogPacket>(),
            clientbound_clear_dialog_packet::ClientboundClearDialogPacket::stream_codec(),
        )
        .add_packet(
            ConfigurationPacketTypes::clientbound_code_of_conduct::<ClientboundCodeOfConductPacket>(),
            ClientboundCodeOfConductPacket::stream_codec(),
        );
    })
}

/// `CLIENTBOUND`.
pub fn clientbound() -> Box<dyn crate::network::protocol::protocol_info::ProtocolInfo> {
    clientbound_template().bind()
}
