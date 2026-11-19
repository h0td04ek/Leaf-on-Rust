//! Перенос `net.minecraft.world.item.enchantment.effects.EnchantmentLocationBasedEffect`
//! + `DamageImmunity`.

use crate::core::holder::Holder;
use crate::world::damagesource::damage_type::DamageType;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::item::enchantment::effects::enchantment_attribute_effect::EnchantmentAttributeEffect;
use crate::world::item::enchantment::effects::enchantment_entity_effect::EnchantmentEntityEffect;
use crate::world::item::enchantment::effects::enchanted_item_in_use::EnchantedItemInUse;
use crate::world::phys::Vec3;
use crate::world::level::level_reader::LevelReader;
use std::sync::Arc;

/// `DamageImmunity` — unit-запись (иммунитет к типу урона).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamageImmunity;

/// `EnchantmentLocationBasedEffect` — эффекты, срабатывающие при
/// изменении блока/локации. Реализации: `AllOf.LocationBasedEffects`,
/// `EnchantmentAttributeEffect`, все `EnchantmentEntityEffect`.
#[derive(Clone)]
pub enum EnchantmentLocationBasedEffect {
    /// `AllOf.LocationBasedEffects`.
    AllOf(Vec<EnchantmentLocationBasedEffect>),
    /// `EnchantmentAttributeEffect`.
    Attribute(EnchantmentAttributeEffect),
    /// `EnchantmentEntityEffect` (наследует location-интерфейс).
    Entity(Box<EnchantmentEntityEffect>),
}

impl EnchantmentLocationBasedEffect {
    /// `onChangedBlock(level, enchantmentLevel, item, entity, pos, applyTransientEffects)`.
    pub fn on_changed_block(
        &self,
        level: &Arc<dyn LevelReader>,
        enchantment_level: i32,
        item: &mut EnchantedItemInUse,
        entity: &mut LivingEntity,
        pos: Vec3,
        apply_transient_effects: bool,
    ) {
        match self {
            Self::AllOf(effects) => {
                for e in effects {
                    e.on_changed_block(
                        level,
                        enchantment_level,
                        item,
                        entity,
                        pos,
                        apply_transient_effects,
                    );
                }
            }
            Self::Attribute(attr) => {
                attr.on_changed_block(
                    level,
                    enchantment_level,
                    item,
                    entity,
                    pos,
                    apply_transient_effects,
                );
            }
            Self::Entity(e) => {
                e.on_changed_block(
                    level,
                    enchantment_level,
                    item,
                    entity,
                    pos,
                    apply_transient_effects,
                );
            }
        }
    }

    /// `onDeactivated(item, entity, pos, enchantmentLevel)`.
    pub fn on_deactivated(
        &self,
        item: &EnchantedItemInUse,
        entity: &mut LivingEntity,
        pos: Vec3,
        enchantment_level: i32,
    ) {
        match self {
            Self::AllOf(effects) => {
                for e in effects {
                    e.on_deactivated(item, entity, pos, enchantment_level);
                }
            }
            Self::Attribute(attr) => {
                attr.on_deactivated(item, entity, pos, enchantment_level);
            }
            Self::Entity(_) => {}
        }
    }
}

/// `DamageType` holder-helper для `DamageImmunity` списков.
pub type DamageImmunityList = Vec<Holder<DamageType>>;
