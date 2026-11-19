//! Перенос `net.minecraft.world.item.enchantment.EnchantmentEffectComponents` —
//! 28 типов компонентов эффектов зачарований (реестр
//! `ENCHANTMENT_EFFECT_COMPONENT_TYPE`).

use crate::resources::identifier::Identifier;
use crate::core::component::data_component_map::DataComponentMap;
use crate::world::item::enchantment::conditional_effect::ConditionalEffect;
use crate::world::item::enchantment::effects::enchantment_value_effect::EnchantmentValueEffect;
use crate::world::item::enchantment::effects::enchantment_entity_effect::EnchantmentEntityEffect;
use crate::world::item::enchantment::effects::enchantment_location_based_effect::EnchantmentLocationBasedEffect;
use crate::world::item::enchantment::effects::enchantment_attribute_effect::EnchantmentAttributeEffect;
use crate::world::item::enchantment::effects::targeted_conditional_effect::TargetedConditionalEffect;
use crate::world::item::enchantment::effects::damage_immunity::DamageImmunity;

/// Идентификаторы компонентов эффектов (registry names).
pub mod ids {
    use crate::resources::identifier::Identifier;
    macro_rules! id { ($n:literal) => { Identifier::minecraft($n) } }
    pub fn damage_protection() -> Identifier { id!("damage_protection") }
    pub fn damage_immunity() -> Identifier { id!("damage_immunity") }
    pub fn damage() -> Identifier { id!("damage") }
    pub fn smash_damage_per_fallen_block() -> Identifier { id!("smash_damage_per_fallen_block") }
    pub fn knockback() -> Identifier { id!("knockback") }
    pub fn armor_effectiveness() -> Identifier { id!("armor_effectiveness") }
    pub fn post_attack() -> Identifier { id!("post_attack") }
    pub fn post_piercing_attack() -> Identifier { id!("post_piercing_attack") }
    pub fn hit_block() -> Identifier { id!("hit_block") }
    pub fn item_damage() -> Identifier { id!("item_damage") }
    pub fn attributes() -> Identifier { id!("attributes") }
    pub fn equipment_drops() -> Identifier { id!("equipment_drops") }
    pub fn location_changed() -> Identifier { id!("location_changed") }
    pub fn tick() -> Identifier { id!("tick") }
    pub fn ammo_use() -> Identifier { id!("ammo_use") }
    pub fn projectile_piercing() -> Identifier { id!("projectile_piercing") }
    pub fn projectile_spawned() -> Identifier { id!("projectile_spawned") }
    pub fn projectile_spread() -> Identifier { id!("projectile_spread") }
    pub fn projectile_count() -> Identifier { id!("projectile_count") }
    pub fn trident_return_acceleration() -> Identifier { id!("trident_return_acceleration") }
    pub fn fishing_time_reduction() -> Identifier { id!("fishing_time_reduction") }
    pub fn fishing_luck_bonus() -> Identifier { id!("fishing_luck_bonus") }
    pub fn block_experience() -> Identifier { id!("block_experience") }
    pub fn mob_experience() -> Identifier { id!("mob_experience") }
    pub fn repair_with_xp() -> Identifier { id!("repair_with_xp") }
    pub fn crossbow_charge_time() -> Identifier { id!("crossbow_charge_time") }
    pub fn crossbow_charging_sounds() -> Identifier { id!("crossbow_charging_sounds") }
    pub fn trident_sound() -> Identifier { id!("trident_sound") }
    pub fn prevent_equipment_drop() -> Identifier { id!("prevent_equipment_drop") }
    pub fn prevent_armor_change() -> Identifier { id!("prevent_armor_change") }
    pub fn trident_spin_attack_strength() -> Identifier { id!("trident_spin_attack_strength") }
}

/// Типизированные алиасы хранимых значений компонентов.
pub type ValueEffects = Vec<ConditionalEffect<EnchantmentValueEffect>>;
pub type ImmunityEffects = Vec<ConditionalEffect<DamageImmunity>>;
pub type EntityEffects = Vec<ConditionalEffect<EnchantmentEntityEffect>>;
pub type TargetedEntityEffects = Vec<TargetedConditionalEffect<EnchantmentEntityEffect>>;
pub type TargetedValueEffects = Vec<TargetedConditionalEffect<EnchantmentValueEffect>>;
pub type LocationEffects = Vec<ConditionalEffect<EnchantmentLocationBasedEffect>>;
pub type AttributeEffects = Vec<EnchantmentAttributeEffect>;

/// `Enchantment.getEffects(component)` — `effects.get(component)`
/// с приведением к ожидаемому типу через `DataComponentMap`.
/// `DataComponentMap` хранит `Box<dyn Any>` — вызывающий код
/// запрашивает конкретный тип; несовпадение типа → None.
pub fn get_effects<'a, T: 'static>(
    effects: &'a DataComponentMap,
    component_id: &Identifier,
) -> Option<&'a T> {
    effects.get_by_id::<T>(component_id)
}
