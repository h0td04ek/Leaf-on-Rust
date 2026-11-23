//! Перенос `ClientboundOpenScreenPacket`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json::component_stream_codec;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundOpenScreenPacket`.
///
/// `type` — числовой ID в реестре `Registries.MENU`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundOpenScreenPacket {
    /// `containerId`.
    pub container_id: i32,
    /// `type` — registry-ID типа меню.
    pub menu_type: i32,
    /// `title`.
    pub title: Component,
}

impl ClientboundOpenScreenPacket {
    /// `ClientboundOpenScreenPacket::new`.
    pub fn new(container_id: i32, menu_type: i32, title: Component) -> Self {
        Self {
            container_id,
            menu_type,
            title,
        }
    }

    /// `getContainerId`.
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }

    /// `getType`.
    pub fn get_type(&self) -> i32 {
        self.menu_type
    }

    /// `getTitle`.
    pub fn get_title(&self) -> &Component {
        &self.title
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundOpenScreenPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundOpenScreenPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let container_id = buf.read_container_id();
                let menu_type = buf.read_var_int();
                let title = component_stream_codec().decode(buf);
                ClientboundOpenScreenPacket::new(container_id, menu_type, title)
            },
        )
    }
}

impl Packet for ClientboundOpenScreenPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_open_screen::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        buf.write_var_int(self.menu_type);
        component_stream_codec().encode(buf, &self.title);
    }
}
