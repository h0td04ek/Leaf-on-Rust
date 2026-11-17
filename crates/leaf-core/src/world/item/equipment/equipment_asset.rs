//! Перенос `net.minecraft.world.item.equipment.EquipmentAsset` —
//! маркерный тип ключей equipment-ассетов.

/// `EquipmentAsset` — пустой класс-маркер; используется как
/// `ResourceKey<EquipmentAsset>` (реестр `equipment_asset`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EquipmentAsset;
