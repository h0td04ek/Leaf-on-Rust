//! Перенос `net.minecraft.world.effect.MobEffect` — тип эффекта
//! для wire-формата (`MobEffect.STREAM_CODEC = holderRegistry`).
//!
//! Как и `ItemStack.item`, значение реестра представлено его
//! `ResourceLocation`-ключом; полная модель эффекта (атрибуты,
//! цвет, мгновенные/периодические флаги) портируется с
//! `MobEffectInstance` и реестром `Registries.MOB_EFFECT`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
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

    /// `MobEffect.STREAM_CODEC` — `ByteBufCodecs.holderRegistry(MOB_EFFECT)`:
    /// на проводе только VarInt holder-id реестра (Reference).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<MobEffect>> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, holder: &Holder<MobEffect>| match holder {
                Holder::Reference(id) => {
                    buf.write_var_int(id + 1);
                }
                Holder::Direct(effect) => {
                    panic!(
                        "holderRegistry codec cannot encode direct holder: {}",
                        effect.id
                    );
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let i = buf.read_var_int();
                assert!(i != 0, "invalid mob effect holder id");
                Holder::Reference(i - 1)
            },
        )
    }
}
