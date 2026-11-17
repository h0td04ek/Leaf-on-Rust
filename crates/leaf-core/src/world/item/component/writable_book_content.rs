//! Перенос `net.minecraft.world.item.component.WritableBookContent`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::server::network::filterable::Filterable;

/// `WritableBookContent` — `List<Filterable<String>>` страниц
/// (≤100, строка ≤1024).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WritableBookContent {
    /// `pages`.
    pub pages: Vec<Filterable<String>>,
}

impl WritableBookContent {
    /// `MAX_PAGES` / `MAX_PAGE_LENGTH`.
    pub const MAX_PAGES: usize = 100;
    pub const MAX_PAGE_LENGTH: usize = 1024;

    /// `new`.
    pub fn new(pages: Vec<Filterable<String>>) -> Self {
        Self { pages }
    }

    /// `pageCount`.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// `STREAM_CODEC` — list(100, Filterable(stringUtf8(1024))).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, WritableBookContent> + Send + Sync + 'static {
        let page = Filterable::stream_codec(std::sync::Arc::new(byte_buf_codecs::string_utf8(1024)));
        let list_enc = byte_buf_codecs::list_limited(Self::MAX_PAGES, page);
        let page2 = Filterable::stream_codec(std::sync::Arc::new(byte_buf_codecs::string_utf8(1024)));
        let list_dec = byte_buf_codecs::list_limited(Self::MAX_PAGES, page2);
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &WritableBookContent| {
                list_enc.encode(buf, &v.pages);
            },
            move |buf: &mut FriendlyByteBuf| WritableBookContent {
                pages: list_dec.decode(buf),
            },
        )
    }
}
