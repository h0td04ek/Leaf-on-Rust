//! Перенос `net.minecraft.world.item.equipment.trim.TrimMaterial`.

use crate::core::holder::{Holder, HolderStreamCodec};
use crate::network::chat::component_json::component_stream_codec;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::chat::component::Component;
use crate::world::item::equipment::trim::material_asset_group::MaterialAssetGroup;

/// `TrimMaterial` — (assets, description).
#[derive(Clone, Debug, PartialEq)]
pub struct TrimMaterial {
    /// `assets`.
    pub assets: MaterialAssetGroup,
    /// `description`.
    pub description: Component,
}

impl TrimMaterial {
    /// `new`.
    pub fn new(assets: MaterialAssetGroup, description: Component) -> Self {
        Self { assets, description }
    }

    /// `DIRECT_STREAM_CODEC` — MaterialAssetGroup + Component.
    pub fn direct_stream_codec() -> impl StreamCodec<FriendlyByteBuf, TrimMaterial> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &TrimMaterial| {
                MaterialAssetGroup::stream_codec().encode(buf, &v.assets);
                component_stream_codec().encode(buf, &v.description);
            },
            |buf: &mut FriendlyByteBuf| {
                let assets = MaterialAssetGroup::stream_codec().decode(buf);
                let description = component_stream_codec().decode(buf);
                TrimMaterial::new(assets, description)
            },
        )
    }

    /// `TrimMaterial.STREAM_CODEC` — `holder(TRIM_MATERIAL, direct)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<TrimMaterial>> + Send + Sync + 'static {
        HolderStreamCodec::new(TrimMaterial::direct_stream_codec())
    }
}
