//! Перенос `net.minecraft.network.protocol.common.ClientCommonPacketListener`.

use super::clientbound_clear_dialog_packet::ClientboundClearDialogPacket;
use super::custom_payload_packets::ClientboundCustomPayloadPacket;
use super::clientbound_custom_report_details_packet::ClientboundCustomReportDetailsPacket;
use super::clientbound_disconnect_packet::ClientboundDisconnectPacket;
use super::clientbound_keep_alive_packet::ClientboundKeepAlivePacket;
use super::clientbound_ping_packet::ClientboundPingPacket;
use super::clientbound_resource_pack_pop_packet::ClientboundResourcePackPopPacket;
use super::clientbound_resource_pack_push_packet::ClientboundResourcePackPushPacket;
use super::clientbound_server_links_packet::ClientboundServerLinksPacket;
use super::clientbound_store_cookie_packet::ClientboundStoreCookiePacket;
use super::clientbound_transfer_packet::ClientboundTransferPacket;
use super::clientbound_update_tags_packet::ClientboundUpdateTagsPacket;
use crate::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener;
use crate::network::protocol::packet::ClientboundPacketListener;

/// `ClientCommonPacketListener`.
/// (`handleShowDialog` — с портом `server.dialog`.)
pub trait ClientCommonPacketListener: ClientCookiePacketListener + ClientboundPacketListener {
    fn handle_keep_alive(&mut self, packet: &ClientboundKeepAlivePacket);
    fn handle_ping(&mut self, packet: &ClientboundPingPacket);
    fn handle_custom_payload(&mut self, packet: &ClientboundCustomPayloadPacket);
    fn handle_disconnect(&mut self, packet: &ClientboundDisconnectPacket);
    fn handle_resource_pack_push(&mut self, packet: &ClientboundResourcePackPushPacket);
    fn handle_resource_pack_pop(&mut self, packet: &ClientboundResourcePackPopPacket);
    fn handle_update_tags(&mut self, packet: &ClientboundUpdateTagsPacket);
    fn handle_store_cookie(&mut self, packet: &ClientboundStoreCookiePacket);
    fn handle_transfer(&mut self, packet: &ClientboundTransferPacket);
    fn handle_custom_report_details(&mut self, packet: &ClientboundCustomReportDetailsPacket);
    fn handle_server_links(&mut self, packet: &ClientboundServerLinksPacket);
    fn handle_clear_dialog(&mut self, packet: &ClientboundClearDialogPacket);
}
