//! Перенос `net.minecraft.network.protocol.common` — common-пакеты
//! конфигурации/геймплея (keepalive, ping, disconnect, resource pack,
//! cookies, transfer, custom payload, tags, server links).

pub mod client_common_packet_listener;
pub mod clientbound_clear_dialog_packet;
pub mod clientbound_custom_report_details_packet;
pub mod clientbound_disconnect_packet;
pub mod clientbound_keep_alive_packet;
pub mod clientbound_ping_packet;
pub mod clientbound_resource_pack_pop_packet;
pub mod clientbound_resource_pack_push_packet;
pub mod clientbound_server_links_packet;
pub mod clientbound_store_cookie_packet;
pub mod clientbound_transfer_packet;
pub mod clientbound_update_tags_packet;
pub mod common_packet_types;
pub mod custom;
pub mod custom_payload_packets;
pub mod server_common_packet_listener;
pub mod serverbound_keep_alive_packet;
pub mod serverbound_misc_packets;
pub mod serverbound_pong_packet;

// Реэкспорты для удобства (имена как в Java-классах).
pub use clientbound_clear_dialog_packet::ClientboundClearDialogPacket;
pub use clientbound_custom_report_details_packet::ClientboundCustomReportDetailsPacket;
pub use clientbound_disconnect_packet::ClientboundDisconnectPacket;
pub use clientbound_keep_alive_packet::ClientboundKeepAlivePacket;
pub use clientbound_ping_packet::ClientboundPingPacket;
pub use clientbound_resource_pack_pop_packet::ClientboundResourcePackPopPacket;
pub use clientbound_resource_pack_push_packet::ClientboundResourcePackPushPacket;
pub use clientbound_server_links_packet::ClientboundServerLinksPacket;
pub use clientbound_store_cookie_packet::ClientboundStoreCookiePacket;
pub use clientbound_transfer_packet::ClientboundTransferPacket;
pub use clientbound_update_tags_packet::{ClientboundUpdateTagsPacket, NetworkPayload};
pub use custom::custom_packet_payload::CustomPacketPayload;
pub use custom_payload_packets::{
    ClientboundCustomPayloadPacket, ServerboundClientInformationPacket,
    ServerboundCustomPayloadPacket,
};
pub use serverbound_misc_packets::{
    ResourcePackAction, ServerboundCustomClickActionPacket, ServerboundResourcePackPacket,
};
pub use serverbound_keep_alive_packet::ServerboundKeepAlivePacket;
pub use serverbound_pong_packet::ServerboundPongPacket;
pub mod clientbound_show_dialog_packet;
