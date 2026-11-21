//! `net.minecraft.world.level.WorldDataConfiguration` —
//! `DataPackConfig` + `FeatureFlagSet` (level.dat `WorldGenSettings`).

use crate::nbt::{CompoundTag, Tag};
use crate::world::flag::feature_flag_set::FeatureFlagSet;

use super::data_pack_config::DataPackConfig;

/// `WorldDataConfiguration`.
#[derive(Clone, Debug)]
pub struct WorldDataConfiguration {
    /// `dataPacks`.
    pub data_packs: DataPackConfig,
    /// `enabledFeatures`.
    pub enabled_features: FeatureFlagSet,
}

impl WorldDataConfiguration {
    /// `ENABLED_FEATURES_ID`.
    pub const ENABLED_FEATURES_ID: &'static str = "enabled_features";

    /// `WorldDataConfiguration.DEFAULT`.
    pub fn default_config() -> Self {
        Self {
            data_packs: DataPackConfig::default_config(),
            enabled_features: FeatureFlagSet::default_flags(),
        }
    }

    /// `expandFeatures(newFeatures)` — join множеств флагов.
    pub fn expand_features(&self, new_features: &FeatureFlagSet) -> Self {
        Self {
            data_packs: self.data_packs.clone(),
            enabled_features: self.enabled_features.join(new_features),
        }
    }

    /// `MAP_CODEC` → NBT (`{DataPacks: {...}, enabled_features: [...]}`).
    pub fn to_nbt(&self) -> Tag {
        let mut c = CompoundTag::new();
        c.put("DataPacks", self.data_packs.to_nbt());
        c.put(
            Self::ENABLED_FEATURES_ID,
            Tag::List(
                self.enabled_features
                    .flags
                    .iter()
                    .map(|id| Tag::String(id.to_short_string()))
                    .collect(),
            ),
        );
        Tag::Compound(c)
    }

    /// `MAP_CODEC.parse` — загрузка из NBT.
    pub fn from_nbt(tag: &Tag) -> Self {
        let mut out = Self::default_config();
        if let Tag::Compound(c) = tag {
            if let Some(dp) = c.get("DataPacks") {
                out.data_packs = DataPackConfig::from_nbt(dp);
            }
            if let Some(Tag::List(l)) = c.get(Self::ENABLED_FEATURES_ID) {
                out.enabled_features.flags = l
                    .iter()
                    .filter_map(|t| match t {
                        Tag::String(s) => {
                            crate::resources::identifier::Identifier::parse(s).ok()
                        }
                        _ => None,
                    })
                    .collect();
            }
        }
        out
    }
}
