//! Перенос `net.minecraft.world.entity.ai.attributes.RangedAttribute`.

use crate::entity::ai::attributes::attribute::Attribute;
use crate::resources::identifier::Identifier;
use crate::util::mth;

/// `RangedAttribute` — атрибут с min/max диапазоном.
#[derive(Clone, Debug)]
pub struct RangedAttribute {
    /// Базовый `Attribute`.
    pub base: Attribute,
    /// `minValue`.
    pub min_value: f64,
    /// `maxValue`.
    pub max_value: f64,
}

impl RangedAttribute {
    /// `new(descriptionId, defaultValue, minValue, maxValue)`.
    pub fn new(
        id: Identifier,
        description_id: impl Into<String>,
        default_value: f64,
        min_value: f64,
        max_value: f64,
    ) -> Self {
        assert!(min_value <= max_value, "Minimum value cannot be bigger than maximum value!");
        assert!(default_value >= min_value, "Default value cannot be lower than minimum value!");
        assert!(default_value <= max_value, "Default value cannot be bigger than maximum value!");
        Self {
            base: Attribute::new(id, description_id, default_value),
            min_value,
            max_value,
        }
    }

    /// `getMinValue()`.
    pub fn get_min_value(&self) -> f64 {
        self.min_value
    }

    /// `getMaxValue()`.
    pub fn get_max_value(&self) -> f64 {
        self.max_value
    }

    /// `sanitizeValue(value)` — clamp к диапазону.
    pub fn sanitize_value(&self, value: f64) -> f64 {
        if value.is_nan() {
            self.min_value
        } else {
            mth::clamp_f64(value, self.min_value, self.max_value)
        }
    }

    /// Делегаты к `base`.
    pub fn id(&self) -> &Identifier {
        &self.base.id
    }
    pub fn get_default_value(&self) -> f64 {
        self.base.default_value
    }
    pub fn is_client_syncable(&self) -> bool {
        self.base.syncable
    }
    pub fn get_description_id(&self) -> &str {
        &self.base.description_id
    }
}
