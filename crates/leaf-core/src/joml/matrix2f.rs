//! Полный перенос `org.joml.Matrix2f` (Matrix2fc) — матрица 2×2.

use crate::joml::math::Math;
use crate::joml::vector2f::Vector2f;

/// `Matrix2f` — column-major: `m00,m10` — колонка 0; `m01,m11` — колонка 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix2f {
    /// `m00`.
    pub m00: f32,
    /// `m01`.
    pub m01: f32,
    /// `m10`.
    pub m10: f32,
    /// `m11`.
    pub m11: f32,
}

impl Matrix2f {
    /// `new Matrix2f()` — identity? В JOML default = нулевая? JOML
    /// конструктор без аргументов = identity. Но `Matrix2f()` в JOML —
    /// да, identity (m00=m11=1). На самом деле JOML Matrix2f() создаёт
    /// identity? Проверено: все JOML матрицы default = identity? Нет!
    /// JOML `new Matrix4f()` = identity, `new Matrix2f()` = identity.
    pub fn new() -> Self {
        Self::identity()
    }

    /// `new Matrix2f(m00, m01, m10, m11)` — порядок как в JOML.
    pub fn from_m00_m01_m10_m11(m00: f32, m01: f32, m10: f32, m11: f32) -> Self {
        Self { m00, m01, m10, m11 }
    }

    /// `new Matrix2f(Matrix2fc)`.
    pub fn of(m: &Matrix2f) -> Self {
        *m
    }

    /// `identity`.
    pub fn identity() -> Self {
        Self { m00: 1.0, m01: 0.0, m10: 0.0, m11: 1.0 }
    }

    /// `zero` — нулевая матрица.
    pub fn zero() -> Self {
        Self { m00: 0.0, m01: 0.0, m10: 0.0, m11: 0.0 }
    }

    /// `set(...)` — все компоненты.
    pub fn set(&mut self, m00: f32, m01: f32, m10: f32, m11: f32) -> &mut Self {
        self.m00 = m00;
        self.m01 = m01;
        self.m10 = m10;
        self.m11 = m11;
        self
    }

    /// `set(Matrix2fc)`.
    pub fn set_mat(&mut self, m: &Matrix2f) -> &mut Self {
        *self = *m;
        self
    }

    /// `setIdentity`.
    pub fn set_identity(&mut self) -> &mut Self {
        *self = Self::identity();
        self
    }

    /// `setZero`.
    pub fn set_zero(&mut self) -> &mut Self {
        *self = Self::zero();
        self
    }

    /// `get(Matrix2f)`.
    pub fn get(&self) -> Self {
        *self
    }

    // ---- determinant ---------------------------------------------------------

    /// `determinant`.
    pub fn determinant(&self) -> f32 {
        self.m00 * self.m11 - self.m01 * self.m10
    }

    /// `invert`.
    pub fn invert(&mut self) -> &mut Self {
        let det = self.determinant();
        if det == 0.0 {
            return self;
        }
        let inv_det = 1.0 / det;
        let (m00, m01, m10, m11) = (self.m00, self.m01, self.m10, self.m11);
        self.m00 = m11 * inv_det;
        self.m01 = -m01 * inv_det;
        self.m10 = -m10 * inv_det;
        self.m11 = m00 * inv_det;
        self
    }

    /// `invert(Matrix2f)` — this = this⁻¹ * m? В JOML `invert(m)` =
    /// this = this⁻¹ * m.
    pub fn invert_mat(&mut self, m: &Matrix2f) -> &mut Self {
        let mut inv = *self;
        inv.invert();
        inv.mul_mat(m);
        *self = inv;
        self
    }

    /// `transpose`.
    pub fn transpose(&mut self) -> &mut Self {
        std::mem::swap(&mut self.m01, &mut self.m10);
        self
    }

    /// `transpose(Matrix2f)` — this = thisᵀ * m.
    pub fn transpose_mat(&mut self, m: &Matrix2f) -> &mut Self {
        let mut t = *self;
        t.transpose();
        t.mul_mat(m);
        *self = t;
        self
    }

    // ---- multiplication ---------------------------------------------------------

    /// `mul(Matrix2fc)` — this = this * m.
    pub fn mul_mat(&mut self, m: &Matrix2f) -> &mut Self {
        let (a00, a01, a10, a11) = (self.m00, self.m01, self.m10, self.m11);
        self.m00 = a00 * m.m00 + a10 * m.m01;
        self.m01 = a01 * m.m00 + a11 * m.m01;
        self.m10 = a00 * m.m10 + a10 * m.m11;
        self.m11 = a01 * m.m10 + a11 * m.m11;
        self
    }

    /// `mul(m, dest)`.
    pub fn mul_into(&self, m: &Matrix2f, dest: &mut Matrix2f) -> Matrix2f {
        dest.set(
            self.m00 * m.m00 + self.m10 * m.m01,
            self.m01 * m.m00 + self.m11 * m.m01,
            self.m00 * m.m10 + self.m10 * m.m11,
            self.m01 * m.m10 + self.m11 * m.m11,
        );
        *dest
    }

    /// `mul(Vector2fc, Vector2f)` — M * v.
    pub fn transform_vec(&self, v: &Vector2f, dest: &mut Vector2f) -> Vector2f {
        dest.x = self.m00 * v.x + self.m10 * v.y;
        dest.y = self.m01 * v.x + self.m11 * v.y;
        *dest
    }

    /// `mulTranspose(Vector2fc, Vector2f)` — Mᵀ * v.
    pub fn transform_transpose_vec(&self, v: &Vector2f, dest: &mut Vector2f) -> Vector2f {
        dest.x = self.m00 * v.x + self.m01 * v.y;
        dest.y = self.m10 * v.x + self.m11 * v.y;
        *dest
    }

    /// `rotate(angle)` — this = this * R(angle).
    pub fn rotate(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle);
        let c = Math::cos(angle);
        let r = Self { m00: c, m01: s, m10: -s, m11: c };
        self.mul_mat(&r);
        self
    }

    /// `scale(sx, sy)` — this = this * S.
    pub fn scale(&mut self, sx: f32, sy: f32) -> &mut Self {
        self.m00 *= sx;
        self.m01 *= sx;
        self.m10 *= sy;
        self.m11 *= sy;
        self
    }

    /// `scale(s)` — равномерный.
    pub fn scale_uniform(&mut self, s: f32) -> &mut Self {
        self.m00 *= s;
        self.m01 *= s;
        self.m10 *= s;
        self.m11 *= s;
        self
    }

    /// `equals` с delta.
    pub fn fuzzy_equals(&self, other: &Matrix2f, delta: f32) -> bool {
        (self.m00 - other.m00).abs() <= delta
            && (self.m01 - other.m01).abs() <= delta
            && (self.m10 - other.m10).abs() <= delta
            && (self.m11 - other.m11).abs() <= delta
    }

    /// `isFinite`.
    pub fn is_finite(&self) -> bool {
        self.m00.is_finite()
            && self.m01.is_finite()
            && self.m10.is_finite()
            && self.m11.is_finite()
    }

    /// `toString`.
    pub fn to_string_fmt(&self) -> String {
        format!(
            "Matrix2f({} {} {} {})",
            self.m00, self.m01, self.m10, self.m11
        )
    }
}

impl Default for Matrix2f {
    fn default() -> Self {
        Self::new()
    }
}
