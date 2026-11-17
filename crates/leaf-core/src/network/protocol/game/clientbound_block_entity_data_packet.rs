//! `ClientboundBlockEntityDataPacket` — данные block entity.

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::nbt::compound::CompoundTag;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::level::block::entity::block_entity_type::BlockEntityType;

/// `ClientboundBlockEntityDataPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundBlockEntityDataPacket {
    /// `pos`.
    pub pos: BlockPos,
    /// `type` — `Holder<BlockEntityType>`.
    pub type_: Holder<BlockEntityType>,
    /// `tag`.
    pub tag: CompoundTag,
}

impl ClientboundBlockEntityDataPacket {
    /// `new`.
    pub fn new(pos: BlockPos, type_: Holder<BlockEntityType>, tag: CompoundTag) -> Self {
        Self { pos, type_, tag }
    }

    /// `getPos`.
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    /// `getType`.
    pub fn get_type(&self) -> &Holder<BlockEntityType> {
        &self.type_
    }

    /// `getTag`.
    pub fn get_tag(&self) -> &CompoundTag {
        &self.tag
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundBlockEntityDataPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundBlockEntityDataPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = BlockPos::stream_codec().decode(buf);
                let type_ = BlockEntityType::stream_codec().decode(buf);
                let tag = crate::network::codec::byte_buf_codecs::compound_tag_codec(false).decode(buf).unwrap_or_default();
                ClientboundBlockEntityDataPacket::new(pos, type_, tag)
            },
        )
    }
}

impl Packet for ClientboundBlockEntityDataPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_block_entity_data::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        BlockPos::stream_codec().encode(buf, &self.pos);
        BlockEntityType::stream_codec().encode(buf, &self.type_);
        crate::network::codec::byte_buf_codecs::compound_tag_codec(false).encode(buf, &Some(self.tag.clone()));
    }
}
