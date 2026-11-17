//! Перенос `net.minecraft.world.damagesource.DamageType` —
//! тип урона для wire-формата.
//!
//! `DamageType.STREAM_CODEC = ByteBufCodecs.holderRegistry(DAMAGE_TYPE)` —
//! на проводе только VarInt holder-id реестра. Полная модель
//! (сообщения смерти, эффекты, истощение) — с реестром `DamageType`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
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

    /// `DamageType.STREAM_CODEC` — holderRegistry: только VarInt id.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<DamageType>> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, holder: &Holder<DamageType>| match holder {
                Holder::Reference(id) => {
                    buf.write_var_int(id + 1);
                }
                Holder::Direct(damage_type) => {
                    panic!("holderRegistry codec cannot encode direct holder: {}", damage_type.id);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let i = buf.read_var_int();
                assert!(i != 0, "invalid damage type holder id");
                Holder::Reference(i - 1)
            },
        )
    }
}
