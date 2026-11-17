//! Перенос `net.minecraft.world.item.component.CustomModelData`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `CustomModelData` — float/flag/string/color списки для
/// переопределения модели предмета.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomModelData {
    /// `floats`.
    pub floats: Vec<f32>,
    /// `flags`.
    pub flags: Vec<bool>,
    /// `strings`.
    pub strings: Vec<String>,
    /// `colors`.
    pub colors: Vec<i32>,
}

impl CustomModelData {
    /// `EMPTY`.
    pub const EMPTY: CustomModelData = CustomModelData {
        floats: Vec::new(),
        flags: Vec::new(),
        strings: Vec::new(),
        colors: Vec::new(),
    };

    /// `getFloat(index)`.
    pub fn get_float(&self, index: usize) -> Option<f32> {
        self.floats.get(index).copied()
    }

    /// `getBoolean(index)`.
    pub fn get_boolean(&self, index: usize) -> Option<bool> {
        self.flags.get(index).copied()
    }

    /// `getString(index)`.
    pub fn get_string(&self, index: usize) -> Option<&str> {
        self.strings.get(index).map(String::as_str)
    }

    /// `getColor(index)`.
    pub fn get_color(&self, index: usize) -> Option<i32> {
        self.colors.get(index).copied()
    }

    /// `STREAM_CODEC` — 4 списка: FLOAT/BOOL/STRING_UTF8/INT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, CustomModelData> + Send + Sync + 'static {
        let floats = std::sync::Arc::new(byte_buf_codecs::list(byte_buf_codecs::FLOAT));
        let flags = std::sync::Arc::new(byte_buf_codecs::list(byte_buf_codecs::BOOL));
        let strings = std::sync::Arc::new(byte_buf_codecs::list(byte_buf_codecs::STRING_UTF8));
        let colors = std::sync::Arc::new(byte_buf_codecs::list(byte_buf_codecs::INT));
                let floats2 = floats.clone();
let flags2 = flags.clone();
let strings2 = strings.clone();
let colors2 = colors.clone();
sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &CustomModelData| {
                floats.encode(buf, &v.floats);
                flags.encode(buf, &v.flags);
                strings.encode(buf, &v.strings);
                colors.encode(buf, &v.colors);
            },
            move |buf: &mut FriendlyByteBuf| CustomModelData {
                floats: floats2.decode(buf),
                flags: flags2.decode(buf),
                strings: strings2.decode(buf),
                colors: colors2.decode(buf),
            },
        )
    }
}
