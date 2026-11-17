//! Полный перенос `org.joml.Matrix3f` (Matrix3fc) — матрица 3×3.

use crate::joml::math::Math;
use crate::joml::axis_angle4f::AxisAngle4f;
use crate::joml::quaternionf::Quaternionf;
use crate::joml::vector3f::Vector3f;

/// `Matrix3f` — column-major: `m00,m10,m20` — столбец 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix3f {
    /// `m00`, `m10`, `m20` — столбец 0.
    pub m00: f32,
    pub m10: f32,
    pub m20: f32,
    /// `m01`, `m11`, `m21` — столбец 1.
    pub m01: f32,
    pub m11: f32,
    pub m21: f32,
    /// `m02`, `m12`, `m22` — столбец 2.
    pub m02: f32,
    pub m12: f32,
    pub m22: f32,
}

impl Matrix3f {
    /// `new Matrix3f()` — identity.
    pub fn new() -> Self {
        Self::identity()
    }

    /// Все 9 компонент в порядке JOML (m00, m01, m02, m10, m11, m12, m20, m21, m22).
    #[allow(clippy::too_many_arguments)]
    pub fn from_m00_m01_m02_m10_m11_m12_m20_m21_m22(
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    ) -> Self {
        Self { m00, m01, m02, m10, m11, m12, m20, m21, m22 }
    }

    /// `new Matrix3f(Matrix3fc)`.
    pub fn of(m: &Matrix3f) -> Self {
        *m
    }

    /// `identity`.
    pub fn identity() -> Self {
        Self {
            m00: 1.0,
            m01: 0.0,
            m02: 0.0,
            m10: 0.0,
            m11: 1.0,
            m12: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
        }
    }

    /// `zero`.
    pub fn zero() -> Self {
        Self {
            m00: 0.0,
            m01: 0.0,
            m02: 0.0,
            m10: 0.0,
            m11: 0.0,
            m12: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 0.0,
        }
    }

    /// `set` всех компонент (порядок JOML).
    #[allow(clippy::too_many_arguments)]
    pub fn set(
        &mut self,
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    ) -> &mut Self {
        self.m00 = m00;
        self.m01 = m01;
        self.m02 = m02;
        self.m10 = m10;
        self.m11 = m11;
        self.m12 = m12;
        self.m20 = m20;
        self.m21 = m21;
        self.m22 = m22;
        self
    }

    /// `set(Matrix3fc)`.
    pub fn set_mat(&mut self, m: &Matrix3f) -> &mut Self {
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

    /// `set(float[])` — 9 элементов row-major как в JOML? JOML set(float[])
    /// читает в column-major порядке полей. Здесь — column-major срез.
    pub fn set_slice(&mut self, v: &[f32]) -> &mut Self {
        self.m00 = v[0];
        self.m10 = v[1];
        self.m20 = v[2];
        self.m01 = v[3];
        self.m11 = v[4];
        self.m21 = v[5];
        self.m02 = v[6];
        self.m12 = v[7];
        self.m22 = v[8];
        self
    }

    /// `get(int, float[])` — запись в срез column-major.
    pub fn write_slice(&self, out: &mut [f32]) {
        out[0] = self.m00;
        out[1] = self.m10;
        out[2] = self.m20;
        out[3] = self.m01;
        out[4] = self.m11;
        out[5] = self.m21;
        out[6] = self.m02;
        out[7] = self.m12;
        out[8] = self.m22;
    }

    /// `get(int)` — компонент по индексу (column-major).
    pub fn get_component(&self, index: i32) -> f32 {
        match index {
            0 => self.m00,
            1 => self.m10,
            2 => self.m20,
            3 => self.m01,
            4 => self.m11,
            5 => self.m21,
            6 => self.m02,
            7 => self.m12,
            8 => self.m22,
            _ => panic!("Illegal matrix index {}", index),
        }
    }

    /// `set(int, value)` — по индексу column-major.
    pub fn set_component(&mut self, index: i32, value: f32) {
        match index {
            0 => self.m00 = value,
            1 => self.m10 = value,
            2 => self.m20 = value,
            3 => self.m01 = value,
            4 => self.m11 = value,
            5 => self.m21 = value,
            6 => self.m02 = value,
            7 => self.m12 = value,
            8 => self.m22 = value,
            _ => panic!("Illegal matrix index {}", index),
        }
    }

    /// `determinant`.
    pub fn determinant(&self) -> f32 {
        (self.m00 * self.m11 - self.m01 * self.m10) * self.m22
            + (self.m02 * self.m10 - self.m00 * self.m12) * self.m21
            + (self.m01 * self.m12 - self.m02 * self.m11) * self.m20
    }

    /// `invert`.
    pub fn invert(&mut self) -> &mut Self {
        let det = self.determinant();
        if det == 0.0 {
            return self;
        }
        let inv_det = 1.0 / det;
        let (m00, m01, m02, m10, m11, m12, m20, m21, m22) = (
            self.m00, self.m01, self.m02, self.m10, self.m11, self.m12, self.m20, self.m21, self.m22,
        );
        // формула JOML invert(): a,b,c,d,s
        let a = m00 * m11 - m01 * m10;
        let b = m02 * m10 - m00 * m12;
        let c = m01 * m12 - m02 * m11;
        let s = inv_det;
        self.m00 = (m11 * m22 - m21 * m12) * s;
        self.m01 = (m20 * m12 - m10 * m22) * s;
        self.m02 = c * s;
        self.m10 = (m21 * m02 - m01 * m22) * s;
        self.m11 = (m00 * m22 - m20 * m02) * s;
        self.m12 = b * s;
        self.m20 = (m10 * m21 - m20 * m11) * s;
        self.m21 = (m20 * m01 - m00 * m21) * s;
        self.m22 = a * s;
        self
    }

    /// `invert(Matrix3fc)` — this = this⁻¹ * m.
    pub fn invert_mat(&mut self, m: &Matrix3f) -> &mut Self {
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
        std::mem::swap(&mut self.m12, &mut self.m21);
        self
    }

    /// `transpose(Matrix3fc)` — this = thisᵀ * m.
    pub fn transpose_mat(&mut self, m: &Matrix3f) -> &mut Self {
        let mut t = *self;
        t.transpose();
        t.mul_mat(m);
        *self = t;
        self
    }

    // ---- multiplication ---------------------------------------------------------

    /// `mul(Matrix3fc)` — this = this * m.
    pub fn mul_mat(&mut self, m: &Matrix3f) -> &mut Self {
        let (a00, a01, a02, a10, a11, a12, a20, a21, a22) = (
            self.m00, self.m01, self.m02, self.m10, self.m11, self.m12, self.m20, self.m21, self.m22,
        );
        // C[c][r] = sum_k A[k][r] * B[c][k]  (mXY = столбец X, строка Y)
        self.m00 = a00 * m.m00 + a10 * m.m01 + a20 * m.m02;
        self.m01 = a01 * m.m00 + a11 * m.m01 + a21 * m.m02;
        self.m02 = a02 * m.m00 + a12 * m.m01 + a22 * m.m02;
        self.m10 = a00 * m.m10 + a10 * m.m11 + a20 * m.m12;
        self.m11 = a01 * m.m10 + a11 * m.m11 + a21 * m.m12;
        self.m12 = a02 * m.m10 + a12 * m.m11 + a22 * m.m12;
        self.m20 = a00 * m.m20 + a10 * m.m21 + a20 * m.m22;
        self.m21 = a01 * m.m20 + a11 * m.m21 + a21 * m.m22;
        self.m22 = a02 * m.m20 + a12 * m.m21 + a22 * m.m22;
        self
    }

    /// `mul(m, dest)`.
    pub fn mul_into(&self, m: &Matrix3f, dest: &mut Matrix3f) -> Matrix3f {
        // порядок аргументов set(): m00,m01,m02,m10,m11,m12,m20,m21,m22
        dest.set(
            self.m00 * m.m00 + self.m10 * m.m01 + self.m20 * m.m02,
            self.m01 * m.m00 + self.m11 * m.m01 + self.m21 * m.m02,
            self.m02 * m.m00 + self.m12 * m.m01 + self.m22 * m.m02,
            self.m00 * m.m10 + self.m10 * m.m11 + self.m20 * m.m12,
            self.m01 * m.m10 + self.m11 * m.m11 + self.m21 * m.m12,
            self.m02 * m.m10 + self.m12 * m.m11 + self.m22 * m.m12,
            self.m00 * m.m20 + self.m10 * m.m21 + self.m20 * m.m22,
            self.m01 * m.m20 + self.m11 * m.m21 + self.m21 * m.m22,
            self.m02 * m.m20 + self.m12 * m.m21 + self.m22 * m.m22,
        );
        *dest
    }

    /// `mul(Vector3fc, Vector3f)` — M * v.
    pub fn transform_vec(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        dest.x = self.m00 * v.x + self.m10 * v.y + self.m20 * v.z;
        dest.y = self.m01 * v.x + self.m11 * v.y + self.m21 * v.z;
        dest.z = self.m02 * v.x + self.m12 * v.y + self.m22 * v.z;
        *dest
    }

    /// `mulTranspose(Vector3fc, Vector3f)` — Mᵀ * v.
    pub fn transform_transpose_vec(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        dest.x = self.m00 * v.x + self.m01 * v.y + self.m02 * v.z;
        dest.y = self.m10 * v.x + self.m11 * v.y + self.m12 * v.z;
        dest.z = self.m20 * v.x + self.m21 * v.y + self.m22 * v.z;
        *dest
    }

    /// `scale(sx, sy, sz)` — this = this * S.
    pub fn scale(&mut self, sx: f32, sy: f32, sz: f32) -> &mut Self {
        self.m00 *= sx;
        self.m01 *= sx;
        self.m02 *= sx;
        self.m10 *= sy;
        self.m11 *= sy;
        self.m12 *= sy;
        self.m20 *= sz;
        self.m21 *= sz;
        self.m22 *= sz;
        self
    }

    /// `scale(s)` — равномерный.
    pub fn scale_uniform(&mut self, s: f32) -> &mut Self {
        self.m00 *= s;
        self.m01 *= s;
        self.m02 *= s;
        self.m10 *= s;
        self.m11 *= s;
        self.m12 *= s;
        self.m20 *= s;
        self.m21 *= s;
        self.m22 *= s;
        self
    }

    /// `rotate(angle, axisX, axisY, axisZ)` — this = this * R(axis,angle).
    pub fn rotate(&mut self, angle: f32, axis_x: f32, axis_y: f32, axis_z: f32) -> &mut Self {
        let mut q = Quaternionf::new();
        q.set_angle_axis(angle, axis_x, axis_y, axis_z);
        let mut r = Matrix3f::new();
        q.get_matrix3(&mut r);
        self.mul_mat(&r);
        self
    }

    /// `rotate(angle, axis)`.
    pub fn rotate_axis(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.rotate(angle, axis.x, axis.y, axis.z)
    }

    /// `rotateX/Y/Z(angle)` — повороты вокруг осей.
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

    /// `setFromAxisAngleRad(axis, angle)` — матрица поворота вокруг оси.
    pub fn set_from_axis_angle(&mut self, axis: &Vector3f, angle: f32) -> &mut Self {
        let (s, c) = Math::sin_cos(angle);
        let (x, y, z) = (axis.x, axis.y, axis.z);
        let t = 1.0 - c;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        self.m00 = c + x * x * t;
        self.m10 = xy * t - z * s;
        self.m20 = xz * t + y * s;
        self.m01 = xy * t + z * s;
        self.m11 = c + y * y * t;
        self.m21 = yz * t - x * s;
        self.m02 = xz * t - y * s;
        self.m12 = yz * t + x * s;
        self.m22 = c + z * z * t;
        self
    }

    /// `setFromAxisAngleDeg`.
    pub fn set_from_axis_angle_deg(&mut self, axis: &Vector3f, angle_deg: f32) -> &mut Self {
        self.set_from_axis_angle(axis, Math::to_radians(angle_deg))
    }

    /// `setFromUnnormalized`/`setFromNormalized(Quaternionfc)` — через get.
    pub fn set_from_quaternion(&mut self, q: &Quaternionf) -> &mut Self {
        q.get_matrix3(self);
        self
    }

    /// `set(AxisAngle4f)` — матрица поворота, ось нормализуется (JOML).
    pub fn set_axis_angle(&mut self, aa: &AxisAngle4f) -> &mut Self {
        let n = Math::invsqrt(aa.x * aa.x + aa.y * aa.y + aa.z * aa.z);
        let axis = Vector3f::new(aa.x * n, aa.y * n, aa.z * n);
        self.set_from_axis_angle(&axis, aa.angle)
    }

    /// `fromBasis` — ортонормированный базис (столбцы).
    pub fn from_basis(x_axis: &Vector3f, y_axis: &Vector3f, z_axis: &Vector3f) -> Self {
        Self {
            m00: x_axis.x,
            m01: x_axis.y,
            m02: x_axis.z,
            m10: y_axis.x,
            m11: y_axis.y,
            m12: y_axis.z,
            m20: z_axis.x,
            m21: z_axis.y,
            m22: z_axis.z,
        }
    }

    /// `normal()` — эта матрица = транспонированная обратная (нормальная).
    /// JOML: normal matrix = transpose of inverse.
    pub fn normal(&mut self) -> &mut Self {
        self.invert();
        self.transpose();
        self
    }

    /// Нормальная матрица из верхнего 3×3 матрицы 4×4 (как JOML normal).
    pub fn set_normal_matrix(&mut self, m: &crate::joml::matrix4f::Matrix4f) -> &mut Self {
        self.m00 = m.m00;
        self.m01 = m.m01;
        self.m02 = m.m02;
        self.m10 = m.m10;
        self.m11 = m.m11;
        self.m12 = m.m12;
        self.m20 = m.m20;
        self.m21 = m.m21;
        self.m22 = m.m22;
        self.normal()
    }

    /// `normalize` — для матрицы нормалей (div по длине столбцов).
    /// В JOML normalize() нормирует столбцы для normal-матриц.
    pub fn normalize_columns(&mut self) -> &mut Self {
        // нормализация каждого столбца
        let l0 = Math::sqrt(self.m00 * self.m00 + self.m01 * self.m01 + self.m02 * self.m02);
        let l1 = Math::sqrt(self.m10 * self.m10 + self.m11 * self.m11 + self.m12 * self.m12);
        let l2 = Math::sqrt(self.m20 * self.m20 + self.m21 * self.m21 + self.m22 * self.m22);
        if l0 != 0.0 {
            self.m00 /= l0;
            self.m01 /= l0;
            self.m02 /= l0;
        }
        if l1 != 0.0 {
            self.m10 /= l1;
            self.m11 /= l1;
            self.m12 /= l1;
        }
        if l2 != 0.0 {
            self.m20 /= l2;
            self.m21 /= l2;
            self.m22 /= l2;
        }
        self
    }

    /// `equals` с delta.
    pub fn fuzzy_equals(&self, other: &Matrix3f, delta: f32) -> bool {
        (self.m00 - other.m00).abs() <= delta
            && (self.m01 - other.m01).abs() <= delta
            && (self.m02 - other.m02).abs() <= delta
            && (self.m10 - other.m10).abs() <= delta
            && (self.m11 - other.m11).abs() <= delta
            && (self.m12 - other.m12).abs() <= delta
            && (self.m20 - other.m20).abs() <= delta
            && (self.m21 - other.m21).abs() <= delta
            && (self.m22 - other.m22).abs() <= delta
    }

    /// `isFinite`.
    pub fn is_finite(&self) -> bool {
        self.m00.is_finite()
            && self.m01.is_finite()
            && self.m02.is_finite()
            && self.m10.is_finite()
            && self.m11.is_finite()
            && self.m12.is_finite()
            && self.m20.is_finite()
            && self.m21.is_finite()
            && self.m22.is_finite()
    }

    /// `toString`.
    pub fn to_string_fmt(&self) -> String {
        format!(
            "Matrix3f({} {} {} {} {} {} {} {} {})",
            self.m00, self.m01, self.m02, self.m10, self.m11, self.m12, self.m20, self.m21, self.m22
        )
    }
}

impl Default for Matrix3f {
    fn default() -> Self {
        Self::new()
    }
}
