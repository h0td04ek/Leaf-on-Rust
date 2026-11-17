//! Перенос `net.minecraft.world.level.block.entity.BannerPatternLayers`
//! — data-component `banner_patterns`.

use crate::core::holder::Holder;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::dye_color::DyeColor;
use crate::world::level::block::entity::banner_pattern::BannerPattern;

/// `BannerPatternLayers.Layer` — (pattern, color).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BannerPatternLayer {
    /// `pattern`.
    pub pattern: Holder<BannerPattern>,
    /// `color`.
    pub color: DyeColor,
}

impl BannerPatternLayer {
    /// `new`.
    pub fn new(pattern: Holder<BannerPattern>, color: DyeColor) -> Self {
        Self { pattern, color }
    }

    /// `Layer.STREAM_CODEC` — BannerPattern holder + DyeColor.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Self| {
                BannerPattern::stream_codec().encode(buf, &v.pattern);
                DyeColor::stream_codec().encode(buf, &v.color);
            },
            |buf: &mut FriendlyByteBuf| {
                let pattern = BannerPattern::stream_codec().decode(buf);
                let color = DyeColor::stream_codec().decode(buf);
                BannerPatternLayer::new(pattern, color)
            },
        )
    }
}

/// `BannerPatternLayers` — список слоёв.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BannerPatternLayers {
    /// `layers`.
    pub layers: Vec<BannerPatternLayer>,
}

impl BannerPatternLayers {
    /// `new`.
    pub fn new(layers: Vec<BannerPatternLayer>) -> Self {
        Self { layers }
    }

    /// `STREAM_CODEC` — list(Layer).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list(BannerPatternLayer::stream_codec());
        let list_dec = byte_buf_codecs::list(BannerPatternLayer::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Self| {
                list_enc.encode(buf, &v.layers);
            },
            move |buf: &mut FriendlyByteBuf| BannerPatternLayers {
                layers: list_dec.decode(buf),
            },
        )
    }
}
