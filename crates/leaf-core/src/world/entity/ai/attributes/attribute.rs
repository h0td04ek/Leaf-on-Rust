//! Перенос `net.minecraft.world.entity.ai.attributes.Attribute` —
//! тип атрибута (реестровая запись; `STREAM_CODEC` = holderRegistry).

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `Attribute` — запись реестра `Registries.ATTRIBUTE`.
/// Полная модель (default value, sandbox, синхронизация) —
/// с системой атрибутов сущностей.
#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    /// Registry-ключ атрибута.
    pub id: Identifier,
}

impl Attribute {
    /// `new`.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    /// `Attribute.STREAM_CODEC` — `holderRegistry(ATTRIBUTE)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<Attribute>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<Attribute>()
    }
}
