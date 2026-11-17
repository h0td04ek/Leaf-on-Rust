//! `ClientboundSetEquipmentPacket` — экипировка сущности.

use crate::entity::equipment_slot::EquipmentSlot;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::item::item_stack::ItemStack;

/// `ClientboundSetEquipmentPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundSetEquipmentPacket {
    /// `entity`.
    pub entity: i32,
    /// `slots` — `(EquipmentSlot, ItemStack)` пары.
    pub slots: Vec<(EquipmentSlot, ItemStack)>,
}

impl ClientboundSetEquipmentPacket {
    /// `new`.
    pub fn new(entity: i32, slots: Vec<(EquipmentSlot, ItemStack)>) -> Self {
        Self { entity, slots }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetEquipmentPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetEquipmentPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity = buf.read_var_int();
                let mut slots = Vec::new();
                loop {
                    let byte = buf.read_byte() as u8;
                    let slot = EquipmentSlot::by_id((byte & 0x7F) as i32);
                    let stack = ItemStack::optional_stream_codec().decode(buf);
                    slots.push((slot, stack));
                    if (byte & 0x80) == 0 { break; }
                }
                ClientboundSetEquipmentPacket::new(entity, slots)
            },
        )
    }
}

impl Packet for ClientboundSetEquipmentPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_equipment::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity);
        let size = self.slots.len();
        for (i, (slot, stack)) in self.slots.iter().enumerate() {
            let flag = i != size - 1;
            let ordinal = slot.get_id() as u8;
            buf.write_byte(if flag { (ordinal | 0x80) as i32 } else { ordinal as i32 });
            ItemStack::optional_stream_codec().encode(buf, stack);
        }
    }
}
