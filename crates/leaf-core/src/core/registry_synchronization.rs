//! Перенос `net.minecraft.core.RegistrySynchronization` —
//! `PackedRegistryEntry` (запись реестра для `ClientboundRegistryDataPacket`).

use crate::nbt::tag::Tag;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `RegistrySynchronization.PackedRegistryEntry` — (id, optional NBT data).
#[derive(Clone, Debug, PartialEq)]
pub struct PackedRegistryEntry {
    /// `id` — идентификатор элемента реестра.
    pub id: Identifier,
    /// `data` — `None` = элемент из known pack (не передаётся).
    pub data: Option<Tag>,
}

impl PackedRegistryEntry {
    /// `new PackedRegistryEntry(id, data)`.
    pub fn new(id: Identifier, data: Option<Tag>) -> Self {
        Self { id, data }
    }

    /// `STREAM_CODEC` — `Identifier` + `TAG.optional()`
    /// (bool + NBT-тег без End-маркера при отсутствии).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, PackedRegistryEntry> {
        let tag_codec = byte_buf_codecs::TAG;
        sync_fn(
            move |buf: &mut FriendlyByteBuf, e: &PackedRegistryEntry| {
                buf.write_identifier(&e.id).expect("invalid identifier");
                match &e.data {
                    Some(tag) => {
                        buf.write_bool(true);
                        tag_codec.encode(buf, &Some(tag.clone()));
                    }
                    None => {
                        buf.write_bool(false);
                    }
                }
            },
            move |buf: &mut FriendlyByteBuf| {
                let tag_codec = byte_buf_codecs::TAG;
                let id = buf.read_identifier().expect("invalid identifier");
                let data = if buf.read_bool() {
                    tag_codec.decode(buf)
                } else {
                    None
                };
                PackedRegistryEntry::new(id, data)
            },
        )
    }
}
