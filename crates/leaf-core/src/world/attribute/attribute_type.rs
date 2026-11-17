//! `net.minecraft.world.attribute.AttributeType` — тип значения атрибута
//! + библиотека модификаторов + lerp-функции.

use super::attribute_modifier::{AttributeModifier, OperationId};
use crate::world::attribute::attribute_modifier::AttributeModifierKind;
use super::attribute_value::AttributeValue;
use super::lerp_function::LerpFunction;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

/// Вид значения `AttributeType` (аналог Java-дженерика `Value`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributeTypeKind {
    /// `BOOLEAN`.
    Boolean,
    /// `TRI_STATE`.
    TriState,
    /// `FLOAT`.
    Float,
    /// `ANGLE_DEGREES`.
    AngleDegrees,
    /// `RGB_COLOR`.
    RgbColor,
    /// `ARGB_COLOR`.
    ArgbColor,
    /// `MOON_PHASE`.
    MoonPhase,
    /// `ACTIVITY`.
    Activity,
    /// `BED_RULE`.
    BedRule,
    /// `PARTICLE`.
    Particle,
    /// `AMBIENT_PARTICLES`.
    AmbientParticles,
    /// `BACKGROUND_MUSIC`.
    BackgroundMusic,
    /// `AMBIENT_SOUNDS`.
    AmbientSounds,
}

impl AttributeTypeKind {
    /// Соответствие `AttributeValue`-варианта.
    pub fn matches(&self, value: &AttributeValue) -> bool {
        use AttributeTypeKind as K;
        use AttributeValue as V;
        matches!(
            (self, value),
            (K::Boolean, V::Boolean(_))
                | (K::TriState, V::TriState(_))
                | (K::Float | K::AngleDegrees, V::Float(_))
                | (K::RgbColor | K::ArgbColor, V::Int(_))
                | (K::MoonPhase, V::MoonPhase(_))
                | (K::Activity, V::Activity(_))
                | (K::BedRule, V::BedRule(_))
                | (K::Particle, V::Particle(_))
                | (K::AmbientParticles, V::AmbientParticles(_))
                | (K::BackgroundMusic, V::BackgroundMusic(_))
                | (K::AmbientSounds, V::AmbientSounds(_))
        )
    }
}

/// `AttributeType<Value>` — тип значения + библиотека модификаторов.
///
/// Дженерик `Value` заменён на `AttributeTypeKind`; значения —
/// `AttributeValue`.
#[derive(Clone)]
pub struct AttributeType {
    /// Registry-ключ типа (`minecraft:float`).
    pub id: Identifier,
    /// Вид значения.
    pub kind: AttributeTypeKind,
    /// `modifierLibrary` — допустимые модификаторы типа.
    pub modifier_library: HashMap<OperationId, AttributeModifier>,
    /// `keyframeLerp`.
    pub keyframe_lerp: LerpFunction,
    /// `stateChangeLerp`.
    pub state_change_lerp: LerpFunction,
    /// `spatialLerp`.
    pub spatial_lerp: LerpFunction,
    /// `partialTickLerp`.
    pub partial_tick_lerp: LerpFunction,
}

impl AttributeType {
    /// `ofInterpolated(valueCodec, library, spatial, partialTick)`.
    fn of_interpolated(
        kind: AttributeTypeKind,
        library: HashMap<OperationId, AttributeModifier>,
        spatial: LerpFunction,
        partial_tick: LerpFunction,
    ) -> Self {
        Self {
            id: Identifier::minecraft(""),
            kind,
            modifier_library: library,
            keyframe_lerp: spatial.clone(),
            state_change_lerp: spatial,
            spatial_lerp: spatial,
            partial_tick_lerp: partial_tick,
        }
    }

    /// `ofNotInterpolated`.
    fn of_not_interpolated(kind: AttributeTypeKind, library: HashMap<OperationId, AttributeModifier>) -> Self {
        Self {
            id: Identifier::minecraft(""),
            kind,
            modifier_library: library,
            keyframe_lerp: LerpFunction::Step { threshold: 1.0 },
            state_change_lerp: LerpFunction::Step { threshold: 0.0 },
            spatial_lerp: LerpFunction::Step { threshold: 0.5 },
            partial_tick_lerp: LerpFunction::Step { threshold: 0.0 },
        }
    }

    /// `checkAllowedModifier`.
    pub fn check_allowed_modifier(&self, modifier: &AttributeModifier) -> Result<(), String> {
        if *modifier != AttributeModifier::Override && !self.modifier_library.contains_key(&modifier.operation_id()) {
            return Err(format!("Modifier {:?} is not valid for {}", modifier, self.id));
        }
        Ok(())
    }
}

/// Реестр `BuiltInRegistries.ATTRIBUTE_TYPE` (18 типов в vanilla-порядке).
pub fn attribute_types() -> &'static [Arc<AttributeType>] {
    static TYPES: OnceLock<Vec<Arc<AttributeType>>> = OnceLock::new();
    TYPES.get_or_init(|| {
        let mut v = Vec::new();
        let mut push = |name: &str, ty: AttributeType| {
            let mut ty = ty;
            ty.id = Identifier::minecraft(name);
            v.push(Arc::new(ty));
        };

        // boolean — BOOLEAN_LIBRARY, not interpolated
        push("boolean", AttributeType::of_not_interpolated(
            AttributeTypeKind::Boolean,
            boolean_library(),
        ));
        // tri_state
        push("tri_state", AttributeType::of_not_interpolated(
            AttributeTypeKind::TriState,
            HashMap::new(),
        ));
        // float — FLOAT_LIBRARY, interpolated ofFloat
        push("float", AttributeType::of_interpolated(
            AttributeTypeKind::Float,
            float_library(),
            LerpFunction::Float,
            LerpFunction::Float,
        ));
        // angle_degrees — spatial ofDegrees(90), partialTick ofFloat
        push("angle_degrees", AttributeType::of_interpolated(
            AttributeTypeKind::AngleDegrees,
            float_library(),
            LerpFunction::Degrees { max_delta: 90.0 },
            LerpFunction::Float,
        ));
        // rgb_color — RGB_COLOR_LIBRARY, ofColor lerp
        push("rgb_color", AttributeType::of_interpolated(
            AttributeTypeKind::RgbColor,
            rgb_color_library(),
            LerpFunction::Color,
            LerpFunction::Color,
        ));
        // argb_color — ARGB_COLOR_LIBRARY, ofColor lerp
        push("argb_color", AttributeType::of_interpolated(
            AttributeTypeKind::ArgbColor,
            argb_color_library(),
            LerpFunction::Color,
            LerpFunction::Color,
        ));
        // moon_phase / activity / bed_rule — not interpolated
        for (name, kind) in [
            ("moon_phase", AttributeTypeKind::MoonPhase),
            ("activity", AttributeTypeKind::Activity),
            ("bed_rule", AttributeTypeKind::BedRule),
            ("particle", AttributeTypeKind::Particle),
            ("ambient_particles", AttributeTypeKind::AmbientParticles),
            ("background_music", AttributeTypeKind::BackgroundMusic),
            ("ambient_sounds", AttributeTypeKind::AmbientSounds),
        ] {
            push(name, AttributeType::of_not_interpolated(kind, HashMap::new()));
        }
        v
    })
}

/// `AttributeType` по registry-id.
pub fn attribute_type_by_id(id: i32) -> Option<Arc<AttributeType>> {
    attribute_types().get(id as usize).cloned()
}

/// `AttributeType` по identifier.
pub fn attribute_type_by_identifier(id: &Identifier) -> Option<Arc<AttributeType>> {
    attribute_types().iter().find(|t| &t.id == id).cloned()
}

/// `AttributeTypes.BOOLEAN_LIBRARY`.
fn boolean_library() -> HashMap<OperationId, AttributeModifier> {
    use AttributeModifier::*;
    HashMap::from([
        (OperationId::And, Boolean(AttributeModifierKind::And)),
        (OperationId::Nand, Boolean(AttributeModifierKind::Nand)),
        (OperationId::Or, Boolean(AttributeModifierKind::Or)),
        (OperationId::Nor, Boolean(AttributeModifierKind::Nor)),
        (OperationId::Xor, Boolean(AttributeModifierKind::Xor)),
        (OperationId::Xnor, Boolean(AttributeModifierKind::Xnor)),
    ])
}

/// `AttributeTypes.FLOAT_LIBRARY`.
fn float_library() -> HashMap<OperationId, AttributeModifier> {
    use AttributeModifier::*;
    HashMap::from([
        (OperationId::AlphaBlend, Float(AttributeModifierKind::AlphaBlend)),
        (OperationId::Add, Float(AttributeModifierKind::Add)),
        (OperationId::Subtract, Float(AttributeModifierKind::Subtract)),
        (OperationId::Multiply, Float(AttributeModifierKind::Multiply)),
        (OperationId::Minimum, Float(AttributeModifierKind::Minimum)),
        (OperationId::Maximum, Float(AttributeModifierKind::Maximum)),
    ])
}

/// `AttributeTypes.RGB_COLOR_LIBRARY`.
fn rgb_color_library() -> HashMap<OperationId, AttributeModifier> {
    use AttributeModifier::*;
    HashMap::from([
        (OperationId::AlphaBlend, Color(AttributeModifierKind::AlphaBlend)),
        (OperationId::Add, Color(AttributeModifierKind::AddRgb)),
        (OperationId::Subtract, Color(AttributeModifierKind::SubtractRgb)),
        (OperationId::Multiply, Color(AttributeModifierKind::MultiplyRgb)),
        (OperationId::BlendToGray, Color(AttributeModifierKind::BlendToGray)),
    ])
}

/// `AttributeTypes.ARGB_COLOR_LIBRARY`.
fn argb_color_library() -> HashMap<OperationId, AttributeModifier> {
    use AttributeModifier::*;
    HashMap::from([
        (OperationId::AlphaBlend, Color(AttributeModifierKind::AlphaBlend)),
        (OperationId::Add, Color(AttributeModifierKind::AddArgb)),
        (OperationId::Subtract, Color(AttributeModifierKind::SubtractArgb)),
        (OperationId::Multiply, Color(AttributeModifierKind::MultiplyArgb)),
        (OperationId::BlendToGray, Color(AttributeModifierKind::BlendToGray)),
    ])
}
