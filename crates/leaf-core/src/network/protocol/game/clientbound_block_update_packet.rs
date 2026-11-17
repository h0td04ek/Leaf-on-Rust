//! Перенос `ClientboundBlockUpdatePacket`.

use crate::core::block_pos::BlockPos;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundBlockUpdatePacket`.
///
/// `block_state` — id в глобальной палитре `Block.BLOCK_STATE_REGISTRY`
/// (VarInt; реестр блоков будет портирован отдельно).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundBlockUpdatePacket {
    /// `pos`.
    pub pos: BlockPos,
    /// `blockState` — global palette id.
    pub block_state: i32,
}

impl ClientboundBlockUpdatePacket {
    /// `ClientboundBlockUpdatePacket::new`.
    pub fn new(pos: BlockPos, block_state: i32) -> Self {
        Self { pos, block_state }
    }

    /// `getPos`.
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    /// `getBlockState`.
    pub fn get_block_state(&self) -> i32 {
        self.block_state
    }

    /// `STREAM_CODEC` — BlockPos + idMapper(BLOCK_STATE_REGISTRY) ≡ VarInt.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundBlockUpdatePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundBlockUpdatePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = buf.read_block_pos();
                let block_state = buf.read_var_int();

                ClientboundBlockUpdatePacket::new(pos, block_state)
            },
        )
    }
}

impl Packet for ClientboundBlockUpdatePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_block_update::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.pos);
        buf.write_var_int(self.block_state);
    }
}
