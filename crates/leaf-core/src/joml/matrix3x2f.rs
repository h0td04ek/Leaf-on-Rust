//! Полный перенос `org.joml.Matrix3x2f` — аффинная матрица 3×2
//! (3 столбца × 2 строки: x-ось, y-ось, перенос).
//!
//! Имена полей как в JOML: `mXY` = столбец X, строка Y.
//! Столбец 0 = (m00, m01), столбец 1 = (m10, m11),
//! столбец 2 = перенос (m20, m21).

use crate::joml::math::Math;
use crate::joml::vector2f::Vector2f;

/// `Matrix3x2f`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix3x2f {
    /// Столбец 0.
    pub m00: f32,
    pub m01: f32,
    /// Столбец 1.
    pub m10: f32,
    pub m11: f32,
    /// Столбец 2 (перенос).
    pub m20: f32,
    pub m21: f32,
}

impl Matrix3x2f {
    /// `new Matrix3x2f()` — identity.
    pub fn new() -> Self {
        Self::identity()
    }

    /// Все 6 компонент в порядке JOML `set(m00,m01,m10,m11,m20,m21)`.
    pub fn from_components(m00: f32, m01: f32, m10: f32, m11: f32, m20: f32, m21: f32) -> Self {
        Self { m00, m01, m10, m11, m20, m21 }
    }

    /// `new Matrix3x2f(Matrix3x2fc)`.
    pub fn of(m: &Matrix3x2f) -> Self {
        *m
    }

    /// `identity`.
    pub fn identity() -> Self {
        Self { m00: 1.0, m01: 0.0, m10: 0.0, m11: 1.0, m20: 0.0, m21: 0.0 }
    }

    /// `zero`.
    pub fn zero() -> Self {
        Self { m00: 0.0, m01: 0.0, m10: 0.0, m11: 0.0, m20: 0.0, m21: 0.0 }
    }

    /// `set(m00, m01, m10, m11, m20, m21)`.
    pub fn set(&mut self, m00: f32, m01: f32, m10: f32, m11: f32, m20: f32, m21: f32) -> &mut Self {
        self.m00 = m00;
        self.m01 = m01;
        self.m10 = m10;
        self.m11 = m11;
        self.m20 = m20;
        self.m21 = m21;
        self
    }

    /// `set(Matrix3x2fc)`.
    pub fn set_mat(&mut self, m: &Matrix3x2f) -> &mut Self {
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

    /// `get`.
    pub fn get(&self) -> Self {
        *self
    }

    /// `setTranslation(x, y)` — только компоненты переноса.
    pub fn set_translation(&mut self, x: f32, y: f32) -> &mut Self {
        self.m20 = x;
        self.m21 = y;
        self
    }

    /// `setTranslation(Vector2fc)`.
    pub fn set_translation_vec(&mut self, v: &Vector2f) -> &mut Self {
        self.set_translation(v.x, v.y)
    }

    /// `getTranslation`.
    pub fn get_translation(&self) -> Vector2f {
        Vector2f::new(self.m20, self.m21)
    }

    /// `set(float[])` — column-major порядок (m00,m01,m10,m11,m20,m21).
    pub fn set_slice(&mut self, v: &[f32]) -> &mut Self {
        self.m00 = v[0];
        self.m01 = v[1];
        self.m10 = v[2];
        self.m11 = v[3];
        self.m20 = v[4];
        self.m21 = v[5];
        self
    }

    /// `get(float[])` — column-major.
    pub fn write_slice(&self, out: &mut [f32]) {
        out[0] = self.m00;
        out[1] = self.m01;
        out[2] = self.m10;
        out[3] = self.m11;
        out[4] = self.m20;
        out[5] = self.m21;
    }

    /// `get(int)` / `set(int, float)` — по индексу column-major.
    pub fn get_component(&self, index: i32) -> f32 {
        match index {
            0 => self.m00,
            1 => self.m01,
            2 => self.m10,
            3 => self.m11,
            4 => self.m20,
            5 => self.m21,
            _ => panic!("Illegal matrix index {}", index),
        }
    }

    /// `set(int, float)`.
    pub fn set_component(&mut self, index: i32, value: f32) {
        match index {
            0 => self.m00 = value,
            1 => self.m01 = value,
            2 => self.m10 = value,
            3 => self.m11 = value,
            4 => self.m20 = value,
            5 => self.m21 = value,
            _ => panic!("Illegal matrix index {}", index),
        }
    }

    // ---- determinant / inverse -------------------------------------------------

    /// `determinant` — по 2×2 линейной части.
    pub fn determinant(&self) -> f32 {
        self.m00 * self.m11 - self.m01 * self.m10
    }

    /// `invert` — this = this⁻¹ (предполагается 3-я строка (0,0,1)).
    pub fn invert(&mut self) -> &mut Self {
        let s = 1.0 / self.determinant();
        let (m00, m01, m10, m11, m20, m21) =
            (self.m00, self.m01, self.m10, self.m11, self.m20, self.m21);
        self.m00 = m11 * s;
        self.m01 = -m01 * s;
        self.m10 = -m10 * s;
        self.m11 = m00 * s;
        self.m20 = (m10 * m21 - m20 * m11) * s;
        self.m21 = (m20 * m01 - m00 * m21) * s;
        self
    }

    /// `invert(m, dest)`.
    pub fn invert_into(&self, dest: &mut Matrix3x2f) -> Matrix3x2f {
        let s = 1.0 / self.determinant();
        dest.m00 = self.m11 * s;
        dest.m01 = -self.m01 * s;
        dest.m10 = -self.m10 * s;
        dest.m11 = self.m00 * s;
        dest.m20 = (self.m10 * self.m21 - self.m20 * self.m11) * s;
        dest.m21 = (self.m20 * self.m01 - self.m00 * self.m21) * s;
        *dest
    }

    /// `invert(m)` — this = this⁻¹ * m.
    pub fn invert_mat(&mut self, m: &Matrix3x2f) -> &mut Self {
        let mut inv = *self;
        inv.invert();
        inv.mul_into(m, self);
        self
    }

    // ---- multiplication ---------------------------------------------------------

    /// `mul(Matrix3x2fc)` — this = this * m.
    pub fn mul_mat(&mut self, m: &Matrix3x2f) -> &mut Self {
        let (a00, a01, a10, a11, a20, a21) =
            (self.m00, self.m01, self.m10, self.m11, self.m20, self.m21);
        self.m00 = a00 * m.m00 + a10 * m.m01;
        self.m01 = a01 * m.m00 + a11 * m.m01;
        self.m10 = a00 * m.m10 + a10 * m.m11;
        self.m11 = a01 * m.m10 + a11 * m.m11;
        self.m20 = a00 * m.m20 + a10 * m.m21 + a20;
        self.m21 = a01 * m.m20 + a11 * m.m21 + a21;
        self
    }

    /// `mul(m, dest)`.
    pub fn mul_into(&self, m: &Matrix3x2f, dest: &mut Matrix3x2f) -> Matrix3x2f {
        dest.m00 = self.m00 * m.m00 + self.m10 * m.m01;
        dest.m01 = self.m01 * m.m00 + self.m11 * m.m01;
        dest.m10 = self.m00 * m.m10 + self.m10 * m.m11;
        dest.m11 = self.m01 * m.m10 + self.m11 * m.m11;
        dest.m20 = self.m00 * m.m20 + self.m10 * m.m21 + self.m20;
        dest.m21 = self.m01 * m.m20 + self.m11 * m.m21 + self.m21;
        *dest
    }

    // ---- transforms ---------------------------------------------------------------

    /// `transformPosition(Vector2fc)` — точка (z=1) с делением? JOML
    /// mulPosition для 3x2 — без перспективы (w всегда 1).
    pub fn transform_vec(&self, v: &Vector2f, dest: &mut Vector2f) -> Vector2f {
        dest.x = self.m00 * v.x + self.m10 * v.y + self.m20;
        dest.y = self.m01 * v.x + self.m11 * v.y + self.m21;
        *dest
    }

    /// `transformDirection(Vector2fc)` — направление (без переноса).
    pub fn transform_direction(&self, v: &Vector2f, dest: &mut Vector2f) -> Vector2f {
        dest.x = self.m00 * v.x + self.m10 * v.y;
        dest.y = self.m01 * v.x + self.m11 * v.y;
        *dest
    }

    /// `transformTransposePosition` — Mᵀ * (v,1).
    pub fn transform_transpose_vec(&self, v: &Vector2f, dest: &mut Vector2f) -> Vector2f {
        dest.x = self.m00 * v.x + self.m01 * v.y + self.m20;
        dest.y = self.m10 * v.x + self.m11 * v.y + self.m21;
        *dest
    }

    /// `transformTransposeDirection` — Mᵀ * (v,0).
    pub fn transform_transpose_direction(&self, v: &Vector2f, dest: &mut Vector2f) -> Vector2f {
        dest.x = self.m00 * v.x + self.m01 * v.y;
        dest.y = self.m10 * v.x + self.m11 * v.y;
        *dest
    }

    // ---- builders -----------------------------------------------------------------

    /// `translate(x, y)` — this = this * T.
    pub fn translate(&mut self, x: f32, y: f32) -> &mut Self {
        self.m20 = self.m00 * x + self.m10 * y + self.m20;
        self.m21 = self.m01 * x + self.m11 * y + self.m21;
        self
    }

    /// `translate(Vector2fc)`.
    pub fn translate_vec(&mut self, v: &Vector2f) -> &mut Self {
        self.translate(v.x, v.y)
    }

    /// `translateLocal(x, y)` — this = T * this.
    pub fn translate_local(&mut self, x: f32, y: f32) -> &mut Self {
        self.m20 += x;
        self.m21 += y;
        self
    }

    /// `rotate(angle)` — this = this * R(angle). Перенос не меняется.
    pub fn rotate(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle);
        let c = Math::cos(angle);
        let (a00, a01, a10, a11) = (self.m00, self.m01, self.m10, self.m11);
        self.m00 = a00 * c + a10 * s;
        self.m01 = a01 * c + a11 * s;
        self.m10 = a00 * -s + a10 * c;
        self.m11 = a01 * -s + a11 * c;
        self
    }

    /// `rotateLocal(angle)` — this = R(angle) * this. Каждый столбец
    /// поворачивается, формулы JOML rotateLocal.
    pub fn rotate_local(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle);
        let c = Math::cos(angle);
        let (a00, a01, a10, a11, a20, a21) =
            (self.m00, self.m01, self.m10, self.m11, self.m20, self.m21);
        self.m00 = c * a00 - s * a01;
        self.m01 = s * a00 + c * a01;
        self.m10 = c * a10 - s * a11;
        self.m11 = s * a10 + c * a11;
        self.m20 = c * a20 - s * a21;
        self.m21 = s * a20 + c * a21;
        self
    }

    /// `rotation(angle)` — матрица чистого поворота.
    pub fn rotation(&mut self, angle: f32) -> &mut Self {
        *self = Self::identity();
        self.rotate(angle);
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
        self.scale(s, s)
    }

    /// `equals` с delta.
    pub fn fuzzy_equals(&self, other: &Matrix3x2f, delta: f32) -> bool {
        (self.m00 - other.m00).abs() <= delta
            && (self.m01 - other.m01).abs() <= delta
            && (self.m10 - other.m10).abs() <= delta
            && (self.m11 - other.m11).abs() <= delta
            && (self.m20 - other.m20).abs() <= delta
            && (self.m21 - other.m21).abs() <= delta
    }

    /// `isFinite`.
    pub fn is_finite(&self) -> bool {
        self.m00.is_finite()
            && self.m01.is_finite()
            && self.m10.is_finite()
            && self.m11.is_finite()
            && self.m20.is_finite()
            && self.m21.is_finite()
    }

    /// `toString`.
    pub fn to_string_fmt(&self) -> String {
        format!(
            "Matrix3x2f({} {} {} {} {} {})",
            self.m00, self.m01, self.m10, self.m11, self.m20, self.m21
        )
    }
}

impl Default for Matrix3x2f {
    fn default() -> Self {
        Self::new()
    }
}
