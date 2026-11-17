//! Перенос `ServerboundEntityTagQueryPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundEntityTagQueryPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundEntityTagQueryPacket {
    /// `transactionId`.
    pub transaction_id: i32,
    /// `entityId`.
    pub entity_id: i32,
}

impl ServerboundEntityTagQueryPacket {
    pub fn new(transaction_id: i32, entity_id: i32) -> Self {
        Self { transaction_id, entity_id }
    }

    /// `getTransactionId`.
    pub fn get_transaction_id(&self) -> i32 {
        self.transaction_id
    }

    /// `getEntityId`.
    pub fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundEntityTagQueryPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundEntityTagQueryPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let transaction_id = buf.read_var_int();
                let entity_id = buf.read_var_int();
                ServerboundEntityTagQueryPacket::new(transaction_id, entity_id)
            },
        )
    }
}

impl Packet for ServerboundEntityTagQueryPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_entity_tag_query::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.transaction_id);
        buf.write_var_int(self.entity_id);
    }
}
