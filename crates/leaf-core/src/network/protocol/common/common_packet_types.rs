//! Перенос `net.minecraft.network.protocol.common.CommonPacketTypes`.

use crate::network::protocol::packet::Packet;
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;

fn cb(name: &str) -> Identifier {
    Identifier::with_default_namespace(name).unwrap()
}

macro_rules! packet_types {
    ($($name:ident => $packet:literal),* $(,)?) => {
        $(pub fn $name<P: Packet>() -> PacketType<P> {
            PacketType::new(PacketFlow::CLIENTBOUND, cb($packet))
        })*
    };
}

packet_types! {
    clientbound_clear_dialog => "clear_dialog",
    clientbound_custom_payload => "custom_payload",
    clientbound_custom_report_details => "custom_report_details",
    clientbound_disconnect => "disconnect",
    clientbound_keep_alive => "keep_alive",
    clientbound_ping => "ping",
    clientbound_resource_pack_pop => "resource_pack_pop",
    clientbound_resource_pack_push => "resource_pack_push",
    clientbound_server_links => "server_links",
    clientbound_store_cookie => "store_cookie",
    clientbound_transfer => "transfer",
    clientbound_update_tags => "update_tags",
}

macro_rules! packet_types_sb {
    ($($name:ident => $packet:literal),* $(,)?) => {
        $(pub fn $name<P: Packet>() -> PacketType<P> {
            PacketType::new(PacketFlow::SERVERBOUND, cb($packet))
        })*
    };
}

packet_types_sb! {
    serverbound_client_information => "client_information",
    serverbound_custom_payload => "custom_payload",
    serverbound_keep_alive => "keep_alive",
    serverbound_pong => "pong",
    serverbound_resource_pack => "resource_pack",
    serverbound_custom_click_action => "custom_click_action",
}
