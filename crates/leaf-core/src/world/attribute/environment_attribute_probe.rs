//! `net.minecraft.world.attribute.EnvironmentAttributeProbe` —
//! запрос значения атрибута в точке/контексте.

use super::attribute_value::AttributeValue;
use super::environment_attribute::EnvironmentAttribute;
use super::environment_attribute_layer::EnvironmentAttributeLayer;
use crate::core::block_pos::BlockPos;

/// `EnvironmentAttributeProbe` — точка запроса атрибута:
/// позиция + слой контекста (климат/биом/погода).
#[derive(Clone, Debug, Default)]
pub struct EnvironmentAttributeProbe {
    /// `blockPos` — позиция запроса (если позиционная).
    pub block_pos: Option<BlockPos>,
    /// Слой значений, наложенный на дефолты реестра.
    pub layer: EnvironmentAttributeLayer,
}

impl EnvironmentAttributeProbe {
    /// `new`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `at(pos)` — позиционный пробник.
    pub fn at(block_pos: BlockPos) -> Self {
        Self { block_pos: Some(block_pos), layer: EnvironmentAttributeLayer::new() }
    }

    /// `with(layer)` — наложить слой.
    pub fn with_layer(mut self, layer: EnvironmentAttributeLayer) -> Self {
        self.layer = layer;
        self
    }

    /// `get(attribute)` — значение: layer override → default.
    pub fn get(&self, attribute: &EnvironmentAttribute) -> AttributeValue {
        self.layer
            .apply_modifier(attribute, attribute.default_value().clone())
    }

    /// `getOrDefault(attribute, default)`.
    pub fn get_or_default(&self, attribute: &EnvironmentAttribute, default: AttributeValue) -> AttributeValue {
        self.layer.apply_modifier(attribute, default)
    }
}

/// `EnvironmentAttributeReader` — read-only view над probe.
#[derive(Clone, Debug)]
pub struct EnvironmentAttributeReader<'a> {
    /// Исходный пробник.
    pub probe: &'a EnvironmentAttributeProbe,
}

impl<'a> EnvironmentAttributeReader<'a> {
    /// `new`.
    pub fn new(probe: &'a EnvironmentAttributeProbe) -> Self {
        Self { probe }
    }

    /// `get(attribute)`.
    pub fn get(&self, attribute: &EnvironmentAttribute) -> AttributeValue {
        self.probe.get(attribute)
    }
}
