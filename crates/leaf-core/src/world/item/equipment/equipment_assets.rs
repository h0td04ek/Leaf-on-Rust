//! Перенос `net.minecraft.world.item.equipment.EquipmentAssets` —
//! ключи реестра `equipment_asset`.

use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;

/// `EquipmentAssets.ROOT_ID` — `minecraft:equipment_asset`.
pub const ROOT_ID_NAME: &str = "equipment_asset";

/// `EquipmentAssets.ROOT_ID` — `ResourceKey<Registry<EquipmentAsset>>`.
pub fn root_id() -> ResourceKey<crate::core::registry::Registry<crate::world::item::equipment::equipment_asset::EquipmentAsset>> {
    ResourceKey::create_registry_key(
        Identifier::parse("minecraft:equipment_asset").expect("equipment_asset registry"),
    )
}

/// `EquipmentAssets.get(name)` — `ResourceKey.create(ROOT_ID, id)`.
pub fn get(name: &str) -> ResourceKey<crate::world::item::equipment::equipment_asset::EquipmentAsset> {
    ResourceKey::create_with_registry_name(
        Identifier::parse("minecraft:equipment_asset").expect("equipment_asset registry"),
        Identifier::parse(&format!("minecraft:{name}")).expect("equipment asset id"),
    )
}
