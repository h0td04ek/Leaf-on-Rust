//! Полный перенос `org.joml.Math` — расширенная математика поверх f32/f64.
//!
//! Имена методов в snake_case; перегрузки по типу объединены (f32-версия
//! возвращает f32, f64 — f64; для функций, где JOML использует `double`
//! внутри f32-версии, сохранено то же поведение).

/// `org.joml.Math`.
pub struct Math;

impl Math {
    // ---- roots / powers -------------------------------------------------

    /// `invsqrt` — 1/sqrt(r) (JOML Math).
    pub fn invsqrt(r: f32) -> f32 {
        1.0 / r.sqrt()
    }

    /// `invsqrt` double.
    pub fn invsqrt_d(r: f64) -> f64 {
        1.0 / r.sqrt()
    }

    /// `safeAcos` — acos с зажимом в [-1, 1] (JOML Math).
    pub fn safe_acos(v: f32) -> f32 {
        v.clamp(-1.0, 1.0).acos()
    }

    /// `safeAcos` double.
    pub fn safe_acos_d(v: f64) -> f64 {
        v.clamp(-1.0, 1.0).acos()
    }

    /// `sqrt`.
    pub fn sqrt(x: f32) -> f32 {
        x.sqrt()
    }
    /// `sqrt` (double).
    pub fn sqrt_d(x: f64) -> f64 {
        x.sqrt()
    }

    /// `cbrt` (есть в JOML 1.10).
    pub fn cbrt(x: f32) -> f32 {
        x.cbrt()
    }
    /// `cbrt` (double).
    pub fn cbrt_d(x: f64) -> f64 {
        x.cbrt()
    }

    /// `pow`.
    pub fn pow(x: f32, y: f32) -> f32 {
        x.powf(y)
    }
    /// `pow` (double).
    pub fn pow_d(x: f64, y: f64) -> f64 {
        x.powf(y)
    }

    /// `exp`.
    pub fn exp(x: f32) -> f32 {
        x.exp()
    }
    /// `exp` (double).
    pub fn exp_d(x: f64) -> f64 {
        x.exp()
    }

    /// `exp2`.
    pub fn exp2(x: f32) -> f32 {
        x.exp2()
    }
    /// `exp2` (double).
    pub fn exp2_d(x: f64) -> f64 {
        x.exp2()
    }

    /// `expm1`.
    pub fn expm1(x: f32) -> f32 {
        x.exp_m1()
    }
    /// `expm1` (double).
    pub fn expm1_d(x: f64) -> f64 {
        x.exp_m1()
    }

    /// `log` (natural).
    pub fn ln(x: f32) -> f32 {
        x.ln()
    }
    /// `log` (double).
    pub fn ln_d(x: f64) -> f64 {
        x.ln()
    }

    /// `log10`.
    pub fn log10(x: f32) -> f32 {
        x.log10()
    }
    /// `log10` (double).
    pub fn log10_d(x: f64) -> f64 {
        x.log10()
    }

    /// `log2`.
    pub fn log2(x: f32) -> f32 {
        x.log2()
    }
    /// `log2` (double).
    pub fn log2_d(x: f64) -> f64 {
        x.log2()
    }

    /// `log1p`.
    pub fn ln_1p(x: f32) -> f32 {
        x.ln_1p()
    }
    /// `log1p` (double).
    pub fn ln_1p_d(x: f64) -> f64 {
        x.ln_1p()
    }

    // ---- trig -----------------------------------------------------------

    /// `sin`.
    pub fn sin(x: f32) -> f32 {
        x.sin()
    }
    /// `sin` (double).
    pub fn sin_d(x: f64) -> f64 {
        x.sin()
    }

    /// `cos`.
    pub fn cos(x: f32) -> f32 {
        x.cos()
    }
    /// `cos` (double).
    pub fn cos_d(x: f64) -> f64 {
        x.cos()
    }

    /// `tan`.
    pub fn tan(x: f32) -> f32 {
        x.tan()
    }
    /// `tan` (double).
    pub fn tan_d(x: f64) -> f64 {
        x.tan()
    }

    /// `asin`.
    pub fn asin(x: f32) -> f32 {
        x.asin()
    }
    /// `asin` (double).
    pub fn asin_d(x: f64) -> f64 {
        x.asin()
    }

    /// `acos`.
    pub fn acos(x: f32) -> f32 {
        x.acos()
    }
    /// `acos` (double).
    pub fn acos_d(x: f64) -> f64 {
        x.acos()
    }

    /// `atan`.
    pub fn atan(x: f32) -> f32 {
        x.atan()
    }
    /// `atan` (double).
    pub fn atan_d(x: f64) -> f64 {
        x.atan()
    }

    /// `atan2(y, x)`.
    pub fn atan2(y: f32, x: f32) -> f32 {
        y.atan2(x)
    }
    /// `atan2` (double).
    pub fn atan2_d(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }

    /// `sinh`.
    pub fn sinh(x: f32) -> f32 {
        x.sinh()
    }
    /// `sinh` (double).
    pub fn sinh_d(x: f64) -> f64 {
        x.sinh()
    }

    /// `cosh`.
    pub fn cosh(x: f32) -> f32 {
        x.cosh()
    }
    /// `cosh` (double).
    pub fn cosh_d(x: f64) -> f64 {
        x.cosh()
    }

    /// `tanh`.
    pub fn tanh(x: f32) -> f32 {
        x.tanh()
    }
    /// `tanh` (double).
    pub fn tanh_d(x: f64) -> f64 {
        x.tanh()
    }

    /// `asinh`.
    pub fn asinh(x: f32) -> f32 {
        x.asinh()
    }
    /// `asinh` (double).
    pub fn asinh_d(x: f64) -> f64 {
        x.asinh()
    }

    /// `acosh`.
    pub fn acosh(x: f32) -> f32 {
        x.acosh()
    }
    /// `acosh` (double).
    pub fn acosh_d(x: f64) -> f64 {
        x.acosh()
    }

    /// `atanh`.
    pub fn atanh(x: f32) -> f32 {
        x.atanh()
    }
    /// `atanh` (double).
    pub fn atanh_d(x: f64) -> f64 {
        x.atanh()
    }

    /// `toRadians`.
    pub fn to_radians(x: f32) -> f32 {
        x.to_radians()
    }
    /// `toRadians` (double).
    pub fn to_radians_d(x: f64) -> f64 {
        x.to_radians()
    }

    /// `toDegrees`.
    pub fn to_degrees(x: f32) -> f32 {
        x.to_degrees()
    }
    /// `toDegrees` (double).
    pub fn to_degrees_d(x: f64) -> f64 {
        x.to_degrees()
    }

    // ---- rounding -------------------------------------------------------

    /// `abs`.
    pub fn abs(x: f32) -> f32 {
        x.abs()
    }
    /// `abs` (double).
    pub fn abs_d(x: f64) -> f64 {
        x.abs()
    }

    /// `abs` (int).
    pub fn abs_i(x: i32) -> i32 {
        x.abs()
    }
    /// `abs` (long).
    pub fn abs_l(x: i64) -> i64 {
        x.abs()
    }

    /// `signum`.
    pub fn signum(x: f32) -> f32 {
        x.signum()
    }
    /// `signum` (double).
    pub fn signum_d(x: f64) -> f64 {
        x.signum()
    }

    /// `floor`.
    pub fn floor(x: f32) -> f32 {
        x.floor()
    }
    /// `floor` (double).
    pub fn floor_d(x: f64) -> f64 {
        x.floor()
    }

    /// `ceil`.
    pub fn ceil(x: f32) -> f32 {
        x.ceil()
    }
    /// `ceil` (double).
    pub fn ceil_d(x: f64) -> f64 {
        x.ceil()
    }

    /// `round`.
    pub fn round(x: f32) -> f32 {
        x.round()
    }
    /// `round` (double).
    pub fn round_d(x: f64) -> f64 {
        x.round()
    }

    /// `trunc`.
    pub fn trunc(x: f32) -> f32 {
        x.trunc()
    }
    /// `trunc` (double).
    pub fn trunc_d(x: f64) -> f64 {
        x.trunc()
    }

    /// `rint`.
    pub fn rint(x: f32) -> f32 {
        x.round_ties_even()
    }
    /// `rint` (double).
    pub fn rint_d(x: f64) -> f64 {
        x.round_ties_even()
    }

    /// `IEEEremainder`.
    pub fn ieee_remainder(x: f32, y: f32) -> f32 {
        x % y
    }
    /// `IEEEremainder` (double).
    pub fn ieee_remainder_d(x: f64, y: f64) -> f64 {
        x % y
    }

    /// `nextAfter`.
    pub fn next_after(x: f32, direction: f32) -> f32 {
        if x == direction {
            direction
        } else {
            x.next_up() * if direction > x { 1.0 } else { -1.0 }
        }
    }

    /// `nextUp`.
    pub fn next_up(x: f32) -> f32 {
        x.next_up()
    }
    /// `nextUp` (double).
    pub fn next_up_d(x: f64) -> f64 {
        x.next_up()
    }

    /// `nextDown`.
    pub fn next_down(x: f32) -> f32 {
        x.next_down()
    }
    /// `nextDown` (double).
    pub fn next_down_d(x: f64) -> f64 {
        x.next_down()
    }

    /// `copySign`.
    pub fn copy_sign(x: f32, y: f32) -> f32 {
        x.copysign(y)
    }
    /// `copySign` (double).
    pub fn copy_sign_d(x: f64, y: f64) -> f64 {
        x.copysign(y)
    }

    /// `scalb`.
    pub fn scalb(x: f32, scale_factor: i32) -> f32 {
        x * 2f32.powi(scale_factor)
    }
    /// `scalb` (double).
    pub fn scalb_d(x: f64, scale_factor: i32) -> f64 {
        x * 2f64.powi(scale_factor)
    }

    /// `ulp`.
    pub fn ulp(x: f32) -> f32 {
        if x.is_nan() || x.is_infinite() {
            f32::NAN
        } else if x == 0.0 {
            f32::MIN_POSITIVE
        } else {
            let bits = x.to_bits();
            let abs_bits = bits & !(1u32 << 31);
            let next_bits = if x > 0.0 { abs_bits + 1 } else { abs_bits - 1 };
            let next = f32::from_bits(if x > 0.0 { next_bits } else { next_bits | (1u32 << 31) });
            (next - x).abs()
        }
    }

    // ---- min/max/clamp ---------------------------------------------------

    /// `max`.
    pub fn max(a: f32, b: f32) -> f32 {
        a.max(b)
    }
    /// `max` (double).
    pub fn max_d(a: f64, b: f64) -> f64 {
        a.max(b)
    }

    /// `min`.
    pub fn min(a: f32, b: f32) -> f32 {
        a.min(b)
    }
    /// `min` (double).
    pub fn min_d(a: f64, b: f64) -> f64 {
        a.min(b)
    }

    /// `clamp(value, min, max)`.
    pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.max(min).min(max)
    }
    /// `clamp` (double).
    pub fn clamp_d(value: f64, min: f64, max: f64) -> f64 {
        value.max(min).min(max)
    }

    /// `clamp` (int).
    pub fn clamp_i(value: i32, min: i32, max: i32) -> i32 {
        value.max(min).min(max)
    }
    /// `clamp` (long).
    pub fn clamp_l(value: i64, min: i64, max: i64) -> i64 {
        value.max(min).min(max)
    }

    // ---- vector-ish helpers ----------------------------------------------

    /// `sqr(x)` — квадрат.
    pub fn sqr(x: f32) -> f32 {
        x * x
    }
    /// `sqr` (double).
    pub fn sqr_d(x: f64) -> f64 {
        x * x
    }

    /// `length(x, y)`.
    pub fn length_2d(x: f32, y: f32) -> f32 {
        Self::sqrt(x * x + y * y)
    }

    /// `length(x, y, z)`.
    pub fn length_3d(x: f32, y: f32, z: f32) -> f32 {
        Self::sqrt(x * x + y * y + z * z)
    }

    /// `length(x, y, z, w)`.
    pub fn length_4d(x: f32, y: f32, z: f32, w: f32) -> f32 {
        Self::sqrt(x * x + y * y + z * z + w * w)
    }

    /// `fma(a, b, c)` — fused multiply-add.
    pub fn fma(a: f32, b: f32, c: f32) -> f32 {
        a.mul_add(b, c)
    }
    /// `fma` (double).
    pub fn fma_d(a: f64, b: f64, c: f64) -> f64 {
        a.mul_add(b, c)
    }

    /// `sincos(angle)` → `(sin, cos)`.
    pub fn sin_cos(x: f32) -> (f32, f32) {
        x.sin_cos()
    }
    /// `sincos` (double).
    pub fn sin_cos_d(x: f64) -> (f64, f64) {
        x.sin_cos()
    }

    /// `sincos` с записью в срез — JOML пишет в float[2]/double[2].
    pub fn sin_cos_into(x: f32, out: &mut [f32]) {
        let (s, c) = x.sin_cos();
        out[0] = s;
        out[1] = c;
    }

    /// `normalize(vec, i)` — JOML `normalize(float[] v, int i)`.
    pub fn normalize_vec(v: &mut [f32], i: usize) {
        let len = Self::sqrt(v[i] * v[i] + v[i + 1] * v[i + 1] + v[i + 2] * v[i + 2]);
        if len != 0.0 {
            let inv = 1.0 / len;
            v[i] *= inv;
            v[i + 1] *= inv;
            v[i + 2] *= inv;
        }
    }

    /// `normalize` для 4-компонентного.
    pub fn normalize_vec4(v: &mut [f32], i: usize) {
        let len = Self::sqrt(
            v[i] * v[i] + v[i + 1] * v[i + 1] + v[i + 2] * v[i + 2] + v[i + 3] * v[i + 3],
        );
        if len != 0.0 {
            let inv = 1.0 / len;
            v[i] *= inv;
            v[i + 1] *= inv;
            v[i + 2] *= inv;
            v[i + 3] *= inv;
        }
    }

    /// `smoothstep` (edge0=0, edge1=1).
    pub fn smoothstep(x: f32) -> f32 {
        Self::smoothstep_(x, 0.0, 1.0)
    }

    /// `smoothstep(x, edge0, edge1)`.
    pub fn smoothstep_(x: f32, edge0: f32, edge1: f32) -> f32 {
        let t = Self::clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// `cubicSpline(p0, p1, p2, p3, t)`.
    pub fn cubic_spline(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        (p1 * 2.0 + (p2 - p0) * t + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t2
            + (3.0 * (p1 - p2) + p3 - p0) * t3)
            * 0.5
    }

    /// `cubicSpline` — 6 точек (полная форма JOML).
    pub fn cubic_spline_6(
        p0: f32,
        p1: f32,
        p2: f32,
        p3: f32,
        p4: f32,
        p5: f32,
        t: f32,
    ) -> f32 {
        let dt = 1.0 / 3.0;
        if t < dt {
            Self::cubic_spline(p0, p1, p2, p3, t / dt)
        } else if t > 1.0 - dt {
            Self::cubic_spline(p3, p4, p5, p5, (t - (1.0 - dt)) / dt)
        } else {
            let u = (t - dt) / (1.0 - 2.0 * dt);
            Self::cubic_spline(p1, p2, p3, p4, u)
        }
    }

    /// `fuzzyEquals`? — в JOML есть `Math.fuzzyEquals`? Нет, но векторы его имеют.

    /// `isFinite`.
    pub fn is_finite(x: f32) -> bool {
        x.is_finite()
    }
    /// `isNaN`.
    pub fn is_nan(x: f32) -> bool {
        x.is_nan()
    }
}
