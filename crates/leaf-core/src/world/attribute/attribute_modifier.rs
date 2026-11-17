//! `net.minecraft.world.attribute.modifier.AttributeModifier` —
//! модификаторы значений атрибутов + `OperationId`.

use super::attribute_value::AttributeValue;
use super::lerp_function::LerpFunction;
use crate::util::argb;
use crate::util::mth;

/// `AttributeModifier.OperationId` — 14 операций.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OperationId {
    /// `override`.
    Override,
    /// `alpha_blend`.
    AlphaBlend,
    /// `add`.
    Add,
    /// `subtract`.
    Subtract,
    /// `multiply`.
    Multiply,
    /// `blend_to_gray`.
    BlendToGray,
    /// `minimum`.
    Minimum,
    /// `maximum`.
    Maximum,
    /// `and`.
    And,
    /// `nand`.
    Nand,
    /// `or`.
    Or,
    /// `nor`.
    Nor,
    /// `xor`.
    Xor,
    /// `xnor`.
    Xnor,
}

impl OperationId {
    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            OperationId::Override => "override",
            OperationId::AlphaBlend => "alpha_blend",
            OperationId::Add => "add",
            OperationId::Subtract => "subtract",
            OperationId::Multiply => "multiply",
            OperationId::BlendToGray => "blend_to_gray",
            OperationId::Minimum => "minimum",
            OperationId::Maximum => "maximum",
            OperationId::And => "and",
            OperationId::Nand => "nand",
            OperationId::Or => "or",
            OperationId::Nor => "nor",
            OperationId::Xor => "xor",
            OperationId::Xnor => "xnor",
        }
    }

    /// `byName`-эквивалент.
    pub fn by_name(name: &str) -> Option<OperationId> {
        Some(match name {
            "override" => OperationId::Override,
            "alpha_blend" => OperationId::AlphaBlend,
            "add" => OperationId::Add,
            "subtract" => OperationId::Subtract,
            "multiply" => OperationId::Multiply,
            "blend_to_gray" => OperationId::BlendToGray,
            "minimum" => OperationId::Minimum,
            "maximum" => OperationId::Maximum,
            "and" => OperationId::And,
            "nand" => OperationId::Nand,
            "or" => OperationId::Or,
            "nor" => OperationId::Nor,
            "xor" => OperationId::Xor,
            "xnor" => OperationId::Xnor,
            _ => return None,
        })
    }

    /// Все значения в порядке объявления.
    pub const VALUES: [OperationId; 14] = [
        OperationId::Override,
        OperationId::AlphaBlend,
        OperationId::Add,
        OperationId::Subtract,
        OperationId::Multiply,
        OperationId::BlendToGray,
        OperationId::Minimum,
        OperationId::Maximum,
        OperationId::And,
        OperationId::Nand,
        OperationId::Or,
        OperationId::Nor,
        OperationId::Xor,
        OperationId::Xnor,
    ];
}

/// Конкретная операция модификатора (аналог подынтерфейсов Java).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributeModifierKind {
    // FloatModifier
    /// `FloatModifier.ALPHA_BLEND`.
    AlphaBlend,
    /// `FloatModifier.ADD`.
    Add,
    /// `FloatModifier.SUBTRACT`.
    Subtract,
    /// `FloatModifier.MULTIPLY`.
    Multiply,
    /// `FloatModifier.MINIMUM`.
    Minimum,
    /// `FloatModifier.MAXIMUM`.
    Maximum,
    // ColorModifier
    /// `ColorModifier.ALPHA_BLEND` (ARGB).
    ColorAlphaBlend,
    /// `ColorModifier.ADD` для RGB.
    AddRgb,
    /// `ColorModifier.SUBTRACT` для RGB.
    SubtractRgb,
    /// `ColorModifier.MULTIPLY` для RGB.
    MultiplyRgb,
    /// `ColorModifier.ADD` для ARGB.
    AddArgb,
    /// `ColorModifier.SUBTRACT` для ARGB.
    SubtractArgb,
    /// `ColorModifier.MULTIPLY` для ARGB.
    MultiplyArgb,
    /// `ColorModifier.BLEND_TO_GRAY`.
    BlendToGray,
    // BooleanModifier
    /// `BooleanModifier.AND`.
    And,
    /// `BooleanModifier.NAND`.
    Nand,
    /// `BooleanModifier.OR`.
    Or,
    /// `BooleanModifier.NOR`.
    Nor,
    /// `BooleanModifier.XOR`.
    Xor,
    /// `BooleanModifier.XNOR`.
    Xnor,
}

/// `AttributeModifier<Value, Argument>` — type-erased модификатор.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributeModifier {
    /// `OverrideModifier` — возвращает аргумент.
    Override,
    /// `FloatModifier`.
    Float(AttributeModifierKind),
    /// `ColorModifier`.
    Color(AttributeModifierKind),
    /// `BooleanModifier`.
    Boolean(AttributeModifierKind),
}

impl AttributeModifier {
    /// `operationId` — id операции для `checkAllowedModifier`.
    pub fn operation_id(&self) -> OperationId {
        use AttributeModifierKind as K;
        match self {
            AttributeModifier::Override => OperationId::Override,
            AttributeModifier::Float(K::AlphaBlend) | AttributeModifier::Color(K::ColorAlphaBlend) => OperationId::AlphaBlend,
            AttributeModifier::Float(K::Add) | AttributeModifier::Color(K::AddRgb) | AttributeModifier::Color(K::AddArgb) => OperationId::Add,
            AttributeModifier::Float(K::Subtract) | AttributeModifier::Color(K::SubtractRgb) | AttributeModifier::Color(K::SubtractArgb) => OperationId::Subtract,
            AttributeModifier::Float(K::Multiply) | AttributeModifier::Color(K::MultiplyRgb) | AttributeModifier::Color(K::MultiplyArgb) => OperationId::Multiply,
            AttributeModifier::Float(K::Minimum) => OperationId::Minimum,
            AttributeModifier::Float(K::Maximum) => OperationId::Maximum,
            AttributeModifier::Color(K::BlendToGray) => OperationId::BlendToGray,
            AttributeModifier::Boolean(K::And) => OperationId::And,
            AttributeModifier::Boolean(K::Nand) => OperationId::Nand,
            AttributeModifier::Boolean(K::Or) => OperationId::Or,
            AttributeModifier::Boolean(K::Nor) => OperationId::Nor,
            AttributeModifier::Boolean(K::Xor) => OperationId::Xor,
            AttributeModifier::Boolean(K::Xnor) => OperationId::Xnor,
            _ => OperationId::Override,
        }
    }

    /// `apply(subject, argument)` — применить модификатор.
    /// Аргумент — `AttributeValue` соответствующего типа.
    pub fn apply(&self, subject: &AttributeValue, argument: &AttributeValue) -> AttributeValue {
        use AttributeModifierKind as K;
        use AttributeValue as V;
        match self {
            AttributeModifier::Override => argument.clone(),
            AttributeModifier::Float(kind) => {
                let s = match subject {
                    V::Float(v) => *v,
                    _ => panic!("Float modifier on non-float subject"),
                };
                match kind {
                    K::AlphaBlend => {
                        let (value, alpha) = match argument {
                            V::FloatWithAlpha { value, alpha } => (*value, *alpha),
                            V::Float(v) => (*v, 1.0),
                            _ => panic!("Float alpha_blend expects FloatWithAlpha"),
                        };
                        V::Float(mth::lerp_f32(alpha, s, value))
                    }
                    K::Add => V::Float(s + as_f32(argument)),
                    K::Subtract => V::Float(s - as_f32(argument)),
                    K::Multiply => V::Float(s * as_f32(argument)),
                    K::Minimum => V::Float(s.min(as_f32(argument))),
                    K::Maximum => V::Float(s.max(as_f32(argument))),
                    _ => panic!("invalid float modifier kind"),
                }
            }
            AttributeModifier::Color(kind) => {
                let s = match subject {
                    V::Int(v) => *v,
                    _ => panic!("Color modifier on non-int subject"),
                };
                match kind {
                    K::ColorAlphaBlend => {
                        let arg = as_i32(argument);
                        V::Int(argb::alpha_blend(s, arg))
                    }
                    K::AddRgb => V::Int(argb::add_rgb(s, as_i32(argument))),
                    K::SubtractRgb => V::Int(argb::subtract_rgb(s, as_i32(argument))),
                    K::MultiplyRgb => V::Int(argb::multiply(s, as_i32(argument))),
                    K::AddArgb => V::Int(argb::add_rgb(s, as_i32(argument))),
                    K::SubtractArgb => V::Int(argb::subtract_rgb(s, as_i32(argument))),
                    K::MultiplyArgb => V::Int(argb::multiply(s, as_i32(argument))),
                    K::BlendToGray => {
                        let (brightness, factor) = match argument {
                            V::BlendToGray { brightness, factor } => (*brightness, *factor),
                            _ => panic!("blend_to_gray expects BlendToGray argument"),
                        };
                        let grey = argb::scale_rgb(argb::greyscale(s), brightness);
                        V::Int(argb::srgb_lerp(factor, s, grey))
                    }
                    _ => panic!("invalid color modifier kind"),
                }
            }
            AttributeModifier::Boolean(kind) => {
                let (s, a) = match (subject, argument) {
                    (V::Boolean(s), V::Boolean(a)) => (*s, *a),
                    _ => panic!("Boolean modifier on non-boolean values"),
                };
                V::Boolean(match kind {
                    K::And => a && s,
                    K::Nand => !a || !s,
                    K::Or => a || s,
                    K::Nor => !a && !s,
                    K::Xor => a ^ s,
                    K::Xnor => a == s,
                    _ => panic!("invalid boolean modifier kind"),
                })
            }
        }
    }

    /// `argumentKeyframeLerp` — lerp-функция аргумента.
    pub fn argument_keyframe_lerp(&self) -> LerpFunction {
        match self {
            AttributeModifier::Float(_) => LerpFunction::Float,
            AttributeModifier::Color(_) => LerpFunction::Color,
            AttributeModifier::Boolean(_) => LerpFunction::Constant,
            AttributeModifier::Override => LerpFunction::Constant,
        }
    }
}

fn as_f32(v: &AttributeValue) -> f32 {
    match v {
        AttributeValue::Float(f) => *f,
        AttributeValue::FloatWithAlpha { value, .. } => *value,
        _ => panic!("expected float attribute value"),
    }
}

fn as_i32(v: &AttributeValue) -> i32 {
    match v {
        AttributeValue::Int(i) => *i,
        _ => panic!("expected int attribute value"),
    }
}
