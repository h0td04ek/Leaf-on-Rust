//! Перенос `net.minecraft.world.entity.ai.attributes.AttributeInstance`.

use super::attribute_modifier::{AttributeModifier, AttributeModifierOperation};
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;

/// Максимальное количество модификаторов на одну операцию.
pub const MAX_MODIFIERS_PER_OPERATION: usize = 1024;

/// `AttributeDirtyListener` — `onAttributeModified(instance, modifier, resource, op)`.
/// В Rust: контекст передаётся через closure-capture.
pub type AttributeDirtyCallback = Box<
    dyn FnMut(&mut AttributeInstance, Option<&AttributeModifier>, &Identifier, AttributeModifierOperation) + Send + Sync,
>;

/// Общий интерфейс `Attribute`/`RangedAttribute` (type-erasure для реестра).
pub trait AttributeLike: Send + Sync + std::fmt::Debug {
    /// `getId()` — registry id атрибута.
    fn id(&self) -> &Identifier;
    /// `getDefaultValue()`.
    fn default_value(&self) -> f64;
    /// `isClientSyncable()`.
    fn is_client_syncable(&self) -> bool;
    /// `sanitizeValue` — кламп к min/max для ranged.
    fn sanitize(&self, value: f64) -> f64;
    /// `getDescriptionId()`.
    fn description_id(&self) -> &str;
    /// Клон в единый тип для хранения.
    fn clone_box(&self) -> Box<dyn AttributeLike>;
}

impl AttributeLike for super::attribute::Attribute {
    fn id(&self) -> &Identifier { &self.id }
    fn default_value(&self) -> f64 { self.default_value }
    fn is_client_syncable(&self) -> bool { self.syncable }
    fn sanitize(&self, value: f64) -> f64 { self.sanitize_value(value) }
    fn description_id(&self) -> &str { &self.description_id }
    fn clone_box(&self) -> Box<dyn AttributeLike> { Box::new(self.clone()) }
}

impl AttributeLike for super::ranged_attribute::RangedAttribute {
    fn id(&self) -> &Identifier { self.base.id() }
    fn default_value(&self) -> f64 { self.base.default_value }
    fn is_client_syncable(&self) -> bool { self.base.syncable }
    fn sanitize(&self, value: f64) -> f64 { self.sanitize_value(value) }
    fn description_id(&self) -> &str { self.base.description_id() }
    fn clone_box(&self) -> Box<dyn AttributeLike> { Box::new(self.clone()) }
}

impl Clone for Box<dyn AttributeLike> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// `AttributeInstance` — значение атрибута с модификаторами.
pub struct AttributeInstance {
    /// `attribute` — type-erased атрибут.
    pub attribute: Box<dyn AttributeLike>,
    /// `baseValue`.
    pub base_value: f64,
    /// `cachedValue`.
    pub cached_value: f64,
    /// `defaultBaseValue`.
    pub default_base_value: f64,
    /// `modifiersByOperation` — operation id (0,1,2) -> modifiers.
    pub modifiers_by_operation: HashMap<i32, HashMap<Identifier, AttributeModifier>>,
    /// `dirty`.
    pub dirty: bool,
    /// Слушатель изменения (`AttributeMap` вешает сюда sync).
    pub dirty_listener: Option<AttributeDirtyCallback>,
}

impl std::fmt::Debug for AttributeInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttributeInstance")
            .field("id", &self.attribute.id())
            .field("base", &self.base_value)
            .field("cached", &self.cached_value)
            .finish()
    }
}

impl AttributeInstance {
    /// `new(attribute)`.
    pub fn new(attribute: Box<dyn AttributeLike>) -> Self {
        let default = attribute.default_value();
        Self {
            attribute,
            base_value: default,
            cached_value: default,
            default_base_value: default,
            modifiers_by_operation: HashMap::new(),
            dirty: true,
            dirty_listener: None,
        }
    }

    /// `setDirtyListener(listener)`.
    pub fn set_dirty_listener<F>(&mut self, listener: F)
    where
        F: FnMut(&mut AttributeInstance, Option<&AttributeModifier>, &Identifier, AttributeModifierOperation) + Send + Sync + 'static,
    {
        self.dirty_listener = Some(Box::new(listener));
    }

    /// `getAttribute()`.
    pub fn get_attribute(&self) -> &dyn AttributeLike {
        &*self.attribute
    }

    /// `getId()`.
    pub fn get_id(&self) -> &Identifier {
        self.attribute.id()
    }

    /// `getBaseValue()`.
    pub fn get_base_value(&self) -> f64 {
        self.base_value
    }

    /// `setBaseValue(newValue)` — sanitize + dirty.
    pub fn set_base_value(&mut self, new_value: f64) {
        self.base_value = self.attribute.sanitize(new_value);
        self.set_dirty(true);
    }

    /// `getDefaultBaseValue()`.
    pub fn get_default_base_value(&self) -> f64 {
        self.default_base_value
    }

    /// `resetToDefaultValue()`.
    pub fn reset_to_default_value(&mut self) {
        self.set_base_value(self.default_base_value);
    }

    /// `setDirty(dirty)` — вызов listener при dirty=true.
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
        if dirty {
            if let Some(mut listener) = self.dirty_listener.take() {
                listener(self, None, &Identifier::minecraft("generic.dummy"), AttributeModifierOperation::AddValue);
                self.dirty_listener = Some(listener);
            }
        }
    }

    /// `getModifier(id)` — первый найденный модификатор по id.
    pub fn get_modifier(&self, id: &Identifier) -> Option<&AttributeModifier> {
        self.modifiers_by_operation.values().find_map(|m| m.get(id))
    }

    /// `getModifiers(operation)` — все модификаторы операции.
    pub fn get_modifiers(&self, operation: AttributeModifierOperation) -> Vec<AttributeModifier> {
        self.modifiers_by_operation
            .get(&operation.id())
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    /// `getModifierCount()`.
    pub fn get_modifier_count(&self) -> usize {
        self.modifiers_by_operation.values().map(|m| m.len()).sum()
    }

    /// `hasModifier(modifier)`.
    pub fn has_modifier(&self, modifier: &AttributeModifier) -> bool {
        self.modifiers_by_operation.values().any(|m| m.contains_key(&modifier.id))
    }

    /// `getValue()` без &mut — возвращает последний
    /// закэшированный total (dirty-пересчёт требует &mut;
    /// используется read-only контекстами вроде pathfinding).
    pub fn get_value_ro(&self) -> f64 {
        self.cached_value
    }

    /// `getValue()` — пересчёт кэша если dirty.
    pub fn get_value(&mut self) -> f64 {
        self.update_cached_value();
        self.cached_value
    }

    /// `updateCachedValue()` — recompute через 3 операции.
    pub fn update_cached_value(&mut self) {
        if self.dirty {
            self.cached_value = self.sanitize_value(self.calculate_value());
            self.dirty = false;
        }
    }

    /// `calculateValue()` — base + add + multiply_base + multiply_total.
    fn calculate_value(&self) -> f64 {
        let mut v = self.base_value;
        for m in self.get_modifiers(AttributeModifierOperation::AddValue) {
            v += m.amount;
        }
        let base_mult = 1.0 + self
            .get_modifiers(AttributeModifierOperation::AddMultipliedBase)
            .iter()
            .map(|m| m.amount)
            .sum::<f64>();
        v *= base_mult;
        let mut result = v;
        for m in self.get_modifiers(AttributeModifierOperation::AddMultipliedTotal) {
            result *= 1.0 + m.amount;
        }
        result
    }

    /// `sanitizeValue` — делегат в `Attribute.sanitizeValue`.
    fn sanitize_value(&self, value: f64) -> f64 {
        self.attribute.sanitize(value)
    }

    /// `addTransientModifier` — то же что `addModifier` (id уже содержит uuid).
    pub fn add_transient_modifier(&mut self, modifier: AttributeModifier) {
        self.add_modifier(modifier);
    }

    /// `addOrUpdateTransientModifier`.
    pub fn add_or_update_transient_modifier(&mut self, modifier: AttributeModifier) {
        self.add_modifier(modifier);
    }

    /// `addPermanentModifier`.
    pub fn add_permanent_modifier(&mut self, modifier: AttributeModifier) {
        self.add_modifier(modifier);
    }

    /// `addModifier(modifier)` — лимит + notify.
    pub fn add_modifier(&mut self, modifier: AttributeModifier) {
        let op = modifier.operation;
        let op_key = op.id();
        let count = self.modifiers_by_operation.get(&op_key).map(|m| m.len()).unwrap_or(0);
        if count >= MAX_MODIFIERS_PER_OPERATION {
            return;
        }
        let map = self.modifiers_by_operation.entry(op_key).or_insert_with(HashMap::new);
        let id = modifier.id.clone();
        map.insert(id.clone(), modifier);
        self.set_dirty(true);
        // notify listener с конкретным modifier
        if let Some(mut listener) = self.dirty_listener.take() {
            if let Some(m) = self.get_modifier(&id).cloned() {
                listener(self, Some(&m), &id, op);
            }
            self.dirty_listener = Some(listener);
        }
    }

    /// `removeModifier(id)`.
    pub fn remove_modifier(&mut self, id: &Identifier) -> bool {
        for op in AttributeModifierOperation::values() {
            let op_key = op.id();
            let removed = self
                .modifiers_by_operation
                .get_mut(&op_key)
                .and_then(|m| m.remove(id));
            if let Some(modifier) = removed {
                self.set_dirty(true);
                if let Some(mut listener) = self.dirty_listener.take() {
                    listener(self, Some(&modifier), id, *op);
                    self.dirty_listener = Some(listener);
                }
                return true;
            }
        }
        false
    }

    /// `removeModifiers()` — очистить все.
    pub fn remove_modifiers(&mut self) {
        self.modifiers_by_operation.clear();
        self.set_dirty(true);
    }

    /// `save(tag)` — NBT (`Base`, `Modifiers`).
    pub fn save_nbt(&self, tag: &mut CompoundTag) {
        tag.put("Base", Tag::Double(self.base_value));
        let mut list: Vec<Tag> = Vec::new();
        for op_map in self.modifiers_by_operation.values() {
            for modifier in op_map.values() {
                let mut m = CompoundTag::new();
                m.put("Name", Tag::String(modifier.id.to_string()));
                m.put("Amount", Tag::Double(modifier.amount));
                m.put("Operation", Tag::String(modifier.operation.get_serialized_name().to_string()));
                list.push(Tag::Compound(m));
            }
        }
        tag.put("Modifiers", Tag::List(list));
    }

    /// `load(tag)`.
    pub fn load_nbt(&mut self, tag: &CompoundTag) {
        self.base_value = tag.get_double("Base").unwrap_or(self.default_base_value);
        self.modifiers_by_operation.clear();
        if let Some(list) = tag.get_list("Modifiers") {
            for item in list {
                if let Tag::Compound(m) = item {
                    let name = m.get_string("Name").unwrap_or_default().to_string();
                    let amount = m.get_double("Amount").unwrap_or(0.0);
                    let op = m.get_string("Operation").unwrap_or_default();
                    let operation = match op {
                        "add_value" => AttributeModifierOperation::AddValue,
                        "add_multiplied_base" => AttributeModifierOperation::AddMultipliedBase,
                        "add_multiplied_total" => AttributeModifierOperation::AddMultipliedTotal,
                        _ => continue,
                    };
                    if let Ok(id) = Identifier::parse(&name) {
                        self.add_modifier(AttributeModifier::new(id, amount, operation));
                    }
                }
            }
        }
        self.set_dirty(true);
    }
}

impl Clone for AttributeInstance {
    fn clone(&self) -> Self {
        Self {
            attribute: self.attribute.clone(),
            base_value: self.base_value,
            cached_value: self.cached_value,
            default_base_value: self.default_base_value,
            modifiers_by_operation: self.modifiers_by_operation.clone(),
            dirty: self.dirty,
            dirty_listener: None,
        }
    }
}

impl std::fmt::Display for AttributeInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AttributeInstance{{{}={}}}", self.get_id(), self.cached_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::attribute::Attribute;

    fn attr() -> Attribute {
        Attribute::new(
            Identifier::minecraft("generic.max_health"),
            "attribute.name.generic.max_health",
            20.0,
        )
    }

    #[test]
    fn test_base_and_modifiers() {
        let mut inst = AttributeInstance::new(Box::new(attr()));
        assert_eq!(inst.get_value(), 20.0);
        inst.set_base_value(30.0);
        assert_eq!(inst.get_value(), 30.0);
        inst.add_modifier(AttributeModifier::new(
            Identifier::minecraft("test.add"),
            5.0,
            AttributeModifierOperation::AddValue,
        ));
        assert_eq!(inst.get_value(), 35.0);
        // multiply base
        inst.add_modifier(AttributeModifier::new(
            Identifier::minecraft("test.mul"),
            0.5,
            AttributeModifierOperation::AddMultipliedBase,
        ));
        assert!((inst.get_value() - 52.5).abs() < 0.001);
        // multiply total
        inst.add_modifier(AttributeModifier::new(
            Identifier::minecraft("test.mul2"),
            1.0,
            AttributeModifierOperation::AddMultipliedTotal,
        ));
        assert!((inst.get_value() - 105.0).abs() < 0.001);
    }

    #[test]
    fn test_remove_modifier() {
        let mut inst = AttributeInstance::new(Box::new(attr()));
        let id = Identifier::minecraft("test.add");
        inst.add_modifier(AttributeModifier::new(id.clone(), 5.0, AttributeModifierOperation::AddValue));
        assert_eq!(inst.get_value(), 25.0);
        assert!(inst.remove_modifier(&id));
        assert_eq!(inst.get_value(), 20.0);
        assert!(!inst.remove_modifier(&id));
    }
}
