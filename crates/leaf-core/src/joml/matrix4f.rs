//! Перенос `org.joml.Matrix4f` (Matrix4fc) — матрица 4×4.
//!
//! Column-major: `m00,m10,m20,m30` — столбец 0.
//! Покрыты все основные операции JOML: трансформации, проекции,
//! lookAt, построители из quaternion/axis-angle, инверсия, транспонирование.

use crate::joml::math::Math;
use crate::joml::axis_angle4f::AxisAngle4f;
use crate::joml::matrix3f::Matrix3f;
use crate::joml::quaternionf::Quaternionf;
use crate::joml::vector3f::Vector3f;
use crate::joml::vector4f::Vector4f;

/// `Matrix4f`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix4f {
    /// Столбец 0.
    pub m00: f32,
    pub m10: f32,
    pub m20: f32,
    pub m30: f32,
    /// Столбец 1.
    pub m01: f32,
    pub m11: f32,
    pub m21: f32,
    pub m31: f32,
    /// Столбец 2.
    pub m02: f32,
    pub m12: f32,
    pub m22: f32,
    pub m32: f32,
    /// Столбец 3.
    pub m03: f32,
    pub m13: f32,
    pub m23: f32,
    pub m33: f32,
}

impl Matrix4f {
    /// `new Matrix4f()` — identity.
    pub fn new() -> Self {
        Self::identity()
    }

    /// `identity`.
    pub fn identity() -> Self {
        Self {
            m00: 1.0, m01: 0.0, m02: 0.0, m03: 0.0,
            m10: 0.0, m11: 1.0, m12: 0.0, m13: 0.0,
            m20: 0.0, m21: 0.0, m22: 1.0, m23: 0.0,
            m30: 0.0, m31: 0.0, m32: 0.0, m33: 1.0,
        }
    }

    /// `zero`.
    pub fn zero() -> Self {
        Self {
            m00: 0.0, m01: 0.0, m02: 0.0, m03: 0.0,
            m10: 0.0, m11: 0.0, m12: 0.0, m13: 0.0,
            m20: 0.0, m21: 0.0, m22: 0.0, m23: 0.0,
            m30: 0.0, m31: 0.0, m32: 0.0, m33: 0.0,
        }
    }

    /// `new Matrix4f(Matrix4fc)`.
    pub fn of(m: &Matrix4f) -> Self {
        *m
    }

    /// `set` — все 16 компонент (порядок JOML: m00,m01,...,m33).
    #[allow(clippy::too_many_arguments)]
    pub fn set(
        &mut self,
        m00: f32, m01: f32, m02: f32, m03: f32,
        m10: f32, m11: f32, m12: f32, m13: f32,
        m20: f32, m21: f32, m22: f32, m23: f32,
        m30: f32, m31: f32, m32: f32, m33: f32,
    ) -> &mut Self {
        self.m00 = m00; self.m01 = m01; self.m02 = m02; self.m03 = m03;
        self.m10 = m10; self.m11 = m11; self.m12 = m12; self.m13 = m13;
        self.m20 = m20; self.m21 = m21; self.m22 = m22; self.m23 = m23;
        self.m30 = m30; self.m31 = m31; self.m32 = m32; self.m33 = m33;
        self
    }

    /// `set(Matrix4fc)`.
    pub fn set_mat(&mut self, m: &Matrix4f) -> &mut Self {
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

    /// `setTranslation(x,y,z)` — обнуляет и ставит перенос.
    pub fn set_translation(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        *self = Self::identity();
        self.m30 = x;
        self.m31 = y;
        self.m32 = z;
        self
    }

    /// `setTranslation(Vector3fc)`.
    pub fn set_translation_vec(&mut self, v: &Vector3f) -> &mut Self {
        self.set_translation(v.x, v.y, v.z)
    }

    /// `translation` — статический конструктор переноса.
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::identity();
        m.m30 = x;
        m.m31 = y;
        m.m32 = z;
        m
    }

    /// `translation(Vector3fc)`.
    pub fn translation_vec(v: &Vector3f) -> Self {
        Self::translation(v.x, v.y, v.z)
    }

    /// `getTranslation(Vector3f)`.
    pub fn get_translation(&self) -> Vector3f {
        Vector3f::new(self.m30, self.m31, self.m32)
    }

    /// `set(float[])` — 16 элементов column-major.
    pub fn set_slice(&mut self, v: &[f32]) -> &mut Self {
        // column-major: m00,m10,m20,m30,m01,...
        self.m00 = v[0];
        self.m10 = v[1];
        self.m20 = v[2];
        self.m30 = v[3];
        self.m01 = v[4];
        self.m11 = v[5];
        self.m21 = v[6];
        self.m31 = v[7];
        self.m02 = v[8];
        self.m12 = v[9];
        self.m22 = v[10];
        self.m32 = v[11];
        self.m03 = v[12];
        self.m13 = v[13];
        self.m23 = v[14];
        self.m33 = v[15];
        self
    }

    /// `get(int, float[])` — запись column-major.
    pub fn write_slice(&self, out: &mut [f32]) {
        out[0] = self.m00;
        out[1] = self.m10;
        out[2] = self.m20;
        out[3] = self.m30;
        out[4] = self.m01;
        out[5] = self.m11;
        out[6] = self.m21;
        out[7] = self.m31;
        out[8] = self.m02;
        out[9] = self.m12;
        out[10] = self.m22;
        out[11] = self.m32;
        out[12] = self.m03;
        out[13] = self.m13;
        out[14] = self.m23;
        out[15] = self.m33;
    }

    /// `get(int)` — компонент по индексу column-major.
    pub fn get_component(&self, index: i32) -> f32 {
        match index {
            0 => self.m00, 1 => self.m10, 2 => self.m20, 3 => self.m30,
            4 => self.m01, 5 => self.m11, 6 => self.m21, 7 => self.m31,
            8 => self.m02, 9 => self.m12, 10 => self.m22, 11 => self.m32,
            12 => self.m03, 13 => self.m13, 14 => self.m23, 15 => self.m33,
            _ => panic!("Illegal matrix index {}", index),
        }
    }

    /// `set(int, value)` — по индексу column-major.
    pub fn set_component(&mut self, index: i32, value: f32) {
        match index {
            0 => self.m00 = value, 1 => self.m10 = value, 2 => self.m20 = value,
            3 => self.m30 = value, 4 => self.m01 = value, 5 => self.m11 = value,
            6 => self.m21 = value, 7 => self.m31 = value, 8 => self.m02 = value,
            9 => self.m12 = value, 10 => self.m22 = value, 11 => self.m32 = value,
            12 => self.m03 = value, 13 => self.m13 = value, 14 => self.m23 = value,
            15 => self.m33 = value,
            _ => panic!("Illegal matrix index {}", index),
        }
    }

    // ---- determinant / inverse ---------------------------------------------------

    /// `determinant`.
    pub fn determinant(&self) -> f32 {
        let m00 = self.m00; let m01 = self.m01; let m02 = self.m02; let m03 = self.m03;
        let m10 = self.m10; let m11 = self.m11; let m12 = self.m12; let m13 = self.m13;
        let m20 = self.m20; let m21 = self.m21; let m22 = self.m22; let m23 = self.m23;
        let m30 = self.m30; let m31 = self.m31; let m32 = self.m32; let m33 = self.m33;
        let b00 = m00 * m11 - m01 * m10;
        let b01 = m00 * m12 - m02 * m10;
        let b02 = m00 * m13 - m03 * m10;
        let b03 = m01 * m12 - m02 * m11;
        let b04 = m01 * m13 - m03 * m11;
        let b05 = m02 * m13 - m03 * m12;
        let b06 = m20 * m31 - m21 * m30;
        let b07 = m20 * m32 - m22 * m30;
        let b08 = m20 * m33 - m23 * m30;
        let b09 = m21 * m32 - m22 * m31;
        let b10 = m21 * m33 - m23 * m31;
        let b11 = m22 * m33 - m23 * m32;
        b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06
    }

    /// `invert`.
    pub fn invert(&mut self) -> &mut Self {
        let (a00, a01, a02, a03) = (self.m00, self.m01, self.m02, self.m03);
        let (a10, a11, a12, a13) = (self.m10, self.m11, self.m12, self.m13);
        let (a20, a21, a22, a23) = (self.m20, self.m21, self.m22, self.m23);
        let (a30, a31, a32, a33) = (self.m30, self.m31, self.m32, self.m33);
        let b00 = a00 * a11 - a01 * a10;
        let b01 = a00 * a12 - a02 * a10;
        let b02 = a00 * a13 - a03 * a10;
        let b03 = a01 * a12 - a02 * a11;
        let b04 = a01 * a13 - a03 * a11;
        let b05 = a02 * a13 - a03 * a12;
        let b06 = a20 * a31 - a21 * a30;
        let b07 = a20 * a32 - a22 * a30;
        let b08 = a20 * a33 - a23 * a30;
        let b09 = a21 * a32 - a22 * a31;
        let b10 = a21 * a33 - a23 * a31;
        let b11 = a22 * a33 - a23 * a32;
        let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
        if det == 0.0 {
            return self;
        }
        let idet = 1.0 / det;
        self.m00 = (a11 * b11 - a12 * b10 + a13 * b09) * idet;
        self.m01 = (a02 * b10 - a01 * b11 - a03 * b09) * idet;
        self.m02 = (a31 * b05 - a32 * b04 + a33 * b03) * idet;
        self.m03 = (a22 * b04 - a21 * b05 - a23 * b03) * idet;
        self.m10 = (a12 * b08 - a10 * b11 - a13 * b07) * idet;
        self.m11 = (a00 * b11 - a02 * b08 + a03 * b07) * idet;
        self.m12 = (a32 * b02 - a30 * b05 + a33 * b01) * idet;
        self.m13 = (a20 * b05 - a22 * b02 - a23 * b01) * idet;
        self.m20 = (a10 * b10 - a11 * b08 + a13 * b06) * idet;
        self.m21 = (a01 * b08 - a00 * b10 - a03 * b06) * idet;
        self.m22 = (a30 * b04 - a31 * b02 + a33 * b00) * idet;
        self.m23 = (a21 * b02 - a20 * b04 - a23 * b00) * idet;
        self.m30 = (a11 * b07 - a10 * b09 - a12 * b06) * idet;
        self.m31 = (a00 * b09 - a01 * b07 + a02 * b06) * idet;
        self.m32 = (a31 * b01 - a30 * b03 - a32 * b00) * idet;
        self.m33 = (a20 * b03 - a21 * b01 + a22 * b00) * idet;
        self
    }

    /// `invert(Matrix4fc)` — this = this⁻¹ * m.
    pub fn invert_mat(&mut self, m: &Matrix4f) -> &mut Self {
        let mut inv = *self;
        inv.invert();
        inv.mul_mat(m);
        *self = inv;
        self
    }

    /// `transpose`.
    pub fn transpose(&mut self) -> &mut Self {
        std::mem::swap(&mut self.m01, &mut self.m10);
        std::mem::swap(&mut self.m02, &mut self.m20);
        std::mem::swap(&mut self.m03, &mut self.m30);
        std::mem::swap(&mut self.m12, &mut self.m21);
        std::mem::swap(&mut self.m13, &mut self.m31);
        std::mem::swap(&mut self.m23, &mut self.m32);
        self
    }

    /// `transpose(Matrix4fc)` — this = thisᵀ * m.
    pub fn transpose_mat(&mut self, m: &Matrix4f) -> &mut Self {
        let mut t = *self;
        t.transpose();
        t.mul_mat(m);
        *self = t;
        self
    }

    // ---- multiplication -----------------------------------------------------------

    /// `mul(Matrix4fc)` — this = this * m.
    pub fn mul_mat(&mut self, m: &Matrix4f) -> &mut Self {
        let a00 = self.m00; let a01 = self.m01; let a02 = self.m02; let a03 = self.m03;
        let a10 = self.m10; let a11 = self.m11; let a12 = self.m12; let a13 = self.m13;
        let a20 = self.m20; let a21 = self.m21; let a22 = self.m22; let a23 = self.m23;
        let a30 = self.m30; let a31 = self.m31; let a32 = self.m32; let a33 = self.m33;
        // C[c][r] = sum_k A[k][r] * B[c][k]  (mXY = столбец X, строка Y)
        self.m00 = a00 * m.m00 + a10 * m.m01 + a20 * m.m02 + a30 * m.m03;
        self.m01 = a01 * m.m00 + a11 * m.m01 + a21 * m.m02 + a31 * m.m03;
        self.m02 = a02 * m.m00 + a12 * m.m01 + a22 * m.m02 + a32 * m.m03;
        self.m03 = a03 * m.m00 + a13 * m.m01 + a23 * m.m02 + a33 * m.m03;
        self.m10 = a00 * m.m10 + a10 * m.m11 + a20 * m.m12 + a30 * m.m13;
        self.m11 = a01 * m.m10 + a11 * m.m11 + a21 * m.m12 + a31 * m.m13;
        self.m12 = a02 * m.m10 + a12 * m.m11 + a22 * m.m12 + a32 * m.m13;
        self.m13 = a03 * m.m10 + a13 * m.m11 + a23 * m.m12 + a33 * m.m13;
        self.m20 = a00 * m.m20 + a10 * m.m21 + a20 * m.m22 + a30 * m.m23;
        self.m21 = a01 * m.m20 + a11 * m.m21 + a21 * m.m22 + a31 * m.m23;
        self.m22 = a02 * m.m20 + a12 * m.m21 + a22 * m.m22 + a32 * m.m23;
        self.m23 = a03 * m.m20 + a13 * m.m21 + a23 * m.m22 + a33 * m.m23;
        self.m30 = a00 * m.m30 + a10 * m.m31 + a20 * m.m32 + a30 * m.m33;
        self.m31 = a01 * m.m30 + a11 * m.m31 + a21 * m.m32 + a31 * m.m33;
        self.m32 = a02 * m.m30 + a12 * m.m31 + a22 * m.m32 + a32 * m.m33;
        self.m33 = a03 * m.m30 + a13 * m.m31 + a23 * m.m32 + a33 * m.m33;
        self
    }

    /// `mul(m, dest)`.
    pub fn mul_into(&self, m: &Matrix4f, dest: &mut Matrix4f) -> Matrix4f {
        let a00 = self.m00; let a01 = self.m01; let a02 = self.m02; let a03 = self.m03;
        let a10 = self.m10; let a11 = self.m11; let a12 = self.m12; let a13 = self.m13;
        let a20 = self.m20; let a21 = self.m21; let a22 = self.m22; let a23 = self.m23;
        let a30 = self.m30; let a31 = self.m31; let a32 = self.m32; let a33 = self.m33;
        dest.m00 = a00 * m.m00 + a10 * m.m01 + a20 * m.m02 + a30 * m.m03;
        dest.m01 = a01 * m.m00 + a11 * m.m01 + a21 * m.m02 + a31 * m.m03;
        dest.m02 = a02 * m.m00 + a12 * m.m01 + a22 * m.m02 + a32 * m.m03;
        dest.m03 = a03 * m.m00 + a13 * m.m01 + a23 * m.m02 + a33 * m.m03;
        dest.m10 = a00 * m.m10 + a10 * m.m11 + a20 * m.m12 + a30 * m.m13;
        dest.m11 = a01 * m.m10 + a11 * m.m11 + a21 * m.m12 + a31 * m.m13;
        dest.m12 = a02 * m.m10 + a12 * m.m11 + a22 * m.m12 + a32 * m.m13;
        dest.m13 = a03 * m.m10 + a13 * m.m11 + a23 * m.m12 + a33 * m.m13;
        dest.m20 = a00 * m.m20 + a10 * m.m21 + a20 * m.m22 + a30 * m.m23;
        dest.m21 = a01 * m.m20 + a11 * m.m21 + a21 * m.m22 + a31 * m.m23;
        dest.m22 = a02 * m.m20 + a12 * m.m21 + a22 * m.m22 + a32 * m.m23;
        dest.m23 = a03 * m.m20 + a13 * m.m21 + a23 * m.m22 + a33 * m.m23;
        dest.m30 = a00 * m.m30 + a10 * m.m31 + a20 * m.m32 + a30 * m.m33;
        dest.m31 = a01 * m.m30 + a11 * m.m31 + a21 * m.m32 + a31 * m.m33;
        dest.m32 = a02 * m.m30 + a12 * m.m31 + a22 * m.m32 + a32 * m.m33;
        dest.m33 = a03 * m.m30 + a13 * m.m31 + a23 * m.m32 + a33 * m.m33;
        *dest
    }

    // ---- transforms ---------------------------------------------------------------

    /// `mulPosition(Vector3fc)` — точка (w=1) с перспективным делением.
    pub fn transform_vec3(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        dest.x = self.m00 * v.x + self.m10 * v.y + self.m20 * v.z + self.m30;
        dest.y = self.m01 * v.x + self.m11 * v.y + self.m21 * v.z + self.m31;
        dest.z = self.m02 * v.x + self.m12 * v.y + self.m22 * v.z + self.m32;
        *dest
    }

    /// `mulProject` — то же что mulPosition.
    pub fn transform_project(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        let inv_w = 1.0 / (self.m03 * v.x + self.m13 * v.y + self.m23 * v.z + self.m33);
        dest.x = (self.m00 * v.x + self.m10 * v.y + self.m20 * v.z + self.m30) * inv_w;
        dest.y = (self.m01 * v.x + self.m11 * v.y + self.m21 * v.z + self.m31) * inv_w;
        dest.z = (self.m02 * v.x + self.m12 * v.y + self.m22 * v.z + self.m32) * inv_w;
        *dest
    }

    /// `mulDirection` — направление (w=0, без переноса).
    pub fn transform_direction(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        dest.x = self.m00 * v.x + self.m10 * v.y + self.m20 * v.z;
        dest.y = self.m01 * v.x + self.m11 * v.y + self.m21 * v.z;
        dest.z = self.m02 * v.x + self.m12 * v.y + self.m22 * v.z;
        *dest
    }

    /// `mul(Vector4fc, Vector4f)` — полный 4d.
    pub fn transform_vec4(&self, v: &Vector4f, dest: &mut Vector4f) -> Vector4f {
        dest.x = self.m00 * v.x + self.m10 * v.y + self.m20 * v.z + self.m30 * v.w;
        dest.y = self.m01 * v.x + self.m11 * v.y + self.m21 * v.z + self.m31 * v.w;
        dest.z = self.m02 * v.x + self.m12 * v.y + self.m22 * v.z + self.m32 * v.w;
        dest.w = self.m03 * v.x + self.m13 * v.y + self.m23 * v.z + self.m33 * v.w;
        *dest
    }

    /// `mulTransposePosition` — Mᵀ * (v,1) с делением.
    pub fn transform_transpose_vec3(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        let w = self.m03 * v.x + self.m13 * v.y + self.m23 * v.z + self.m33;
        let inv = if w != 0.0 { 1.0 / w } else { 1.0 };
        dest.x = (self.m00 * v.x + self.m01 * v.y + self.m02 * v.z + self.m03) * inv;
        dest.y = (self.m10 * v.x + self.m11 * v.y + self.m12 * v.z + self.m13) * inv;
        dest.z = (self.m20 * v.x + self.m21 * v.y + self.m22 * v.z + self.m23) * inv;
        *dest
    }

    /// `mulTransposeDirection` — Mᵀ * (v,0).
    pub fn transform_transpose_direction(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        dest.x = self.m00 * v.x + self.m01 * v.y + self.m02 * v.z;
        dest.y = self.m10 * v.x + self.m11 * v.y + self.m12 * v.z;
        dest.z = self.m20 * v.x + self.m21 * v.y + self.m22 * v.z;
        *dest
    }

    /// `mulPositionW` — w после умножения.
    pub fn transform_position_w(&self, v: &Vector3f) -> f32 {
        self.m03 * v.x + self.m13 * v.y + self.m23 * v.z + self.m33
    }

    /// `transformNormal` — инверсно-транспонированная 3x3 * v.
    pub fn transform_normal(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        let m3 = Matrix3f::new();
        let mut nm = m3;
        nm.set_normal_matrix(self);
        nm.transform_vec(v, dest);
        *dest
    }

    // ---- builders -----------------------------------------------------------------

    /// `translate(x, y, z)` — this = this * T.
    pub fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        // translation column += linear part * t (JOML `translate`)
        self.m30 = self.m00 * x + self.m10 * y + self.m20 * z + self.m30;
        self.m31 = self.m01 * x + self.m11 * y + self.m21 * z + self.m31;
        self.m32 = self.m02 * x + self.m12 * y + self.m22 * z + self.m32;
        self.m33 = self.m03 * x + self.m13 * y + self.m23 * z + self.m33;
        self
    }

    /// `translate(Vector3fc)`.
    pub fn translate_vec(&mut self, v: &Vector3f) -> &mut Self {
        self.translate(v.x, v.y, v.z)
    }

    /// `scale(sx, sy, sz)` — this = this * S.
    pub fn scale(&mut self, sx: f32, sy: f32, sz: f32) -> &mut Self {
        self.m00 *= sx;
        self.m01 *= sx;
        self.m02 *= sx;
        self.m03 *= sx;
        self.m10 *= sy;
        self.m11 *= sy;
        self.m12 *= sy;
        self.m13 *= sy;
        self.m20 *= sz;
        self.m21 *= sz;
        self.m22 *= sz;
        self.m23 *= sz;
        self
    }

    /// `scale(s)` — равномерный.
    pub fn scale_uniform(&mut self, s: f32) -> &mut Self {
        self.scale(s, s, s)
    }

    /// `scale(Vector3fc)`.
    pub fn scale_vec(&mut self, v: &Vector3f) -> &mut Self {
        self.scale(v.x, v.y, v.z)
    }

    /// `rotate(angle, axisX, axisY, axisZ)` — this = this * R.
    pub fn rotate(&mut self, angle: f32, axis_x: f32, axis_y: f32, axis_z: f32) -> &mut Self {
        let mut m = Self::identity();
        m.set_from_axis_angle(angle, axis_x, axis_y, axis_z);
        self.mul_mat(&m);
        self
    }

    /// `rotate(angle, axis)`.
    pub fn rotate_axis(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.rotate(angle, axis.x, axis.y, axis.z)
    }

    /// `rotateX/Y/Z(angle)`.
    pub fn rotate_x(&mut self, angle: f32) -> &mut Self {
        self.rotate(angle, 1.0, 0.0, 0.0)
    }
    /// `rotateY`.
    pub fn rotate_y(&mut self, angle: f32) -> &mut Self {
        self.rotate(angle, 0.0, 1.0, 0.0)
    }
    /// `rotateZ`.
    pub fn rotate_z(&mut self, angle: f32) -> &mut Self {
        self.rotate(angle, 0.0, 0.0, 1.0)
    }

    /// `rotate(Quaternionfc)` — this = this * R(q).
    pub fn rotate_quat(&mut self, q: &Quaternionf) -> &mut Self {
        let mut m = Self::identity();
        m.set_from_quaternion(q);
        self.mul_mat(&m);
        self
    }

    /// `rotateAround(angle, axisX, axisY, axisZ, pointX, pointY, pointZ)` —
    /// поворот вокруг произвольной точки.
    pub fn rotate_around(
        &mut self,
        angle: f32,
        axis_x: f32,
        axis_y: f32,
        axis_z: f32,
        point_x: f32,
        point_y: f32,
        point_z: f32,
    ) -> &mut Self {
        // T(p) * R * T(-p)
        let mut m = Self::identity();
        m.set_from_axis_angle(angle, axis_x, axis_y, axis_z);
        // translate by -p, rotate, translate by p — через композицию:
        // M = T(p) * R * T(-p) * this? В JOML rotateAround: this =
        // this * T(p) * R * T(-p)
        let mut t1 = Self::translation(-point_x, -point_y, -point_z);
        let mut t2 = Self::translation(point_x, point_y, point_z);
        // R уже в m; собираем T(p)*R*T(-p)
        let mut rot = t2;
        rot.mul_mat(&m);
        rot.mul_mat(&t1);
        self.mul_mat(&rot);
        self
    }

    /// `setFromAxisAngleRad(angle, x, y, z)` — перезаписывает матрицу.
    pub fn set_from_axis_angle(&mut self, angle: f32, x: f32, y: f32, z: f32) -> &mut Self {
        *self = Self::identity();
        let (s, c) = Math::sin_cos(angle);
        let (x, y, z) = (x, y, z);
        let t = 1.0 - c;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        self.m00 = c + x * x * t;
        self.m10 = xy * t - z * s;
        self.m20 = xz * t + y * s;
        self.m30 = 0.0;
        self.m01 = xy * t + z * s;
        self.m11 = c + y * y * t;
        self.m21 = yz * t - x * s;
        self.m31 = 0.0;
        self.m02 = xz * t - y * s;
        self.m12 = yz * t + x * s;
        self.m22 = c + z * z * t;
        self.m32 = 0.0;
        self.m03 = 0.0;
        self.m13 = 0.0;
        self.m23 = 0.0;
        self.m33 = 1.0;
        self
    }

    /// `setFromAxisAngleDeg`.
    pub fn set_from_axis_angle_deg(&mut self, angle_deg: f32, x: f32, y: f32, z: f32) -> &mut Self {
        self.set_from_axis_angle(Math::to_radians(angle_deg), x, y, z)
    }

    /// `setFromQuaternion(Quaternionfc)` — матрица поворота из кватерниона.
    pub fn set_from_quaternion(&mut self, q: &Quaternionf) -> &mut Self {
        let (x, y, z, w) = (q.x, q.y, q.z, q.w);
        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;
        self.m00 = 1.0 - 2.0 * (yy + zz);
        self.m01 = 2.0 * (xy + wz);
        self.m02 = 2.0 * (xz - wy);
        self.m03 = 0.0;
        self.m10 = 2.0 * (xy - wz);
        self.m11 = 1.0 - 2.0 * (xx + zz);
        self.m12 = 2.0 * (yz + wx);
        self.m13 = 0.0;
        self.m20 = 2.0 * (xz + wy);
        self.m21 = 2.0 * (yz - wx);
        self.m22 = 1.0 - 2.0 * (xx + yy);
        self.m23 = 0.0;
        self.m30 = 0.0;
        self.m31 = 0.0;
        self.m32 = 0.0;
        self.m33 = 1.0;
        self
    }

    /// `setFromNormalized(Quaternionfc)` — то же.
    pub fn set_from_normalized_quaternion(&mut self, q: &Quaternionf) -> &mut Self {
        self.set_from_quaternion(q)
    }

    /// `set(AxisAngle4f)` — матрица поворота, ось нормализуется (JOML).
    pub fn set_axis_angle(&mut self, aa: &AxisAngle4f) -> &mut Self {
        let n = 1.0 / Math::sqrt(aa.x * aa.x + aa.y * aa.y + aa.z * aa.z);
        self.set_from_axis_angle(aa.angle, aa.x * n, aa.y * n, aa.z * n)
    }

    /// `toMatrix3f` — верхний левый 3×3.
    pub fn to_matrix3f(&self) -> Matrix3f {
        Matrix3f {
            m00: self.m00,
            m10: self.m10,
            m20: self.m20,
            m01: self.m01,
            m11: self.m11,
            m21: self.m21,
            m02: self.m02,
            m12: self.m12,
            m22: self.m22,
        }
    }

    /// `set(Matrix3fc)` — 3×3 в углу + identity остальное? В JOML
    /// set(Matrix3fc) записывает 3x3 блок и ставит affine identity.
    pub fn set_matrix3(&mut self, m: &Matrix3f) -> &mut Self {
        self.m00 = m.m00;
        self.m10 = m.m10;
        self.m20 = m.m20;
        self.m30 = 0.0;
        self.m01 = m.m01;
        self.m11 = m.m11;
        self.m21 = m.m21;
        self.m31 = 0.0;
        self.m02 = m.m02;
        self.m12 = m.m12;
        self.m22 = m.m22;
        self.m32 = 0.0;
        self.m03 = 0.0;
        self.m13 = 0.0;
        self.m23 = 0.0;
        self.m33 = 1.0;
        self
    }

    /// `setTranslationRotationScale`? — пропущены редкие билдеры,
    /// добавляются по мере необходимости порта.

    // ---- projections ----------------------------------------------------------------

    /// `perspective(fovy, aspect, zNear, zFar)` — GL-перспектива (JOML-стиль,
    /// column-major, [-1,1] depth как у JOML).
    pub fn set_perspective(&mut self, fovy: f32, aspect: f32, z_near: f32, z_far: f32) -> &mut Self {
        *self = Self::zero();
        let far = z_far;
        let near = z_near;
        let f = 1.0 / Math::tan(fovy * 0.5);
        self.m00 = f / aspect;
        self.m11 = f;
        self.m22 = (far + near) / (near - far);
        self.m23 = -1.0;
        self.m32 = 2.0 * far * near / (near - far);
        self.m33 = 0.0;
        self
    }

    /// `perspectiveLH`/`RH` варианты — JOML имеет их; добавим базовый
    /// (JOML perspective = RH по умолчанию? JOML `perspective` —
    /// right-handed, near/far positive forward -z).

    /// `ortho(left, right, bottom, top, near, far)`.
    pub fn set_ortho(&mut self, left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> &mut Self {
        *self = Self::zero();
        self.m00 = 2.0 / (right - left);
        self.m11 = 2.0 / (top - bottom);
        self.m22 = -2.0 / (far - near);
        self.m30 = -(right + left) / (right - left);
        self.m31 = -(top + bottom) / (top - bottom);
        self.m32 = -(far + near) / (far - near);
        self.m33 = 1.0;
        self
    }

    /// `ortho2D` — 2D-ортографическая (0..width, 0..height).
    pub fn set_ortho_2d(&mut self, left: f32, right: f32, bottom: f32, top: f32) -> &mut Self {
        *self = Self::zero();
        self.m00 = 2.0 / (right - left);
        self.m11 = 2.0 / (top - bottom);
        self.m22 = -1.0;
        self.m30 = -(right + left) / (right - left);
        self.m31 = -(top + bottom) / (top - bottom);
        self.m33 = 1.0;
        self
    }

    /// `frustum(left, right, bottom, top, near, far)`.
    pub fn set_frustum(&mut self, left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> &mut Self {
        *self = Self::zero();
        self.m00 = 2.0 * near / (right - left);
        self.m11 = 2.0 * near / (top - bottom);
        self.m20 = (right + left) / (right - left);
        self.m21 = (top + bottom) / (top - bottom);
        self.m22 = -(far + near) / (far - near);
        self.m32 = -2.0 * far * near / (far - near);
        self.m23 = -1.0;
        self
    }

    /// `setLookAt(eye, center, up)` — перезаписывает матрицу видом.
    /// JOML: dir = normalize(eye-center); left = normalize(up x dir);
    /// upn = dir x left; строки матрицы = left, upn, dir.
    pub fn set_look_at(&mut self, eye: &Vector3f, center: &Vector3f, up: &Vector3f) -> &mut Self {
        let mut dir = Vector3f::ZERO;
        dir.x = eye.x - center.x;
        dir.y = eye.y - center.y;
        dir.z = eye.z - center.z;
        dir.normalize();
        let mut left = Vector3f::ZERO;
        left.x = up.y * dir.z - up.z * dir.y;
        left.y = up.z * dir.x - up.x * dir.z;
        left.z = up.x * dir.y - up.y * dir.x;
        left.normalize();
        let upn = Vector3f::new(
            dir.y * left.z - dir.z * left.y,
            dir.z * left.x - dir.x * left.z,
            dir.x * left.y - dir.y * left.x,
        );
        self.m00 = left.x;
        self.m01 = upn.x;
        self.m02 = dir.x;
        self.m03 = 0.0;
        self.m10 = left.y;
        self.m11 = upn.y;
        self.m12 = dir.y;
        self.m13 = 0.0;
        self.m20 = left.z;
        self.m21 = upn.z;
        self.m22 = dir.z;
        self.m23 = 0.0;
        self.m30 = -(left.x * eye.x + left.y * eye.y + left.z * eye.z);
        self.m31 = -(upn.x * eye.x + upn.y * eye.y + upn.z * eye.z);
        self.m32 = -(dir.x * eye.x + dir.y * eye.y + dir.z * eye.z);
        self.m33 = 1.0;
        self
    }

    /// `lookAt(eye, center, up)` — this = this * L (постумножение).
    pub fn look_at(&mut self, eye: &Vector3f, center: &Vector3f, up: &Vector3f) -> &mut Self {
        let mut l = Self::identity();
        l.set_look_at(eye, center, up);
        self.mul_mat(&l);
        self
    }

    /// `lookAlong(dir, up, eye)` — камера смотрит вдоль dir.
    pub fn look_along(&mut self, dir: &Vector3f, up: &Vector3f, eye: &Vector3f) -> &mut Self {
        // center = eye + dir
        let mut center = *eye;
        center.add_vec(dir);
        self.look_at(eye, &center, up)
    }

    /// `equals` с delta.
    pub fn fuzzy_equals(&self, other: &Matrix4f, delta: f32) -> bool {
        let mut i = 0;
        while i < 16 {
            if (self.get_component(i) - other.get_component(i)).abs() > delta {
                return false;
            }
            i += 1;
        }
        true
    }

    /// `isFinite`.
    pub fn is_finite(&self) -> bool {
        self.m00.is_finite()
            && self.m01.is_finite()
            && self.m02.is_finite()
            && self.m03.is_finite()
            && self.m10.is_finite()
            && self.m11.is_finite()
            && self.m12.is_finite()
            && self.m13.is_finite()
            && self.m20.is_finite()
            && self.m21.is_finite()
            && self.m22.is_finite()
            && self.m23.is_finite()
            && self.m30.is_finite()
            && self.m31.is_finite()
            && self.m32.is_finite()
            && self.m33.is_finite()
    }

    /// `toString`.
    pub fn to_string_fmt(&self) -> String {
        let mut buf = [0.0f32; 16];
        self.write_slice(&mut buf);
        format!(
            "Matrix4f({} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {})",
            buf[0], buf[1], buf[2], buf[3],
            buf[4], buf[5], buf[6], buf[7],
            buf[8], buf[9], buf[10], buf[11],
            buf[12], buf[13], buf[14], buf[15],
        )
    }
}

impl Default for Matrix4f {
    fn default() -> Self {
        Self::new()
    }
}
