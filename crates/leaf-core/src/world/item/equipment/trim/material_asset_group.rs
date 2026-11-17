//! Перенос `net.minecraft.world.item.equipment.trim.MaterialAssetGroup`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::world::item::equipment::equipment_asset::EquipmentAsset;
use std::collections::BTreeMap;

/// `MaterialAssetGroup.AssetInfo` — (suffix).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetInfo {
    /// `suffix`.
    pub suffix: String,
}

impl AssetInfo {
    /// `new`.
    pub fn new(suffix: impl Into<String>) -> Self {
        Self { suffix: suffix.into() }
    }

    /// `AssetInfo.STREAM_CODEC` — STRING_UTF8.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, AssetInfo> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &AssetInfo| {
                buf.write_utf(&v.suffix).expect("asset info suffix");
            },
            |buf: &mut FriendlyByteBuf| AssetInfo::new(buf.read_utf().expect("asset info suffix")),
        )
    }
}

/// `MaterialAssetGroup` — (base, overrides: map<ResourceKey<EquipmentAsset>, AssetInfo>).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialAssetGroup {
    /// `base`.
    pub base: AssetInfo,
    /// `overrides`.
    pub overrides: BTreeMap<ResourceKey<EquipmentAsset>, AssetInfo>,
}

impl MaterialAssetGroup {
    /// `create(name)`.
    pub fn create(name: impl Into<String>) -> Self {
        Self { base: AssetInfo::new(name), overrides: BTreeMap::new() }
    }

    /// `SEPARATOR`.
    pub const SEPARATOR: &'static str = "_";

    /// `assetId(assetKey)`.
    pub fn asset_id(&self, asset_key: &ResourceKey<EquipmentAsset>) -> &AssetInfo {
        self.overrides.get(asset_key).unwrap_or(&self.base)
    }

    /// `MaterialAssetGroup.STREAM_CODEC` — AssetInfo +
    /// map(ResourceKey.streamCodec(ROOT_ID) → AssetInfo).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MaterialAssetGroup> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, v: &MaterialAssetGroup| {
                AssetInfo::stream_codec().encode(buf, &v.base);
                // map: VarInt count + (Identifier key + AssetInfo value)
                buf.write_var_int(v.overrides.len() as i32);
                for (k, val) in &v.overrides {
                    buf.write_identifier(&k.identifier).expect("equipment asset key");
                    AssetInfo::stream_codec().encode(buf, val);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let base = AssetInfo::stream_codec().decode(buf);
                let count = buf.read_var_int();
                let mut overrides = BTreeMap::new();
                for _ in 0..count.max(0) {
                    let id = buf.read_identifier().expect("equipment asset key");
                    let key = ResourceKey::create_with_registry_name(
                        Identifier::parse("minecraft:equipment_asset").expect("equipment_asset registry"),
                        id,
                    );
                    let val = AssetInfo::stream_codec().decode(buf);
                    overrides.insert(key, val);
                }
                MaterialAssetGroup { base, overrides }
            },
        )
    }
}
