//! Перенос `net.minecraft.server.network.Filterable<T>` —
//! (raw, Optional<filtered>) для book-контента.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use std::sync::Arc;

/// `Filterable<T>` — raw-значение + опциональная фильтрованная
/// (покрытая) версия.
#[derive(Clone, Debug, PartialEq)]
pub struct Filterable<T> {
    /// `raw`.
    pub raw: T,
    /// `filtered` (optional).
    pub filtered: Option<T>,
}

impl<T> Filterable<T> {
    /// `passThrough(value)`.
    pub fn pass_through(value: T) -> Self
    where
        T: Clone,
    {
        Self { raw: value.clone(), filtered: None }
    }

    /// `get(filtered)`.
    pub fn get(&self, filtered: bool) -> &T {
        if filtered {
            self.filtered.as_ref().unwrap_or(&self.raw)
        } else {
            &self.raw
        }
    }

    /// `map`.
    pub fn map<U, F: Fn(&T) -> U>(&self, f: F) -> Filterable<U> {
        Filterable {
            raw: f(&self.raw),
            filtered: self.filtered.as_ref().map(|v| f(v)),
        }
    }

    /// `streamCodec(codec)` — raw + optional(filtered).
    /// `codec` — общий кодек значения (Arc, т.к. нужен дважды).
    pub fn stream_codec(
        codec: Arc<dyn StreamCodec<FriendlyByteBuf, T> + Send + Sync>,
    ) -> impl StreamCodec<FriendlyByteBuf, Filterable<T>> + Send + Sync + 'static
    where
        T: Clone + Send + Sync + 'static,
    {
        let codec_raw = codec.clone();
        let codec_raw2 = codec.clone();
        let codec_filtered = codec.clone();
        let optional_enc =
            Arc::new(crate::network::codec::byte_buf_codecs::optional(codec_filtered));
        let optional_dec =
            Arc::new(crate::network::codec::byte_buf_codecs::optional(codec));
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Filterable<T>| {
                codec_raw.encode(buf, &v.raw);
                optional_enc.encode(buf, &v.filtered);
            },
            move |buf: &mut FriendlyByteBuf| {
                let raw = codec_raw2.decode(buf);
                let filtered = optional_dec.decode(buf);
                Filterable { raw, filtered }
            },
        )
    }
}
