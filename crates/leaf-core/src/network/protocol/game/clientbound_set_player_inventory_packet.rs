//! `ClientboundSetPlayerInventoryPacket` — установка предмета в инвентарь игрока.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::item::item_stack::ItemStack;

/// `ClientboundSetPlayerInventoryPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundSetPlayerInventoryPacket {
    /// `slot`.
    pub slot: i32,
    /// `contents`.
    pub contents: ItemStack,
}

impl ClientboundSetPlayerInventoryPacket {
    /// `new`.
    pub fn new(slot: i32, contents: ItemStack) -> Self {
        Self { slot, contents }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetPlayerInventoryPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetPlayerInventoryPacket| {
                buf.write_var_int(p.slot);
                ItemStack::optional_stream_codec().encode(buf, &p.contents);
            },
            |buf: &mut FriendlyByteBuf| {
                let slot = buf.read_var_int();
                let contents = ItemStack::optional_stream_codec().decode(buf);
                ClientboundSetPlayerInventoryPacket::new(slot, contents)
            },
        )
    }
}

impl Packet for ClientboundSetPlayerInventoryPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_player_inventory::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.slot);
        ItemStack::optional_stream_codec().encode(buf, &self.contents);
    }
}
