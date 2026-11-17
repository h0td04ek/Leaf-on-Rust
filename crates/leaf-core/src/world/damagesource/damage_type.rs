//! Перенос `net.minecraft.world.damagesource.DamageType` —
//! тип урона для wire-формата.
//!
//! `DamageType.STREAM_CODEC = ByteBufCodecs.holderRegistry(DAMAGE_TYPE)` —
//! на проводе только VarInt holder-id реестра. Полная модель
//! (сообщения смерти, эффекты, истощение) — с реестром `DamageType`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `DamageType` — запись реестра `Registries.DAMAGE_TYPE`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DamageType {
    /// Registry-ключ типа урона (`minecraft:in_fire` и т.п.).
    pub id: Identifier,
}

impl DamageType {
    /// Конструктор по registry-ключу.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    /// `DamageType.STREAM_CODEC` — `holderRegistry(DAMAGE_TYPE)`:
    /// plain VarInt registry-id (0-based).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<DamageType>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<DamageType>()
    }
}
