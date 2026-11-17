//! Перенос `net.minecraft.world.level.block.entity.PotDecorations` —
//! data-component `pot_decorations`.
//!
//! Java хранит `Optional<Item>` ×4; wire — `list(4)` из
//! `ByteBufCodecs.registry(ITEM)` (holderRegistry → VarInt id).
//! Здесь — `Vec<Holder<Item>>` упорядоченных сторон (BRICK для
//! пустых пишет vanilla; значение BRICK — реестровая запись).

use crate::core::holder::Holder;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::item::Item;

/// `PotDecorations` — 4 стороны (back, left, right, front).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PotDecorations {
    /// `back`, `left`, `right`, `front` — holder'ы предметов.
    /// Пустая сторона в Java — `Items.BRICK`.
    pub back: Option<Holder<Item>>,
    /// `left`.
    pub left: Option<Holder<Item>>,
    /// `right`.
    pub right: Option<Holder<Item>>,
    /// `front`.
    pub front: Option<Holder<Item>>,
    /// Registry-id `minecraft:brick` — нужен для кодирования
    /// пустых сторон (Java пишет BRICK). Заполняется при
    /// bootstrap реестра предметов.
    pub brick_holder: Option<Holder<Item>>,
}

impl PotDecorations {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// `ordered()` — 4 holder'а в порядке back/left/right/front;
    /// `None` → `brick_holder` (BRICK), как `optional.orElse(Items.BRICK)`.
    pub fn ordered(&self) -> Vec<Holder<Item>> {
        let brick = self.brick_holder.clone().unwrap_or_else(|| Holder::Reference(-1));
        [self.back.clone(), self.left.clone(), self.right.clone(), self.front.clone()]
            .into_iter()
            .map(|o| o.unwrap_or_else(|| brick.clone()))
            .collect()
    }

    /// `STREAM_CODEC` — list(4) из registry(ITEM) → ordered sides.
    /// Декодированные BRICK-стороны остаются `Some(holder)` —
    /// фильтрация в `Option` делается на уровне доступа через
    /// `brick_holder` (как `getItem` в Java).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, PotDecorations> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list(crate::world::item::item::stream_codec());
        let list_dec = byte_buf_codecs::list(crate::world::item::item::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &PotDecorations| {
                list_enc.encode(buf, &v.ordered());
            },
            move |buf: &mut FriendlyByteBuf| {
                let items: Vec<Holder<Item>> = list_dec.decode(buf);
                let get = |i: usize| items.get(i).cloned();
                PotDecorations {
                    back: get(0),
                    left: get(1),
                    right: get(2),
                    front: get(3),
                    brick_holder: None,
                }
            },
        )
    }
}
