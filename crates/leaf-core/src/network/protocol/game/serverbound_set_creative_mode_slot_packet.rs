//! `ServerboundSetCreativeModeSlotPacket` — установка предмета в creative-слот.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::item::item_stack::ItemStack;

/// `ServerboundSetCreativeModeSlotPacket`.
#[derive(Clone, Debug)]
pub struct ServerboundSetCreativeModeSlotPacket {
    /// `slotNum`.
    pub slot_num: i16,
    /// `itemStack`.
    pub item_stack: ItemStack,
}

impl ServerboundSetCreativeModeSlotPacket {
    /// `new`.
    pub fn new(slot_num: i16, item_stack: ItemStack) -> Self {
        Self { slot_num, item_stack }
    }

    /// `new` с `i32` slot (как в Java).
    pub fn new_i32(slot_num: i32, item_stack: ItemStack) -> Self {
        Self { slot_num: slot_num as i16, item_stack }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSetCreativeModeSlotPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSetCreativeModeSlotPacket| {
                buf.write_short(p.slot_num as i32);
                ItemStack::optional_untrusted_stream_codec().encode(buf, &p.item_stack);
            },
            |buf: &mut FriendlyByteBuf| {
                let slot_num = buf.read_short();
                let item_stack = ItemStack::optional_untrusted_stream_codec().decode(buf);
                ServerboundSetCreativeModeSlotPacket::new(slot_num, item_stack)
            },
        )
    }
}

impl Packet for ServerboundSetCreativeModeSlotPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_set_creative_mode_slot::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_short(self.slot_num as i32);
        ItemStack::optional_untrusted_stream_codec().encode(buf, &self.item_stack);
    }
}
