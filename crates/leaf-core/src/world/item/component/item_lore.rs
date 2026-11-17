//! Перенос `net.minecraft.world.item.component.ItemLore`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ItemLore` — строки описания предмета (Component-список).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemLore {
    /// `lines` — нестилизованные строки.
    pub lines: Vec<Component>,
    /// `styledLines` — стилизованные (для tooltip).
    pub styled_lines: Vec<Component>,
}

impl ItemLore {
    /// `EMPTY`.
    pub const EMPTY: ItemLore = ItemLore { lines: Vec::new(), styled_lines: Vec::new() };

    /// `ItemLore(lines)`.
    pub fn new(lines: Vec<Component>) -> Self {
        Self { lines, styled_lines: Vec::new() }
    }

    /// `withLineAdded`.
    pub fn with_line_added(&self, line: Component) -> Self {
        let mut lines = self.lines.clone();
        lines.push(line);
        Self { lines, styled_lines: self.styled_lines.clone() }
    }

    /// `STREAM_CODEC` — `ComponentSerialization.STREAM_CODEC.list(256)`.
    /// (wire: VarInt count + Component×n; styledLines вычисляются
    /// на клиенте из lines — в Java record хранит только lines
    /// для кодека; styledLines — transient поле конструктора.)
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ItemLore> + Send + Sync + 'static {
        let list = std::sync::Arc::new(byte_buf_codecs::list(component_json::component_stream_codec()));
                let list2 = list.clone();
sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &ItemLore| {
                list.encode(buf, &v.lines);
            },
            move |buf: &mut FriendlyByteBuf| {
                let lines = list2.decode(buf);
                ItemLore { lines, styled_lines: Vec::new() }
            },
        )
    }
}
