//! Перенос `net.minecraft.world.item.equipment.trim.ArmorTrim`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::equipment::trim::trim_material::TrimMaterial;
use crate::world::item::equipment::trim::trim_pattern::TrimPattern;

/// `ArmorTrim` — (material, pattern).
#[derive(Clone, Debug, PartialEq)]
pub struct ArmorTrim {
    /// `material`.
    pub material: Holder<TrimMaterial>,
    /// `pattern`.
    pub pattern: Holder<TrimPattern>,
}

impl ArmorTrim {
    /// `new`.
    pub fn new(material: Holder<TrimMaterial>, pattern: Holder<TrimPattern>) -> Self {
        Self { material, pattern }
    }

    /// `ArmorTrim.STREAM_CODEC` — TrimMaterial holder + TrimPattern holder.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ArmorTrim> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &ArmorTrim| {
                TrimMaterial::stream_codec().encode(buf, &v.material);
                TrimPattern::stream_codec().encode(buf, &v.pattern);
            },
            |buf: &mut FriendlyByteBuf| {
                let material = TrimMaterial::stream_codec().decode(buf);
                let pattern = TrimPattern::stream_codec().decode(buf);
                ArmorTrim::new(material, pattern)
            },
        )
    }
}
