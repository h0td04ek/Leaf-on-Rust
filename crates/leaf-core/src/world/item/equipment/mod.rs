//! Перенос `net.minecraft.world.item.equipment`.

pub mod equipment_asset;
pub mod equipment_assets;
pub mod equippable;
pub mod trim;

pub use equipment_asset::EquipmentAsset;
pub use equipment_assets::{get as equipment_asset, root_id as equipment_assets_root_id};
pub use equippable::Equippable;
