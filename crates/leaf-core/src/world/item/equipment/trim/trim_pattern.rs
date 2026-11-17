//! Перенос `net.minecraft.world.item.equipment.trim.TrimPattern`.

use crate::core::holder::{Holder, HolderStreamCodec};
use crate::network::chat::component_json::component_stream_codec;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::chat::component::Component;
use crate::resources::identifier::Identifier;

/// `TrimPattern` — (assetId, description, decal).
#[derive(Clone, Debug, PartialEq)]
pub struct TrimPattern {
    /// `assetId`.
    pub asset_id: Identifier,
    /// `description`.
    pub description: Component,
    /// `decal`.
    pub decal: bool,
}

impl TrimPattern {
    /// `new`.
    pub fn new(asset_id: Identifier, description: Component, decal: bool) -> Self {
        Self { asset_id, description, decal }
    }

    /// `DIRECT_STREAM_CODEC` — IDENTIFIER + Component + BOOL.
    pub fn direct_stream_codec() -> impl StreamCodec<FriendlyByteBuf, TrimPattern> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &TrimPattern| {
                buf.write_identifier(&v.asset_id).expect("trim pattern asset id");
                component_stream_codec().encode(buf, &v.description);
                buf.write_bool(v.decal);
            },
            |buf: &mut FriendlyByteBuf| {
                let asset_id = buf.read_identifier().expect("trim pattern asset id");
                let description = component_stream_codec().decode(buf);
                let decal = buf.read_bool();
                TrimPattern::new(asset_id, description, decal)
            },
        )
    }

    /// `TrimPattern.STREAM_CODEC` — `holder(TRIM_PATTERN, direct)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<TrimPattern>> + Send + Sync + 'static {
        HolderStreamCodec::new(TrimPattern::direct_stream_codec())
    }
}
