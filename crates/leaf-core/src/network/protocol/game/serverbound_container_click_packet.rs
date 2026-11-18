//! `ServerboundContainerClickPacket` — клик в контейнере.

use std::collections::HashMap;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::hashed_stack::HashedStack;
use crate::world::inventory::ClickType;

/// `ServerboundContainerClickPacket`.
#[derive(Clone, Debug)]
pub struct ServerboundContainerClickPacket {
    /// `containerId`.
    pub container_id: i32,
    /// `stateId`.
    pub state_id: i32,
    /// `slotNum`.
    pub slot_num: i16,
    /// `buttonNum`.
    pub button_num: i8,
    /// `clickType`.
    pub click_type: ClickType,
    /// `changedSlots` — `Int2ObjectMap<HashedStack>` → `HashMap<i16, HashedStack>`.
    pub changed_slots: HashMap<i16, HashedStack>,
    /// `carriedItem`.
    pub carried_item: HashedStack,
}

impl ServerboundContainerClickPacket {
    /// `MAX_SLOT_COUNT`.
    pub const MAX_SLOT_COUNT: usize = 128;

    /// `new`.
    pub fn new(
        container_id: i32,
        state_id: i32,
        slot_num: i16,
        button_num: i8,
        click_type: ClickType,
        changed_slots: HashMap<i16, HashedStack>,
        carried_item: HashedStack,
    ) -> Self {
        Self { container_id, state_id, slot_num, button_num, click_type, changed_slots, carried_item }
    }

    /// `SLOTS_STREAM_CODEC` — `ByteBufCodecs.map(Int2ObjectOpenHashMap::new, SHORT.map(Short::intValue, Integer::shortValue), HashedStack.STREAM_CODEC, 128)`.
    fn slots_stream_codec() -> impl StreamCodec<FriendlyByteBuf, HashMap<i16, HashedStack>> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, slots: &HashMap<i16, HashedStack>| {
                buf.write_var_int(slots.len() as i32);
                for (slot, stack) in slots {
                    buf.write_short(*slot as i32);
                    HashedStack::stream_codec().encode(buf, stack);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let len = buf.read_var_int();
                let mut slots = HashMap::new();
                for _ in 0..len {
                    let slot = buf.read_short();
                    let stack = HashedStack::stream_codec().decode(buf);
                    slots.insert(slot, stack);
                }
                slots
            },
        )
    }

    /// `STREAM_CODEC` — `StreamCodec.composite(CONTAINER_ID, VAR_INT, SHORT, BYTE, ClickType, SLOTS, HashedStack, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundContainerClickPacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ServerboundContainerClickPacket| {
                buf.write_container_id(pkt.container_id);
                buf.write_var_int(pkt.state_id);
                buf.write_short(pkt.slot_num as i32);
                buf.write_byte(pkt.button_num as i32);
                ClickType::stream_codec().encode(buf, &pkt.click_type);
                Self::slots_stream_codec().encode(buf, &pkt.changed_slots);
                HashedStack::stream_codec().encode(buf, &pkt.carried_item);
            },
            |buf: &mut FriendlyByteBuf| {
                let container_id = buf.read_container_id();
                let state_id = buf.read_var_int();
                let slot_num = buf.read_short();
                let button_num = buf.read_byte() as i8;
                let click_type = ClickType::stream_codec().decode(buf);
                let changed_slots = Self::slots_stream_codec().decode(buf);
                let carried_item = HashedStack::stream_codec().decode(buf);
                Self::new(container_id, state_id, slot_num, button_num, click_type, changed_slots, carried_item)
            },
        )
    }
}

impl Packet for ServerboundContainerClickPacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_container_click::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        buf.write_var_int(self.state_id);
        buf.write_short(self.slot_num as i32);
        buf.write_byte(self.button_num as i32);
        ClickType::stream_codec().encode(buf, &self.click_type);
        Self::slots_stream_codec().encode(buf, &self.changed_slots);
        HashedStack::stream_codec().encode(buf, &self.carried_item);
    }
}
