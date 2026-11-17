//! Перенос `net.minecraft.util.EasingType`.
//! DFU-кодеки и `SIMPLE_REGISTRY` (LateBoundIdMapper) отложены —
//! реестр нужен только для кодирования; логика apply портирована полностью.

use crate::util::ease;
/// `EasingType` — тип easing-функции для анимаций/сплайнов.
#[derive(Clone, Debug, PartialEq)]
pub enum EasingType {
    /// Встроенные именованные easing'и.
    Simple(SimpleEasing),
    /// `EasingType.CubicBezier` — кубическая кривая Безье.
    CubicBezier(CubicBezier),
}

/// `EasingType.registerSimple`-аналог: 30 встроенных easing'ов.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SimpleEasing {
    Constant,
    Linear,
    InBack,
    InBounce,
    InCirc,
    InCubic,
    InElastic,
    InExpo,
    InQuad,
    InQuart,
    InQuint,
    InSine,
    InOutBack,
    InOutBounce,
    InOutCirc,
    InOutCubic,
    InOutElastic,
    InOutExpo,
    InOutQuad,
    InOutQuart,
    InOutQuint,
    InOutSine,
    OutBack,
    OutBounce,
    OutCirc,
    OutCubic,
    OutElastic,
    OutExpo,
    OutQuad,
    OutQuart,
    OutQuint,
    OutSine,
}

impl SimpleEasing {
    /// `id` — имя в SIMPLE_REGISTRY.
    pub fn id(self) -> &'static str {
        match self {
            Self::Constant => "constant",
            Self::Linear => "linear",
            Self::InBack => "in_back",
            Self::InBounce => "in_bounce",
            Self::InCirc => "in_circ",
            Self::InCubic => "in_cubic",
            Self::InElastic => "in_elastic",
            Self::InExpo => "in_expo",
            Self::InQuad => "in_quad",
            Self::InQuart => "in_quart",
            Self::InQuint => "in_quint",
            Self::InSine => "in_sine",
            Self::InOutBack => "in_out_back",
            Self::InOutBounce => "in_out_bounce",
            Self::InOutCirc => "in_out_circ",
            Self::InOutCubic => "in_out_cubic",
            Self::InOutElastic => "in_out_elastic",
            Self::InOutExpo => "in_out_expo",
            Self::InOutQuad => "in_out_quad",
            Self::InOutQuart => "in_out_quart",
            Self::InOutQuint => "in_out_quint",
            Self::InOutSine => "in_out_sine",
            Self::OutBack => "out_back",
            Self::OutBounce => "out_bounce",
            Self::OutCirc => "out_circ",
            Self::OutCubic => "out_cubic",
            Self::OutElastic => "out_elastic",
            Self::OutExpo => "out_expo",
            Self::OutQuad => "out_quad",
            Self::OutQuart => "out_quart",
            Self::OutQuint => "out_quint",
            Self::OutSine => "out_sine",
        }
    }

    /// Поиск по id (аналог SIMPLE_REGISTRY.get).
    pub fn by_id(id: &str) -> Option<Self> {
        Some(match id {
            "constant" => Self::Constant,
            "linear" => Self::Linear,
            "in_back" => Self::InBack,
            "in_bounce" => Self::InBounce,
            "in_circ" => Self::InCirc,
            "in_cubic" => Self::InCubic,
            "in_elastic" => Self::InElastic,
            "in_expo" => Self::InExpo,
            "in_quad" => Self::InQuad,
            "in_quart" => Self::InQuart,
            "in_quint" => Self::InQuint,
            "in_sine" => Self::InSine,
            "in_out_back" => Self::InOutBack,
            "in_out_bounce" => Self::InOutBounce,
            "in_out_circ" => Self::InOutCirc,
            "in_out_cubic" => Self::InOutCubic,
            "in_out_elastic" => Self::InOutElastic,
            "in_out_expo" => Self::InOutExpo,
            "in_out_quad" => Self::InOutQuad,
            "in_out_quart" => Self::InOutQuart,
            "in_out_quint" => Self::InOutQuint,
            "in_out_sine" => Self::InOutSine,
            "out_back" => Self::OutBack,
            "out_bounce" => Self::OutBounce,
            "out_circ" => Self::OutCirc,
            "out_cubic" => Self::OutCubic,
            "out_elastic" => Self::OutElastic,
            "out_expo" => Self::OutExpo,
            "out_quad" => Self::OutQuad,
            "out_quart" => Self::OutQuart,
            "out_quint" => Self::OutQuint,
            "out_sine" => Self::OutSine,
            _ => return None,
        })
    }
}

impl EasingType {
    pub fn apply(&self, x: f32) -> f32 {
        match self {
            Self::Simple(e) => e.apply(x),
            Self::CubicBezier(c) => c.apply(x),
        }
    }

    /// `cubicBezier(x1, y1, x2, y2)`.
    pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self::CubicBezier(CubicBezier::new(CubicBezierControls::new(x1, y1, x2, y2)))
    }

    /// `symmetricCubicBezier(x1, y1)`.
    pub fn symmetric_cubic_bezier(x1: f32, y1: f32) -> Self {
        Self::cubic_bezier(x1, y1, 1.0 - x1, 1.0 - y1)
    }
}

impl SimpleEasing {
    pub fn apply(self, x: f32) -> f32 {
        match self {
            Self::Constant => 0.0,
            Self::Linear => x,
            Self::InBack => ease::in_back(x),
            Self::InBounce => ease::in_bounce(x),
            Self::InCirc => ease::in_circ(x),
            Self::InCubic => ease::in_cubic(x),
            Self::InElastic => ease::in_elastic(x),
            Self::InExpo => ease::in_expo(x),
            Self::InQuad => ease::in_quad(x),
            Self::InQuart => ease::in_quart(x),
            Self::InQuint => ease::in_quint(x),
            Self::InSine => ease::in_sine(x),
            Self::InOutBack => ease::in_out_back(x),
            Self::InOutBounce => ease::in_out_bounce(x),
            Self::InOutCirc => ease::in_out_circ(x),
            Self::InOutCubic => ease::in_out_cubic(x),
            Self::InOutElastic => ease::in_out_elastic(x),
            Self::InOutExpo => ease::in_out_expo(x),
            Self::InOutQuad => ease::in_out_quad(x),
            Self::InOutQuart => ease::in_out_quart(x),
            Self::InOutQuint => ease::in_out_quint(x),
            Self::InOutSine => ease::in_out_sine(x),
            Self::OutBack => ease::out_back(x),
            Self::OutBounce => ease::out_bounce(x),
            Self::OutCirc => ease::out_circ(x),
            Self::OutCubic => ease::out_cubic(x),
            Self::OutElastic => ease::out_elastic(x),
            Self::OutExpo => ease::out_expo(x),
            Self::OutQuad => ease::out_quad(x),
            Self::OutQuart => ease::out_quart(x),
            Self::OutQuint => ease::out_quint(x),
            Self::OutSine => ease::out_sine(x),
        }
    }
}

/// Константы `EasingType.CONSTANT`/`LINEAR`.
pub fn constant() -> EasingType {
    EasingType::Simple(SimpleEasing::Constant)
}
pub fn linear() -> EasingType {
    EasingType::Simple(SimpleEasing::Linear)
}

/// `EasingType.CubicBezier` — кубический сплайн Безье для easing.
#[derive(Clone)]
pub struct CubicBezier {
    controls: CubicBezierControls,
    x_curve: CubicCurve,
    y_curve: CubicCurve,
}

const NEWTON_RAPHSON_ITERATIONS: usize = 4;

/// `CubicBezierControls(x1, y1, x2, y2)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezierControls {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl CubicBezierControls {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

#[derive(Clone, Copy)]
struct CubicCurve {
    a: f32,
    b: f32,
    c: f32,
}

impl CubicCurve {
    fn sample(&self, t: f32) -> f32 {
        ((self.a * t + self.b) * t + self.c) * t
    }
    fn sample_gradient(&self, t: f32) -> f32 {
        (3.0 * self.a * t + 2.0 * self.b) * t + self.c
    }
}

impl CubicBezier {
    pub fn new(controls: CubicBezierControls) -> Self {
        Self {
            controls,
            x_curve: Self::curve_from_controls(controls.x1, controls.x2),
            y_curve: Self::curve_from_controls(controls.y1, controls.y2),
        }
    }

    fn curve_from_controls(v1: f32, v2: f32) -> CubicCurve {
        CubicCurve {
            a: 3.0 * v1 - 3.0 * v2 + 1.0,
            b: -6.0 * v1 + 3.0 * v2,
            c: 3.0 * v1,
        }
    }

    /// `getControls()`.
    pub fn controls(&self) -> CubicBezierControls {
        self.controls
    }
}

impl CubicBezier {
    /// `CubicBezier.apply(x)` — Ньютон-Рафсон для x, затем y.
    pub fn apply(&self, x: f32) -> f32 {
        self.apply_curve(x)
    }

    fn apply_curve(&self, x: f32) -> f32 {
        let mut f = x;
        for _ in 0..NEWTON_RAPHSON_ITERATIONS {
            let f1 = self.x_curve.sample_gradient(f);
            if f1 < 1.0e-5 {
                break;
            }
            let f2 = self.x_curve.sample(f) - x;
            f -= f2 / f1;
        }
        self.y_curve.sample(f)
    }
}

impl std::fmt::Debug for CubicBezier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CubicBezier({}, {}, {}, {})",
            self.controls.x1, self.controls.y1, self.controls.x2, self.controls.y2
        )
    }
}

impl PartialEq for CubicBezier {
    fn eq(&self, other: &Self) -> bool {
        self.controls == other.controls
    }
}

impl Eq for CubicBezier {}

impl std::fmt::Display for CubicBezier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CubicBezier({}, {}, {}, {})", self.controls.x1, self.controls.y1, self.controls.x2, self.controls.y2)
    }
}

impl std::fmt::Display for EasingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple(e) => write!(f, "{}", e.id()),
            Self::CubicBezier(c) => write!(f, "{}", c),
        }
    }
}
