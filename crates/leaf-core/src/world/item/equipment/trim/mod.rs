//! Перенос `net.minecraft.world.item.equipment.trim`.

pub mod armor_trim;
pub mod material_asset_group;
pub mod trim_material;
pub mod trim_pattern;

pub use armor_trim::ArmorTrim;
pub use material_asset_group::{AssetInfo, MaterialAssetGroup};
pub use trim_material::TrimMaterial;
pub use trim_pattern::TrimPattern;
