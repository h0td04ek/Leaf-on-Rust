//! `ClientboundDebugBlockValuePacket` — debug-значение блока.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::core::block_pos::BlockPos;
use crate::util::debug::debug_subscription::Update;

/// `ClientboundDebugBlockValuePacket`.
#[derive(Clone, Debug)]
pub struct ClientboundDebugBlockValuePacket {
    /// `blockPos`.
    pub block_pos: BlockPos,
    /// `update`.
    pub update: Update,
}

impl ClientboundDebugBlockValuePacket {
    /// `new`.
    pub fn new(block_pos: BlockPos, update: Update) -> Self {
        Self { block_pos, update }
    }

    /// `STREAM_CODEC` — `StreamCodec.composite(BlockPos.STREAM_CODEC, update, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundDebugBlockValuePacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ClientboundDebugBlockValuePacket| {
                BlockPos::stream_codec().encode(buf, &pkt.block_pos);
                Update::stream_codec().encode(buf, &pkt.update);
            },
            |buf: &mut FriendlyByteBuf| {
                let block_pos = BlockPos::stream_codec().decode(buf);
                let update = Update::stream_codec().decode(buf);
                Self::new(block_pos, update)
            },
        )
    }
}

impl Packet for ClientboundDebugBlockValuePacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_debug_block_value::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        BlockPos::stream_codec().encode(buf, &self.block_pos);
        Update::stream_codec().encode(buf, &self.update);
    }
}
