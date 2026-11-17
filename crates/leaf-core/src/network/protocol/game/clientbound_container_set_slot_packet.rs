//! `ClientboundContainerSetSlotPacket` — установка предмета в слот контейнера.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::item::item_stack::ItemStack;

/// `ClientboundContainerSetSlotPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundContainerSetSlotPacket {
    /// `containerId`.
    pub container_id: i32,
    /// `stateId`.
    pub state_id: i32,
    /// `slot`.
    pub slot: i32,
    /// `itemStack`.
    pub item_stack: ItemStack,
}

impl ClientboundContainerSetSlotPacket {
    /// `new`.
    pub fn new(container_id: i32, state_id: i32, slot: i32, item_stack: ItemStack) -> Self {
        Self { container_id, state_id, slot, item_stack }
    }

    /// `getContainerId`.
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }

    /// `getSlot`.
    pub fn get_slot(&self) -> i32 {
        self.slot
    }

    /// `getItem`.
    pub fn get_item(&self) -> &ItemStack {
        &self.item_stack
    }

    /// `getStateId`.
    pub fn get_state_id(&self) -> i32 {
        self.state_id
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundContainerSetSlotPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundContainerSetSlotPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let container_id = buf.read_container_id();
                let state_id = buf.read_var_int();
                let slot = buf.read_short() as i32;
                let item_stack = ItemStack::optional_stream_codec().decode(buf);
                ClientboundContainerSetSlotPacket::new(container_id, state_id, slot, item_stack)
            },
        )
    }
}

impl Packet for ClientboundContainerSetSlotPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_container_set_slot::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        buf.write_var_int(self.state_id);
        buf.write_short(self.slot);
        ItemStack::optional_stream_codec().encode(buf, &self.item_stack);
    }
}
