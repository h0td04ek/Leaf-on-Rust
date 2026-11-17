//! Перенос `net.minecraft.world.level.block.entity.BannerPattern`.

use crate::core::holder::{Holder, HolderStreamCodec};
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `BannerPattern` — (assetId, translationKey).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BannerPattern {
    /// `assetId`.
    pub asset_id: Identifier,
    /// `translationKey`.
    pub translation_key: String,
}

impl BannerPattern {
    /// `new`.
    pub fn new(asset_id: Identifier, translation_key: impl Into<String>) -> Self {
        Self { asset_id, translation_key: translation_key.into() }
    }

    /// `DIRECT_STREAM_CODEC` — IDENTIFIER + STRING_UTF8.
    pub fn direct_stream_codec() -> impl StreamCodec<FriendlyByteBuf, BannerPattern> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &BannerPattern| {
                buf.write_identifier(&v.asset_id).expect("banner pattern asset id");
                buf.write_utf(&v.translation_key).expect("banner pattern translation key");
            },
            |buf: &mut FriendlyByteBuf| {
                let asset_id = buf.read_identifier().expect("banner pattern asset id");
                let translation_key = buf.read_utf().expect("banner pattern translation key");
                BannerPattern::new(asset_id, translation_key)
            },
        )
    }

    /// `BannerPattern.STREAM_CODEC` — `holder(BANNER_PATTERN, direct)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<BannerPattern>> + Send + Sync + 'static {
        HolderStreamCodec::new(BannerPattern::direct_stream_codec())
    }
}
