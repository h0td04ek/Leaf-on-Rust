//! `net.minecraft.world.attribute.EnvironmentAttributeLayer` —
//! слой значений атрибутов (subject→argument маппинг).

use super::attribute_modifier::AttributeModifier;
use super::attribute_value::AttributeValue;
use super::environment_attribute::EnvironmentAttribute;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::Arc;

/// `EnvironmentAttributeLayer` — один слой карты атрибутов.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnvironmentAttributeLayer {
    /// `values` — id атрибута → (argument, modifier).
    pub values: HashMap<Identifier, (AttributeValue, AttributeModifier)>,
}

impl EnvironmentAttributeLayer {
    /// `new`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// `put(attribute, value)` — установить через override.
    pub fn put(&mut self, attribute: &EnvironmentAttribute, value: AttributeValue) {
        self.values.insert(attribute.id.clone(), (value, AttributeModifier::Override));
    }

    /// `modify(attribute, modifier, argument)`.
    pub fn modify(
        &mut self,
        attribute: &EnvironmentAttribute,
        modifier: AttributeModifier,
        argument: AttributeValue,
    ) -> Result<(), String> {
        attribute.check_allowed_modifier(&modifier)?;
        self.values.insert(attribute.id.clone(), (argument, modifier));
        Ok(())
    }

    /// `get(attribute)` — (argument, modifier) если есть.
    pub fn get(&self, attribute: &EnvironmentAttribute) -> Option<&(AttributeValue, AttributeModifier)> {
        self.values.get(&attribute.id)
    }

    /// `applyModifier(attribute, value)` — применить слой к значению.
    pub fn apply_modifier(&self, attribute: &EnvironmentAttribute, value: AttributeValue) -> AttributeValue {
        match self.values.get(&attribute.id) {
            Some((argument, modifier)) => modifier.apply(&value, argument),
            None => value,
        }
    }
}

/// `EnvironmentAttributeSystem` — система слоёв (context → probe).
/// Порядок применения: слои применяются от большего priority к меньшему.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentAttributeSystem {
    /// Слои (Vec, применяются в порядке вставки).
    pub layers: Vec<EnvironmentAttributeLayer>,
}

impl EnvironmentAttributeSystem {
    /// `new`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавить слой.
    pub fn push(&mut self, layer: EnvironmentAttributeLayer) {
        self.layers.push(layer);
    }

    /// `probe`-стиль: применить все слои к значению атрибута.
    pub fn apply(&self, attribute: &EnvironmentAttribute, mut value: AttributeValue) -> AttributeValue {
        for layer in self.layers.iter().rev() {
            value = layer.apply_modifier(attribute, value);
        }
        value
    }
}

/// Утилита: собрать map из списка `(attribute, value)` пар.
pub fn layer_from_pairs(pairs: Vec<(Arc<EnvironmentAttribute>, AttributeValue)>) -> EnvironmentAttributeLayer {
    let mut layer = EnvironmentAttributeLayer::new();
    for (attr, value) in pairs {
        layer.put(&attr, value);
    }
    layer
}
