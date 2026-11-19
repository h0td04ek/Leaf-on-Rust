//! Перенос `net.minecraft.world.item.enchantment.effects.EnchantmentAttributeEffect`
//! + `EnchantmentLocationBasedEffect` dispatch.

use crate::core::holder::Holder;
use crate::entity::ai::attributes::attribute::Attribute;
use crate::entity::ai::attributes::attribute_modifier::{
    AttributeModifier, AttributeModifierOperation,
};
use crate::resources::identifier::Identifier;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::item::enchantment::effects::enchanted_item_in_use::EnchantedItemInUse;
use crate::world::item::enchantment::level_based_value::LevelBasedValue;
use crate::world::phys::Vec3;
use crate::world::level::level_reader::LevelReader;
use std::collections::HashMap;
use std::sync::Arc;

/// `EnchantmentAttributeEffect` — модификатор атрибута при
/// изменении блока (location-based).
#[derive(Clone)]
pub struct EnchantmentAttributeEffect {
    /// `id`.
    pub id: Identifier,
    /// `attribute`.
    pub attribute: Holder<Attribute>,
    /// `amount`.
    pub amount: LevelBasedValue,
    /// `operation`.
    pub operation: AttributeModifierOperation,
}

/// `Holder<Attribute>` → registry-id атрибута.
pub fn attribute_holder_id(holder: &Holder<Attribute>) -> Identifier {
    match holder {
        Holder::Direct(a) => a.id.clone(),
        Holder::Key(k) => k.id().clone(),
        Holder::Reference(_) => Identifier::minecraft("generic.unknown"),
    }
}

impl EnchantmentAttributeEffect {
    /// `new`.
    pub fn new(
        id: Identifier,
        attribute: Holder<Attribute>,
        amount: LevelBasedValue,
        operation: AttributeModifierOperation,
    ) -> Self {
        Self { id, attribute, amount, operation }
    }

    /// `idForSlot(slot)` — `id.withSuffix("/" + slot)`.
    pub fn id_for_slot(&self, slot: &str) -> Identifier {
        Identifier::with_default_namespace(&format!("{}/{}", self.id, slot))
            .unwrap_or_else(|_| self.id.clone())
    }

    /// `getModifier(enchantmentLevel, slot)`.
    pub fn get_modifier(&self, enchantment_level: i32, slot: &str) -> AttributeModifier {
        AttributeModifier::new(
            self.id_for_slot(slot),
            self.amount.calculate(enchantment_level) as f64,
            self.operation,
        )
    }

    /// `makeAttributeMap(enchantmentLevel, slot)` →
    /// `HashMultimap<Holder<Attribute>, AttributeModifier>`
    /// (здесь: attribute-id → modifier-id список).
    pub fn make_attribute_map(
        &self,
        enchantment_level: i32,
        slot: Option<crate::entity::equipment_slot::EquipmentSlot>,
    ) -> HashMap<Identifier, Vec<Identifier>> {
        let slot_name = slot
            .map(|s| s.get_name().to_string())
            .unwrap_or_else(|| "mainhand".to_string());
        let modifier = self.get_modifier(enchantment_level, &slot_name);
        let mut map = HashMap::new();
        map.insert(
            attribute_holder_id(&self.attribute),
            vec![modifier.id.clone()],
        );
        map
    }

    /// `onChangedBlock` — transient modifiers на living entity.
    pub fn on_changed_block(
        &self,
        _level: &Arc<dyn LevelReader>,
        enchantment_level: i32,
        item: &EnchantedItemInUse,
        entity: &mut LivingEntity,
        _pos: Vec3,
        apply_transient_effects: bool,
    ) {
        if !apply_transient_effects {
            return;
        }
        let attr_id = attribute_holder_id(&self.attribute);
        let slot_name = item
            .in_slot
            .map(|s| s.get_name().to_string())
            .unwrap_or_else(|| "mainhand".to_string());
        let modifier = self.get_modifier(enchantment_level, &slot_name);
        entity
            .attributes
            .add_transient_attribute_modifiers(&attr_id, vec![modifier]);
    }

    /// `onDeactivated` — снять модификаторы.
    pub fn on_deactivated(
        &self,
        item: &EnchantedItemInUse,
        entity: &mut LivingEntity,
        _pos: Vec3,
        enchantment_level: i32,
    ) {
        let map = self.make_attribute_map(enchantment_level, item.in_slot);
        for (attr, ids) in map {
            entity.attributes.remove_attribute_modifiers(&attr, &ids);
        }
    }
}
