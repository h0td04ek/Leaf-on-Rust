//! Полный перенос `org.joml.Vector2f` (Vector2fc).

use crate::joml::math::Math;
use crate::joml::matrix2f::Matrix2f;
use crate::joml::matrix3x2f::Matrix3x2f;
use crate::joml::vector3f::Vector3f;

/// `Vector2f`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2f {
    /// `x`.
    pub x: f32,
    /// `y`.
    pub y: f32,
}

impl Vector2f {
    /// `ZERO`.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    /// `ONE`.
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    /// `UNIT_X`.
    pub const UNIT_X: Self = Self { x: 1.0, y: 0.0 };
    /// `UNIT_Y`.
    pub const UNIT_Y: Self = Self { x: 0.0, y: 1.0 };
    /// `NAN`.
    pub const NAN: Self = Self { x: f32::NAN, y: f32::NAN };

    /// `new Vector2f()`.
    pub fn new_zero() -> Self {
        Self::ZERO
    }

    /// `new Vector2f(x, y)`.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// `new Vector2f(splat)`.
    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    /// `new Vector2f(Vector2fc)`.
    pub fn of(v: &Vector2f) -> Self {
        *v
    }

    /// `new Vector2f(Vector3fc)` — берёт x,y.
    pub fn from_vec3(v: &Vector3f) -> Self {
        Self::new(v.x, v.y)
    }

    /// `x()`, `y()`.
    pub fn get_x(&self) -> f32 {
        self.x
    }
    /// `y()`.
    pub fn get_y(&self) -> f32 {
        self.y
    }

    // ---- set -----------------------------------------------------------------

    /// `set(x,y)`.
    pub fn set(&mut self, x: f32, y: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }

    /// `set(splat)`.
    pub fn set_splat(&mut self, v: f32) -> &mut Self {
        self.x = v;
        self.y = v;
        self
    }

    /// `set(Vector2fc)`.
    pub fn set_vec(&mut self, v: &Vector2f) -> &mut Self {
        *self = *v;
        self
    }

    /// `set(Vector3fc)`.
    pub fn set_vec3(&mut self, v: &Vector3f) -> &mut Self {
        self.x = v.x;
        self.y = v.y;
        self
    }

    /// `setComponent`.
    pub fn set_component(&mut self, index: i32, value: f32) {
        match index {
            0 => self.x = value,
            1 => self.y = value,
            _ => panic!("Illegal component index {}", index),
        }
    }

    /// `get(int)`.
    pub fn get_component(&self, index: i32) -> f32 {
        match index {
            0 => self.x,
            1 => self.y,
            _ => panic!("Illegal component index {}", index),
        }
    }

    /// `set(float[])`.
    pub fn set_slice(&mut self, v: &[f32]) -> &mut Self {
        self.x = v[0];
        self.y = v[1];
        self
    }

    /// `toArray`.
    pub fn to_array(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    /// `get(Vector2f)`.
    pub fn get(&self) -> Self {
        *self
    }

    // ---- arithmetic ------------------------------------------------------------

    /// `add(x,y)`.
    pub fn add(&mut self, x: f32, y: f32) -> &mut Self {
        self.x += x;
        self.y += y;
        self
    }

    /// `add(Vector2fc)`.
    pub fn add_vec(&mut self, v: &Vector2f) -> &mut Self {
        self.x += v.x;
        self.y += v.y;
        self
    }

    /// `sub(x,y)`.
    pub fn sub(&mut self, x: f32, y: f32) -> &mut Self {
        self.x -= x;
        self.y -= y;
        self
    }

    /// `sub(Vector2fc)`.
    pub fn sub_vec(&mut self, v: &Vector2f) -> &mut Self {
        self.x -= v.x;
        self.y -= v.y;
        self
    }

    /// `mul(float)`.
    pub fn mul_scalar(&mut self, s: f32) -> &mut Self {
        self.x *= s;
        self.y *= s;
        self
    }

    /// `mul(x,y)`.
    pub fn mul(&mut self, x: f32, y: f32) -> &mut Self {
        self.x *= x;
        self.y *= y;
        self
    }

    /// `mul(Vector2fc)`.
    pub fn mul_vec(&mut self, v: &Vector2f) -> &mut Self {
        self.x *= v.x;
        self.y *= v.y;
        self
    }

    /// `div(float)`.
    pub fn div_scalar(&mut self, s: f32) -> &mut Self {
        self.x /= s;
        self.y /= s;
        self
    }

    /// `div(x,y)`.
    pub fn div(&mut self, x: f32, y: f32) -> &mut Self {
        self.x /= x;
        self.y /= y;
        self
    }

    /// `div(Vector2fc)`.
    pub fn div_vec(&mut self, v: &Vector2f) -> &mut Self {
        self.x /= v.x;
        self.y /= v.y;
        self
    }

    /// `fma(a, b)` — this = a*b + this.
    pub fn fma_vec(&mut self, a: &Vector2f, b: &Vector2f) -> &mut Self {
        self.x = a.x.mul_add(b.x, self.x);
        self.y = a.y.mul_add(b.y, self.y);
        self
    }

    /// `fma(scalar, b)`.
    pub fn fma_scalar(&mut self, s: f32, b: &Vector2f) -> &mut Self {
        self.x = s.mul_add(b.x, self.x);
        self.y = s.mul_add(b.y, self.y);
        self
    }

    /// `mulAdd` — алиас fma.
    pub fn mul_add(&mut self, a: &Vector2f, b: &Vector2f) -> &mut Self {
        self.x += a.x * b.x;
        self.y += a.y * b.y;
        self
    }

    /// `negate`.
    pub fn negate(&mut self) -> &mut Self {
        self.x = -self.x;
        self.y = -self.y;
        self
    }

    /// `absolute`.
    pub fn absolute(&mut self) -> &mut Self {
        self.x = self.x.abs();
        self.y = self.y.abs();
        self
    }

    /// `floor`.
    pub fn floor(&mut self) -> &mut Self {
        self.x = self.x.floor();
        self.y = self.y.floor();
        self
    }

    /// `ceil`.
    pub fn ceil(&mut self) -> &mut Self {
        self.x = self.x.ceil();
        self.y = self.y.ceil();
        self
    }

    /// `round`.
    pub fn round(&mut self) -> &mut Self {
        self.x = self.x.round();
        self.y = self.y.round();
        self
    }

    /// `min(Vector2fc)`.
    pub fn min_vec(&mut self, v: &Vector2f) -> &mut Self {
        self.x = self.x.min(v.x);
        self.y = self.y.min(v.y);
        self
    }

    /// `max(Vector2fc)`.
    pub fn max_vec(&mut self, v: &Vector2f) -> &mut Self {
        self.x = self.x.max(v.x);
        self.y = self.y.max(v.y);
        self
    }

    /// `zero`.
    pub fn zero(&mut self) -> &mut Self {
        self.set(0.0, 0.0)
    }

    // ---- measurements ------------------------------------------------------------

    /// `lengthSquared`.
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// `lengthSquared(x,y)`.
    pub fn length_squared_floats(x: f32, y: f32) -> f32 {
        x * x + y * y
    }

    /// `length`.
    pub fn length(&self) -> f32 {
        Math::sqrt(self.length_squared())
    }

    /// `length(x,y)`.
    pub fn length_floats(x: f32, y: f32) -> f32 {
        Math::sqrt(Self::length_squared_floats(x, y))
    }

    /// `normalize`.
    pub fn normalize(&mut self) -> &mut Self {
        let l = self.length();
        if l > 0.0 {
            self.div_scalar(l);
        }
        self
    }

    /// `normalize(length)`.
    pub fn normalize_len(&mut self, length: f32) -> &mut Self {
        self.div_scalar(length);
        self
    }

    /// `distance(Vector2fc)`.
    pub fn distance(&self, v: &Vector2f) -> f32 {
        Math::sqrt(self.distance_squared(v))
    }

    /// `distance(x,y)`.
    pub fn distance_floats(&self, x: f32, y: f32) -> f32 {
        Math::sqrt(self.distance_squared_floats(x, y))
    }

    /// `distanceSquared(Vector2fc)`.
    pub fn distance_squared(&self, v: &Vector2f) -> f32 {
        let dx = self.x - v.x;
        let dy = self.y - v.y;
        dx * dx + dy * dy
    }

    /// `distanceSquared(x,y)`.
    pub fn distance_squared_floats(&self, x: f32, y: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        dx * dx + dy * dy
    }

    /// `distance` статический.
    pub fn distance_static(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
        Math::sqrt(Self::distance_squared_static(ax, ay, bx, by))
    }

    /// `distanceSquared` статический.
    pub fn distance_squared_static(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
        let dx = ax - bx;
        let dy = ay - by;
        dx * dx + dy * dy
    }

    /// `dot(Vector2fc)`.
    pub fn dot_vec(&self, v: &Vector2f) -> f32 {
        self.x * v.x + self.y * v.y
    }

    /// `dot(x,y)`.
    pub fn dot(&self, x: f32, y: f32) -> f32 {
        self.x * x + self.y * y
    }

    /// `perpendicular` — (y, -x)? В JOML perpendicular = (-y, x).
    pub fn perpendicular(&mut self) -> &mut Self {
        let x = self.x;
        self.x = -self.y;
        self.y = x;
        self
    }

    /// `angle(Vector2fc)`.
    pub fn angle(&self, v: &Vector2f) -> f32 {
        let d = self.dot_vec(v);
        let l = self.length() * v.length();
        if l == 0.0 {
            return 0.0;
        }
        Math::acos((d / l).clamp(-1.0, 1.0))
    }

    // ---- matrix ------------------------------------------------------------------

    /// `mul(Matrix2fc)` — M * v.
    pub fn mul_matrix2(&mut self, m: &Matrix2f) -> &mut Self {
        let (x, y) = (self.x, self.y);
        self.x = m.m00 * x + m.m10 * y;
        self.y = m.m01 * x + m.m11 * y;
        self
    }

    /// `mulTranspose(Matrix2fc)` — Mᵀ * v.
    pub fn mul_transpose_matrix2(&mut self, m: &Matrix2f) -> &mut Self {
        let (x, y) = (self.x, self.y);
        self.x = m.m00 * x + m.m01 * y;
        self.y = m.m10 * x + m.m11 * y;
        self
    }

    /// `mulPosition(Matrix3x2fc)` — affine: x' = m00*x + m10*y + m20.
    pub fn mul_position(&mut self, m: &Matrix3x2f) -> &mut Self {
        let (x, y) = (self.x, self.y);
        self.x = m.m00 * x + m.m10 * y + m.m20;
        self.y = m.m01 * x + m.m11 * y + m.m21;
        self
    }

    /// `mulDirection(Matrix3x2fc)` — без переноса.
    pub fn mul_direction(&mut self, m: &Matrix3x2f) -> &mut Self {
        let (x, y) = (self.x, self.y);
        self.x = m.m00 * x + m.m10 * y;
        self.y = m.m01 * x + m.m11 * y;
        self
    }

    // ---- interpolation ----------------------------------------------------------------

    /// `lerp(v, t)`.
    pub fn lerp(&mut self, v: &Vector2f, t: f32) -> &mut Self {
        self.x += (v.x - self.x) * t;
        self.y += (v.y - self.y) * t;
        self
    }

    /// `equals` с delta.
    pub fn fuzzy_equals(&self, other: &Vector2f, delta: f32) -> bool {
        (self.x - other.x).abs() <= delta && (self.y - other.y).abs() <= delta
    }

    /// `equals(x,y)`.
    pub fn equals_floats(&self, x: f32, y: f32) -> bool {
        self.x == x && self.y == y
    }

    /// `maxComponent`/`minComponent`.
    pub fn max_component(&self) -> i32 {
        if self.x > self.y { 0 } else { 1 }
    }

    /// `minComponent`.
    pub fn min_component(&self) -> i32 {
        if self.x < self.y { 0 } else { 1 }
    }

    /// `isFinite`.
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// `toString`.
    pub fn to_string_fmt(&self) -> String {
        format!("({} {})", self.x, self.y)
    }
}
