//! Перенос `net.minecraft.world.level.block.entity.BlockEntityType` —
//! реестровая запись типа блок-энтити.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `BlockEntityType` — запись реестра `Registries.BLOCK_ENTITY_TYPE`.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockEntityType {
    /// Registry-ключ типа.
    pub id: Identifier,
}

impl BlockEntityType {
    /// `new`.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    /// `BlockEntityType.STREAM_CODEC` — `holderRegistry(BLOCK_ENTITY_TYPE)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<BlockEntityType>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<BlockEntityType>()
    }
}
