//! `net.minecraft.world.attribute.LerpFunction` — интерполяция
//! значений атрибутов.

use super::attribute_value::AttributeValue;
use crate::util::argb;
use crate::util::mth;

/// `LerpFunction<T>` — интерполяция между start/end по delta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LerpFunction {
    /// `ofFloat` — `Mth.lerp`.
    Float,
    /// `ofDegrees(maxDelta)` — угловая интерполяция.
    Degrees { max_delta: f32 },
    /// `ofConstant` — всегда start.
    Constant,
    /// `ofStep(threshold)` — step-функция.
    Step { threshold: f32 },
    /// `ofColor` — `ARGB.srgbLerp`.
    Color,
}

impl LerpFunction {
    /// `apply(delta, start, end)` — типизированная интерполяция
    /// для `AttributeValue::Float`/`Int` (color).
    pub fn apply(&self, delta: f32, start: &AttributeValue, end: &AttributeValue) -> AttributeValue {
        use AttributeValue::*;
        match self {
            LerpFunction::Float => {
                let s = float_of(start);
                let e = float_of(end);
                Float(mth::lerp_f32(delta, s, e))
            }
            LerpFunction::Degrees { max_delta } => {
                let s = float_of(start);
                let e = float_of(end);
                let diff = mth::wrap_degrees_f32(e - s);
                Float(if diff.abs() >= *max_delta { e } else { s + delta * diff })
            }
            LerpFunction::Constant => start.clone(),
            LerpFunction::Step { threshold } => {
                if delta >= *threshold { end.clone() } else { start.clone() }
            }
            LerpFunction::Color => {
                let s = int_of(start);
                let e = int_of(end);
                Int(argb::srgb_lerp(delta, s, e))
            }
        }
    }
}

fn float_of(v: &AttributeValue) -> f32 {
    match v {
        AttributeValue::Float(f) => *f,
        _ => panic!("expected float attribute value"),
    }
}

fn int_of(v: &AttributeValue) -> i32 {
    match v {
        AttributeValue::Int(i) => *i,
        _ => panic!("expected int attribute value"),
    }
}
