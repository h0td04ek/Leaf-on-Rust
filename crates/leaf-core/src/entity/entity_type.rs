//! Перенос `net.minecraft.world.entity.EntityType` —
//! реестровая запись типа сущности.
//!
//! `EntityType.STREAM_CODEC = ByteBufCodecs.holderRegistry(ENTITY_TYPE)`.
//! Полная модель (фабрика сущностей, размеры, spawn costs,
//! serialization codec) — с портом системы сущностей.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `EntityType` — запись реестра `Registries.ENTITY_TYPE`.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityType {
    /// Registry-ключ типа сущности (`minecraft:zombie` и т.п.).
    pub id: Identifier,
}

impl EntityType {
    /// `new`.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    /// `getDescriptionId`-базовый id (`entity.minecraft.zombie`).
    pub fn id_string(&self) -> String {
        format!("entity.{}", self.id)
    }

    /// `EntityType.STREAM_CODEC` — `holderRegistry(ENTITY_TYPE)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<EntityType>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<EntityType>()
    }
}
