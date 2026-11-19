//! Перенос `net.minecraft.world.item.enchantment.effects` —
//! эффекты зачарований (value/entity/location/attribute).

pub mod damage_immunity;
pub mod enchanted_item_in_use;
pub mod enchantment_attribute_effect;
pub mod enchantment_effect_components;
pub mod enchantment_entity_effect;
pub mod enchantment_location_based_effect;
pub mod enchantment_target;
pub mod enchantment_value_effect;
pub mod targeted_conditional_effect;

pub use damage_immunity::DamageImmunity;
pub use enchanted_item_in_use::EnchantedItemInUse;
pub use enchantment_attribute_effect::EnchantmentAttributeEffect;
pub use enchantment_entity_effect::{
    EnchantmentEntityEffect, EnchantmentEntityEffect as EntityEffect,
    ExplodeEffect, ExplosionInteraction, PositionSource, PositionSourceType,
    SpawnParticlesEffect, VelocitySource,
};
pub use enchantment_location_based_effect::EnchantmentLocationBasedEffect;
pub use enchantment_target::EnchantmentTarget;
pub use enchantment_value_effect::EnchantmentValueEffect;
pub use targeted_conditional_effect::TargetedConditionalEffect;
