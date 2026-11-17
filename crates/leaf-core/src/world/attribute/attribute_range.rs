//! `AttributeRange<Value>` — диапазон допустимых значений атрибута.

/// `AttributeRange` — валидация/клэмп значения.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttributeRange {
    /// `any()` — без ограничений.
    Any,
    /// `ofFloat(min, max)`.
    Float { min: f32, max: f32 },
}

impl AttributeRange {
    /// `UNIT_FLOAT` — [0, 1].
    pub const UNIT_FLOAT: AttributeRange = AttributeRange::Float { min: 0.0, max: 1.0 };
    /// `NON_NEGATIVE_FLOAT` — [0, +∞).
    pub const NON_NEGATIVE_FLOAT: AttributeRange = AttributeRange::Float {
        min: 0.0,
        max: f32::INFINITY,
    };

    /// `any()`.
    pub fn any() -> AttributeRange {
        AttributeRange::Any
    }

    /// `ofFloat(min, max)`.
    pub fn of_float(min: f32, max: f32) -> AttributeRange {
        AttributeRange::Float { min, max }
    }

    /// `validate` — Ok(()) если значение в диапазоне.
    pub fn validate(&self, value: f32) -> Result<f32, String> {
        match self {
            AttributeRange::Any => Ok(value),
            AttributeRange::Float { min, max } => {
                if value >= *min && value <= *max {
                    Ok(value)
                } else {
                    Err(format!("{} is not in range [{}; {}]", value, min, max))
                }
            }
        }
    }

    /// `sanitize` — клэмп в диапазон.
    pub fn sanitize(&self, value: f32) -> f32 {
        match self {
            AttributeRange::Any => value,
            AttributeRange::Float { min, max } => value.clamp(*min, *max),
        }
    }
}
