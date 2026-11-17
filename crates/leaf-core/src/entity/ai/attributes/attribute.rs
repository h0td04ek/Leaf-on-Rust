//! Перенос `net.minecraft.world.entity.ai.attributes.Attribute` —
//! тип атрибута для wire-формата.
//!
//! `Attribute.STREAM_CODEC = ByteBufCodecs.holderRegistry(ATTRIBUTE)` —
//! на проводе только VarInt holder-id реестра (как у `MobEffect`).
//! Полная модель атрибута (диапазоны, `toColumn`, снапшоты) —
//! с портом `AttributeInstance`/`AttributeMap`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `Attribute` — запись реестра `Registries.ATTRIBUTE`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Attribute {
    /// Registry-ключ атрибута (`minecraft:generic.max_health` и т.п.).
    pub id: Identifier,
}

impl Attribute {
    /// Конструктор по registry-ключу.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    /// `Attribute.STREAM_CODEC` — holderRegistry: только VarInt id.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<Attribute>> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, holder: &Holder<Attribute>| match holder {
                Holder::Reference(id) => {
                    buf.write_var_int(id + 1);
                }
                Holder::Direct(attr) => {
                    panic!("holderRegistry codec cannot encode direct holder: {}", attr.id);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let i = buf.read_var_int();
                assert!(i != 0, "invalid attribute holder id");
                Holder::Reference(i - 1)
            },
        )
    }
}
