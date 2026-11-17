//! Перенос `net.minecraft.world.item.component.CustomData`.

use crate::nbt::compound::CompoundTag;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `CustomData` — (tag: CompoundTag).
/// `STREAM_CODEC` — `ByteBufCodecs.COMPOUND_TAG` (deprecated в Java,
/// но используется для `bucket_entity_data`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomData {
    /// `tag`.
    pub tag: CompoundTag,
}

impl CustomData {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self { tag: CompoundTag::new() }
    }

    /// `new`.
    pub fn new(tag: CompoundTag) -> Self {
        Self { tag }
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.tag.is_empty()
    }

    /// `STREAM_CODEC` — COMPOUND_TAG.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, CustomData> + Send + Sync + 'static {
        let tag_enc = byte_buf_codecs::COMPOUND_TAG_REQUIRED;
        let tag_dec = byte_buf_codecs::COMPOUND_TAG_REQUIRED;
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &CustomData| {
                tag_enc.encode(buf, &v.tag);
            },
            move |buf: &mut FriendlyByteBuf| CustomData::new(tag_dec.decode(buf)),
        )
    }
}
