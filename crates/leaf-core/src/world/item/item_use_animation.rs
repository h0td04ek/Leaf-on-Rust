//! Перенос `net.minecraft.world.item.ItemUseAnimation`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ItemUseAnimation` — анимация использования предмета.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemUseAnimation {
    /// `NONE`.
    NONE,
    /// `EAT`.
    EAT,
    /// `DRINK`.
    DRINK,
    /// `BLOCK`.
    BLOCK,
    /// `BOW`.
    BOW,
    /// `TRIDENT`.
    TRIDENT,
    /// `CROSSBOW`.
    CROSSBOW,
    /// `SPYGLASS`.
    SPYGLASS,
    /// `TOOT_HORN`.
    TOOT_HORN,
    /// `BRUSH`.
    BRUSH,
    /// `BUNDLE`.
    BUNDLE,
    /// `SPEAR`.
    SPEAR,
}

impl ItemUseAnimation {
    /// `values()` — порядок = id.
    pub const VALUES: [ItemUseAnimation; 12] = [
        ItemUseAnimation::NONE, ItemUseAnimation::EAT, ItemUseAnimation::DRINK,
        ItemUseAnimation::BLOCK, ItemUseAnimation::BOW, ItemUseAnimation::TRIDENT,
        ItemUseAnimation::CROSSBOW, ItemUseAnimation::SPYGLASS, ItemUseAnimation::TOOT_HORN,
        ItemUseAnimation::BRUSH, ItemUseAnimation::BUNDLE, ItemUseAnimation::SPEAR,
    ];

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        *self as i32
    }

    /// `ByIdMap.continuous(..., ZERO)` — вне диапазона → `NONE`.
    pub fn by_id(id: i32) -> ItemUseAnimation {
        if (0..12).contains(&id) {
            Self::VALUES[id as usize]
        } else {
            Self::VALUES[0]
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            ItemUseAnimation::NONE => "none",
            ItemUseAnimation::EAT => "eat",
            ItemUseAnimation::DRINK => "drink",
            ItemUseAnimation::BLOCK => "block",
            ItemUseAnimation::BOW => "bow",
            ItemUseAnimation::TRIDENT => "trident",
            ItemUseAnimation::CROSSBOW => "crossbow",
            ItemUseAnimation::SPYGLASS => "spyglass",
            ItemUseAnimation::TOOT_HORN => "toot_horn",
            ItemUseAnimation::BRUSH => "brush",
            ItemUseAnimation::BUNDLE => "bundle",
            ItemUseAnimation::SPEAR => "spear",
        }
    }

    /// `isBlocking`-подобные флаги Java (`eat`/`drink`/`spear` = true
    /// в конструкторе — поле `stack`-поведение; для wire не нужно).
    pub fn consumes_via_animation(&self) -> bool {
        matches!(self, ItemUseAnimation::EAT | ItemUseAnimation::DRINK | ItemUseAnimation::SPEAR)
    }

    /// `STREAM_CODEC` — idMapper.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ItemUseAnimation> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &ItemUseAnimation| {
                buf.write_var_int(v.get_id());
            },
            |buf: &mut FriendlyByteBuf| ItemUseAnimation::by_id(buf.read_var_int()),
        )
    }
}
