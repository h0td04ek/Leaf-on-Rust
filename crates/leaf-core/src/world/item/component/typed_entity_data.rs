//! Перенос `net.minecraft.world.item.component.TypedEntityData` —
//! (type, tag) для `entity_data`/`block_entity_data`/`bees`.

use crate::nbt::compound::CompoundTag;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `TypedEntityData<T>` — (type, tag) — тип + NBT данных сущности/блок-энтити.
/// Аналог `world.item.component.TypedEntityData`.
#[derive(Clone, Debug, PartialEq)]
pub struct TypedEntityData<T> {
    /// `type`.
    pub type_value: T,
    /// `tag`.
    pub tag: CompoundTag,
}

impl<T> TypedEntityData<T> {
    /// `new`.
    pub fn new(type_value: T, tag: CompoundTag) -> Self {
        Self { type_value, tag }
    }

    /// `TypedEntityData.streamCodec(idCodec)` — idCodec + COMPOUND_TAG.
    pub fn stream_codec(
        id_codec: impl StreamCodec<FriendlyByteBuf, T> + Send + Sync + 'static,
    ) -> impl StreamCodec<FriendlyByteBuf, TypedEntityData<T>> + Send + Sync + 'static
    where
        T: Send + Sync + 'static,
    {
        let tag_enc = byte_buf_codecs::COMPOUND_TAG_REQUIRED;
        let tag_dec = byte_buf_codecs::COMPOUND_TAG_REQUIRED;
        let id_codec: std::sync::Arc<dyn StreamCodec<FriendlyByteBuf, T> + Send + Sync> =
            std::sync::Arc::new(id_codec);
        let id_enc = id_codec.clone();
        let id_dec = id_codec;
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &TypedEntityData<T>| {
                id_enc.encode(buf, &v.type_value);
                tag_enc.encode(buf, &v.tag);
            },
            move |buf: &mut FriendlyByteBuf| {
                let type_value = id_dec.decode(buf);
                let tag = tag_dec.decode(buf);
                TypedEntityData::new(type_value, tag)
            },
        )
    }
}

