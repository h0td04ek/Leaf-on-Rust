//! `ClientboundSetCursorItemPacket` — установка предмета в курсоре.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::item::item_stack::ItemStack;

/// `ClientboundSetCursorItemPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundSetCursorItemPacket {
    /// `contents`.
    pub contents: ItemStack,
}

impl ClientboundSetCursorItemPacket {
    /// `new`.
    pub fn new(contents: ItemStack) -> Self {
        Self { contents }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetCursorItemPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetCursorItemPacket| {
                ItemStack::optional_stream_codec().encode(buf, &p.contents);
            },
            |buf: &mut FriendlyByteBuf| {
                ClientboundSetCursorItemPacket::new(ItemStack::optional_stream_codec().decode(buf))
            },
        )
    }
}

impl Packet for ClientboundSetCursorItemPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_cursor_item::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        ItemStack::optional_stream_codec().encode(buf, &self.contents);
    }
}
