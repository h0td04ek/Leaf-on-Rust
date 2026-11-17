//! Перенос `net.minecraft.network.protocol.common.ServerCommonPacketListener`.

use super::custom_payload_packets::ClientboundCustomPayloadPacket;
use super::custom_payload_packets::ServerboundClientInformationPacket;
use super::serverbound_misc_packets::ServerboundCustomClickActionPacket;
use super::custom_payload_packets::ServerboundCustomPayloadPacket;
use super::serverbound_keep_alive_packet::ServerboundKeepAlivePacket;
use super::serverbound_pong_packet::ServerboundPongPacket;
use super::serverbound_misc_packets::ServerboundResourcePackPacket;
use crate::network::protocol::cookie::server_cookie_packet_listener::ServerCookiePacketListener;
use crate::network::protocol::packet::ServerboundPacketListener;

/// `ServerCommonPacketListener`.
pub trait ServerCommonPacketListener: ServerCookiePacketListener + ServerboundPacketListener {
    fn handle_keep_alive(&mut self, packet: &ServerboundKeepAlivePacket);
    fn handle_pong(&mut self, packet: &ServerboundPongPacket);
    fn handle_custom_payload(&mut self, packet: &ServerboundCustomPayloadPacket);
    fn handle_resource_pack_response(&mut self, packet: &ServerboundResourcePackPacket);
    fn handle_client_information(&mut self, packet: &ServerboundClientInformationPacket);
    fn handle_custom_click_action(&mut self, packet: &ServerboundCustomClickActionPacket);
}
