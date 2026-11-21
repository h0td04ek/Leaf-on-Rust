//! `net.minecraft.world.level.DataPackConfig` — список
//! включённых/выключенных датапаков (level.dat `DataPacks`).

use crate::nbt::{CompoundTag, Tag};

/// `DataPackConfig`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataPackConfig {
    /// `enabled`.
    pub enabled: Vec<String>,
    /// `disabled`.
    pub disabled: Vec<String>,
}

impl DataPackConfig {
    /// `DataPackConfig.DEFAULT` — `["vanilla"]`, `[]`.
    pub fn default_config() -> Self {
        Self { enabled: vec!["vanilla".to_string()], disabled: Vec::new() }
    }

    /// `getEnabled()`.
    pub fn get_enabled(&self) -> &[String] {
        &self.enabled
    }

    /// `getDisabled()`.
    pub fn get_disabled(&self) -> &[String] {
        &self.disabled
    }

    /// `CODEC` → NBT (`{Enabled: [...], Disabled: [...]}`).
    pub fn to_nbt(&self) -> Tag {
        let mut c = CompoundTag::new();
        c.put(
            "Enabled",
            Tag::List(self.enabled.iter().map(|s| Tag::String(s.clone())).collect()),
        );
        c.put(
            "Disabled",
            Tag::List(self.disabled.iter().map(|s| Tag::String(s.clone())).collect()),
        );
        Tag::Compound(c)
    }

    /// `CODEC.parse` — загрузка из NBT.
    pub fn from_nbt(tag: &Tag) -> Self {
        let mut out = Self::default();
        if let Tag::Compound(c) = tag {
            if let Some(Tag::List(l)) = c.get("Enabled") {
                out.enabled = l
                    .iter()
                    .filter_map(|t| match t {
                        Tag::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();
            }
            if let Some(Tag::List(l)) = c.get("Disabled") {
                out.disabled = l
                    .iter()
                    .filter_map(|t| match t {
                        Tag::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();
            }
        }
        out
    }
}
