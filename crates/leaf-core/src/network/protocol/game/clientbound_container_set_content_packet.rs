//! `ClientboundContainerSetContentPacket` — содержимое контейнера.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::item::item_stack::ItemStack;

/// `ClientboundContainerSetContentPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundContainerSetContentPacket {
    /// `containerId`.
    pub container_id: i32,
    /// `stateId`.
    pub state_id: i32,
    /// `items`.
    pub items: Vec<ItemStack>,
    /// `carriedItem`.
    pub carried_item: ItemStack,
}

impl ClientboundContainerSetContentPacket {
    /// `new`.
    pub fn new(container_id: i32, state_id: i32, items: Vec<ItemStack>, carried_item: ItemStack) -> Self {
        Self { container_id, state_id, items, carried_item }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundContainerSetContentPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundContainerSetContentPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let container_id = buf.read_container_id();
                let state_id = buf.read_var_int();
                let items = ItemStack::optional_list_stream_codec().decode(buf);
                let carried_item = ItemStack::optional_stream_codec().decode(buf);
                ClientboundContainerSetContentPacket::new(container_id, state_id, items, carried_item)
            },
        )
    }
}

impl Packet for ClientboundContainerSetContentPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_container_set_content::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        buf.write_var_int(self.state_id);
        ItemStack::optional_list_stream_codec().encode(buf, &self.items);
        ItemStack::optional_stream_codec().encode(buf, &self.carried_item);
    }
}
