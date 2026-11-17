//! Перенос `ClientboundTagQueryPacket`.

use crate::nbt::tag::Tag;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundTagQueryPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundTagQueryPacket {
    /// `transactionId`.
    pub transaction_id: i32,
    /// `tag` (`Option<CompoundTag>` → `Option<Tag>` — всегда compound либо null).
    pub tag: Option<Tag>,
}

impl ClientboundTagQueryPacket {
    /// `ClientboundTagQueryPacket::new`.
    pub fn new(transaction_id: i32, tag: Option<Tag>) -> Self {
        Self { transaction_id, tag }
    }

    /// `getTransactionId`.
    pub fn get_transaction_id(&self) -> i32 {
        self.transaction_id
    }

    /// `getTag`.
    pub fn get_tag(&self) -> Option<&Tag> {
        self.tag.as_ref()
    }

    /// `STREAM_CODEC` — `Packet.codec`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundTagQueryPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundTagQueryPacket| {
                p.write_packet(buf);
            },
            |buf: &mut FriendlyByteBuf| {
                let transaction_id = buf.read_var_int();
                let tag = buf.read_nbt().ok().flatten();
                ClientboundTagQueryPacket::new(transaction_id, tag)
            },
        )
    }

    /// Запись пакета (используется и в `write_packet`).
    pub fn write_to(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.transaction_id);
        let _ = buf.write_nbt(self.tag.as_ref());
    }
}

impl Packet for ClientboundTagQueryPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_tag_query::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        self.write_to(buf);
    }

    /// `isSkippable`.
    fn is_skippable(&self) -> bool {
        true
    }
}
