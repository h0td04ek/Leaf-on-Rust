//! Перенос `net.minecraft.world.effect.MobEffect` — тип эффекта
//! для wire-формата (`MobEffect.STREAM_CODEC = holderRegistry`).
//!
//! Как и `ItemStack.item`, значение реестра представлено его
//! `ResourceLocation`-ключом; полная модель эффекта (атрибуты,
//! цвет, мгновенные/периодические флаги) портируется с
//! `MobEffectInstance` и реестром `Registries.MOB_EFFECT`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `MobEffect` — запись реестра `Registries.MOB_EFFECT`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MobEffect {
    /// Registry-ключ эффекта (`minecraft:speed` и т.п.).
    pub id: Identifier,
}

impl MobEffect {
    /// Конструктор по registry-ключу.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    /// `getDescriptionId`-эквивалент — ключ как строка.
    pub fn id_string(&self) -> String {
        self.id.to_string()
    }

    /// `isInstantenous` — базовая реализация `false`;
    /// мгновенные эффекты (harm и т.п.) переопределяют —
    /// поле добавится с полной моделью MobEffect.
    pub fn is_instantaneous(&self) -> bool {
        false
    }

    /// `MobEffect.STREAM_CODEC` — `holderRegistry(MOB_EFFECT)`:
    /// plain VarInt registry-id (0-based).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<MobEffect>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<MobEffect>()
    }
}
