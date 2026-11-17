//! Перенос `ClientboundTabListPacket`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundTabListPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundTabListPacket {
    /// `header`.
    pub header: Component,
    /// `footer`.
    pub footer: Component,
}

impl ClientboundTabListPacket {
    pub fn new(header: Component, footer: Component) -> Self {
        Self { header, footer }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundTabListPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundTabListPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let header = component_json::component_stream_codec().decode(buf);
                let footer = component_json::component_stream_codec().decode(buf);
                ClientboundTabListPacket::new(header, footer)
            },
        )
    }
}

impl Packet for ClientboundTabListPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_tab_list::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        component_json::component_stream_codec().encode(buf, &self.header);
        component_json::component_stream_codec().encode(buf, &self.footer);
    }
}
