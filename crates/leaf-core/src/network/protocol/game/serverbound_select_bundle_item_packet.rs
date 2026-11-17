//! Перенос `ServerboundSelectBundleItemPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ServerboundSelectBundleItemPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundSelectBundleItemPacket {
    /// `slotId`.
    pub slot_id: i32,
    /// `selectedItemIndex`.
    pub selected_item_index: i32,
}

impl ServerboundSelectBundleItemPacket {
    /// `ServerboundSelectBundleItemPacket::new`.
    pub fn new(slot_id: i32, selected_item_index: i32) -> Self {
        if selected_item_index < 0 && selected_item_index != -1 {
            panic!("Invalid selectedItemIndex: {}", selected_item_index);
        }
        Self { slot_id, selected_item_index }
    }
    /// `slotId`.
    pub fn slot_id(&self) -> i32 { self.slot_id }
    /// `selectedItemIndex`.
    pub fn selected_item_index(&self) -> i32 { self.selected_item_index }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSelectBundleItemPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSelectBundleItemPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let slot_id = buf.read_var_int();
                let selected_item_index = buf.read_var_int();
                ServerboundSelectBundleItemPacket::new(slot_id, selected_item_index)
            },
        )
    }
}

impl Packet for ServerboundSelectBundleItemPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_bundle_item_selected::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.slot_id);
        buf.write_var_int(self.selected_item_index);    }
}
