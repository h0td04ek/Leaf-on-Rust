//! Перенос `net.minecraft.world.item.component.WrittenBookContent`.

use crate::network::chat::component_json::component_stream_codec;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::server::network::filterable::Filterable;

/// `WrittenBookContent` — (title?, author, generation, pages,
/// resolved).
#[derive(Clone, Debug, PartialEq)]
pub struct WrittenBookContent {
    /// `title`.
    pub title: Option<Filterable<String>>,
    /// `author`.
    pub author: String,
    /// `generation`.
    pub generation: i32,
    /// `pages`.
    pub pages: Vec<Filterable<crate::network::chat::component::Component>>,
    /// `resolved`.
    pub resolved: bool,
}

impl WrittenBookContent {
    /// `MAX_TITLE_LENGTH` / `MAX_AUTHOR_LENGTH` / `MAX_PAGES` /
    /// `MAX_GENERATION`.
    pub const MAX_TITLE_LENGTH: usize = 32;
    pub const MAX_AUTHOR_LENGTH: usize = 32;
    pub const MAX_PAGES: usize = 100;
    pub const MAX_GENERATION: i32 = 3;

    /// `new`.
    pub fn new(
        title: Option<Filterable<String>>,
        author: String,
        generation: i32,
        pages: Vec<Filterable<crate::network::chat::component::Component>>,
        resolved: bool,
    ) -> Self {
        Self {
            title,
            author,
            generation,
            pages,
            resolved,
        }
    }

    /// `STREAM_CODEC` — порядок полей Java.
    pub fn stream_codec(
    ) -> impl StreamCodec<FriendlyByteBuf, WrittenBookContent> + Send + Sync + 'static {
        let title_enc =
            Filterable::stream_codec(std::sync::Arc::new(byte_buf_codecs::string_utf8(32)));
        let title_dec =
            Filterable::stream_codec(std::sync::Arc::new(byte_buf_codecs::string_utf8(32)));
        let page = Filterable::stream_codec(std::sync::Arc::new(component_stream_codec()));
        let pages_enc = byte_buf_codecs::list_limited(Self::MAX_PAGES, page);
        let page2 = Filterable::stream_codec(std::sync::Arc::new(component_stream_codec()));
        let pages_dec = byte_buf_codecs::list_limited(Self::MAX_PAGES, page2);
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &WrittenBookContent| {
                let empty_title = Filterable {
                    raw: String::new(),
                    filtered: None,
                };
                title_enc.encode(buf, v.title.as_ref().unwrap_or(&empty_title));
                buf.write_utf(&v.author).expect("author");
                buf.write_var_int(v.generation);
                pages_enc.encode(buf, &v.pages);
                buf.write_bool(v.resolved);
            },
            move |buf: &mut FriendlyByteBuf| {
                let title = title_dec.decode(buf);
                let author = buf.read_utf().expect("author");
                let generation = buf.read_var_int();
                let pages = pages_dec.decode(buf);
                let resolved = buf.read_bool();
                WrittenBookContent::new(Some(title), author, generation, pages, resolved)
            },
        )
    }
}
