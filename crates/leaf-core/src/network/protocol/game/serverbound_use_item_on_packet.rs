//! Перенос `ServerboundUseItemOnPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::interaction_hand::InteractionHand;
use crate::world::phys::BlockHitResult;

/// `ServerboundUseItemOnPacket`.
#[derive(Clone, Debug)]
pub struct ServerboundUseItemOnPacket {
    /// `hand`.
    pub hand: crate::world::interaction_hand::InteractionHand,
    /// `blockHit`.
    pub block_hit: crate::world::phys::BlockHitResult,
    /// `sequence`.
    pub sequence: i32,
    /// `timestamp`.
    pub timestamp: i64,
}

impl ServerboundUseItemOnPacket {
    /// `ServerboundUseItemOnPacket::new`.
    pub fn new(hand: crate::world::interaction_hand::InteractionHand, block_hit: crate::world::phys::BlockHitResult, sequence: i32, timestamp: i64) -> Self {
        Self { hand, block_hit, sequence, timestamp }
    }
    /// `getHand`.
    pub fn get_hand(&self) -> crate::world::interaction_hand::InteractionHand { self.hand }
    /// `getHitResult`.
    pub fn get_hit_result(&self) -> &crate::world::phys::BlockHitResult { &self.block_hit }
    /// `getSequence`.
    pub fn get_sequence(&self) -> i32 { self.sequence }
    /// `timestamp` (Spigot).
    pub fn timestamp(&self) -> i64 { self.timestamp }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundUseItemOnPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundUseItemOnPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let hand = crate::world::interaction_hand::InteractionHand::by_id(buf.read_var_int());
                let block_hit = buf.read_block_hit_result();
                let sequence = buf.read_var_int();
                let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
                ServerboundUseItemOnPacket::new(hand, block_hit, sequence, timestamp)
            },
        )
    }
}

impl Packet for ServerboundUseItemOnPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_use_item_on::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_enum(&self.hand, self.hand.ordinal());
        buf.write_block_hit_result(&self.block_hit);
        buf.write_var_int(self.sequence);    }
}
