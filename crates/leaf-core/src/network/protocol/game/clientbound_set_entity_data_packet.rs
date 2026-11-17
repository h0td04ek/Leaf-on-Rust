//! `ClientboundSetEntityDataPacket` — синхронизация данных entity.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::syncher::synched_entity_data::DataValue;

/// `ClientboundSetEntityDataPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundSetEntityDataPacket {
    /// `id` — entity id.
    pub id: i32,
    /// `packedItems` — список `DataValue`.
    pub packed_items: Vec<DataValue>,
}

impl ClientboundSetEntityDataPacket {
    /// `EOF_MARKER`.
    pub const EOF_MARKER: u8 = 255;

    /// `new`.
    pub fn new(id: i32, packed_items: Vec<DataValue>) -> Self {
        Self { id, packed_items }
    }

    /// `pack` — запись `DataValue` списка + EOF marker.
    fn pack(data_values: &[DataValue], buffer: &mut FriendlyByteBuf) {
        for data_value in data_values {
            data_value.write(buffer);
        }
        buffer.write_byte(Self::EOF_MARKER as i32);
    }

    /// `unpack` — чтение `DataValue` списка до EOF marker.
    fn unpack(buffer: &mut FriendlyByteBuf) -> Vec<DataValue> {
        let mut list = Vec::new();
        loop {
            let i = buffer.read_unsigned_byte();
            if i == Self::EOF_MARKER as i32 {
                break;
            }
            list.push(DataValue::read(buffer, i));
        }
        list
    }

    /// `STREAM_CODEC` — `Packet.codec(write, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetEntityDataPacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ClientboundSetEntityDataPacket| {
                buf.write_var_int(pkt.id);
                Self::pack(&pkt.packed_items, buf);
            },
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let items = Self::unpack(buf);
                Self::new(id, items)
            },
        )
    }
}

impl Packet for ClientboundSetEntityDataPacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_entity_data::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        Self::pack(&self.packed_items, buf);
    }
}
