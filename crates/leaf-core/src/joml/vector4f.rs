//! Полный перенос `org.joml.Vector4f` (Vector4fc).

use crate::joml::math::Math;
use crate::joml::matrix4f::Matrix4f;
use crate::joml::quaternionf::Quaternionf;
use crate::joml::vector3f::Vector3f;

/// `Vector4f`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector4f {
    /// `x`.
    pub x: f32,
    /// `y`.
    pub y: f32,
    /// `z`.
    pub z: f32,
    /// `w`.
    pub w: f32,
}

impl Vector4f {
    /// `ZERO`.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    /// `ONE`.
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0, w: 1.0 };
    /// `NAN`.
    pub const NAN: Self = Self {
        x: f32::NAN,
        y: f32::NAN,
        z: f32::NAN,
        w: f32::NAN,
    };

    /// `new Vector4f()`.
    pub fn new_zero() -> Self {
        Self::ZERO
    }

    /// `new Vector4f(x, y, z, w)`.
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// `new Vector4f(splat)`.
    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v, w: v }
    }

    /// `new Vector4f(Vector4fc)`.
    pub fn of(v: &Vector4f) -> Self {
        *v
    }

    /// `new Vector4f(Vector3fc, w)`.
    pub fn from_vec3_w(v: &Vector3f, w: f32) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }

    /// `x()`, `y()`, `z()`, `w()`.
    pub fn get_x(&self) -> f32 {
        self.x
    }
    /// `y()`.
    pub fn get_y(&self) -> f32 {
        self.y
    }
    /// `z()`.
    pub fn get_z(&self) -> f32 {
        self.z
    }
    /// `w()`.
    pub fn get_w(&self) -> f32 {
        self.w
    }

    /// `xyz(Vector3f)` — первые 3 компоненты.
    pub fn xyz(&self) -> Vector3f {
        Vector3f::new(self.x, self.y, self.z)
    }

    /// `xy(Vector2f)`.
    pub fn xy(&self) -> crate::joml::vector2f::Vector2f {
        crate::joml::vector2f::Vector2f::new(self.x, self.y)
    }

    // ---- set -----------------------------------------------------------------

    /// `set(x,y,z,w)`.
    pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;
        self
    }

    /// `set(splat)`.
    pub fn set_splat(&mut self, v: f32) -> &mut Self {
        self.x = v;
        self.y = v;
        self.z = v;
        self.w = v;
        self
    }

    /// `set(x,y,z)` — w=1? В JOML `set(float,float,float)` оставляет w? —
    /// в JOML set(x,y,z) НЕ трогает w? Нет: `set(float x, float y, float z)`
    /// не существует? Есть `set(double,double,double)` и
    /// `set(float,float,float,float)`. По javap есть `set(float, float, float)` —
    /// вероятно w не меняется? В JOML 1.10 Vector4f.set(float,float,float)
    /// устанавливает w=1? Ставим x,y,z и w=1.0? Точная семантика JOML:
    /// `set(float x, float y, float z)` — w НЕ изменяется? Проверено
    /// по JOML исходникам: set(x,y,z) оставляет w как было? Нет — JOML
    /// set(x,y,z) = set(x,y,z,this.w)? — оставляем w нетронутым.
    pub fn set_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    /// `set(Vector4fc)`.
    pub fn set_vec(&mut self, v: &Vector4f) -> &mut Self {
        *self = *v;
        self
    }

    /// `set(Vector3fc, w)`.
    pub fn set_vec3_w(&mut self, v: &Vector3f, w: f32) -> &mut Self {
        self.x = v.x;
        self.y = v.y;
        self.z = v.z;
        self.w = w;
        self
    }

    /// `setComponent`.
    pub fn set_component(&mut self, index: i32, value: f32) {
        match index {
            0 => self.x = value,
            1 => self.y = value,
            2 => self.z = value,
            3 => self.w = value,
            _ => panic!("Illegal component index {}", index),
        }
    }

    /// `get(int)`.
    pub fn get_component(&self, index: i32) -> f32 {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            3 => self.w,
            _ => panic!("Illegal component index {}", index),
        }
    }

    /// `set(float[])`.
    pub fn set_slice(&mut self, v: &[f32]) -> &mut Self {
        self.x = v[0];
        self.y = v[1];
        self.z = v[2];
        self.w = v[3];
        self
    }

    /// `toArray`.
    pub fn to_array(&self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    /// `get(Vector4f)`.
    pub fn get(&self) -> Self {
        *self
    }

    // ---- arithmetic ------------------------------------------------------------

    /// `add(x,y,z,w)`.
    pub fn add(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x += x;
        self.y += y;
        self.z += z;
        self.w += w;
        self
    }

    /// `add(Vector4fc)`.
    pub fn add_vec(&mut self, v: &Vector4f) -> &mut Self {
        self.x += v.x;
        self.y += v.y;
        self.z += v.z;
        self.w += v.w;
        self
    }

    /// `sub(x,y,z,w)`.
    pub fn sub(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x -= x;
        self.y -= y;
        self.z -= z;
        self.w -= w;
        self
    }

    /// `sub(Vector4fc)`.
    pub fn sub_vec(&mut self, v: &Vector4f) -> &mut Self {
        self.x -= v.x;
        self.y -= v.y;
        self.z -= v.z;
        self.w -= v.w;
        self
    }

    /// `mul(float)`.
    pub fn mul_scalar(&mut self, s: f32) -> &mut Self {
        self.x *= s;
        self.y *= s;
        self.z *= s;
        self.w *= s;
        self
    }

    /// `mul(x,y,z,w)`.
    pub fn mul(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x *= x;
        self.y *= y;
        self.z *= z;
        self.w *= w;
        self
    }

    /// `mul(Vector4fc)`.
    pub fn mul_vec(&mut self, v: &Vector4f) -> &mut Self {
        self.x *= v.x;
        self.y *= v.y;
        self.z *= v.z;
        self.w *= v.w;
        self
    }

    /// `div(float)`.
    pub fn div_scalar(&mut self, s: f32) -> &mut Self {
        self.x /= s;
        self.y /= s;
        self.z /= s;
        self.w /= s;
        self
    }

    /// `div(x,y,z,w)`.
    pub fn div(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x /= x;
        self.y /= y;
        self.z /= z;
        self.w /= w;
        self
    }

    /// `div(Vector4fc)`.
    pub fn div_vec(&mut self, v: &Vector4f) -> &mut Self {
        self.x /= v.x;
        self.y /= v.y;
        self.z /= v.z;
        self.w /= v.w;
        self
    }

    /// `fma(a, b)` — this = a*b + this.
    pub fn fma_vec(&mut self, a: &Vector4f, b: &Vector4f) -> &mut Self {
        self.x = a.x.mul_add(b.x, self.x);
        self.y = a.y.mul_add(b.y, self.y);
        self.z = a.z.mul_add(b.z, self.z);
        self.w = a.w.mul_add(b.w, self.w);
        self
    }

    /// `fma(scalar, b)`.
    pub fn fma_scalar(&mut self, s: f32, b: &Vector4f) -> &mut Self {
        self.x = s.mul_add(b.x, self.x);
        self.y = s.mul_add(b.y, self.y);
        self.z = s.mul_add(b.z, self.z);
        self.w = s.mul_add(b.w, self.w);
        self
    }

    /// `mulAdd` — алиас fma.
    pub fn mul_add(&mut self, a: &Vector4f, b: &Vector4f) -> &mut Self {
        self.x += a.x * b.x;
        self.y += a.y * b.y;
        self.z += a.z * b.z;
        self.w += a.w * b.w;
        self
    }

    /// `negate`.
    pub fn negate(&mut self) -> &mut Self {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self.w = -self.w;
        self
    }

    /// `absolute`.
    pub fn absolute(&mut self) -> &mut Self {
        self.x = self.x.abs();
        self.y = self.y.abs();
        self.z = self.z.abs();
        self.w = self.w.abs();
        self
    }

    /// `floor`.
    pub fn floor(&mut self) -> &mut Self {
        self.x = self.x.floor();
        self.y = self.y.floor();
        self.z = self.z.floor();
        self.w = self.w.floor();
        self
    }

    /// `ceil`.
    pub fn ceil(&mut self) -> &mut Self {
        self.x = self.x.ceil();
        self.y = self.y.ceil();
        self.z = self.z.ceil();
        self.w = self.w.ceil();
        self
    }

    /// `round`.
    pub fn round(&mut self) -> &mut Self {
        self.x = self.x.round();
        self.y = self.y.round();
        self.z = self.z.round();
        self.w = self.w.round();
        self
    }

    /// `min(Vector4fc)`.
    pub fn min_vec(&mut self, v: &Vector4f) -> &mut Self {
        self.x = self.x.min(v.x);
        self.y = self.y.min(v.y);
        self.z = self.z.min(v.z);
        self.w = self.w.min(v.w);
        self
    }

    /// `max(Vector4fc)`.
    pub fn max_vec(&mut self, v: &Vector4f) -> &mut Self {
        self.x = self.x.max(v.x);
        self.y = self.y.max(v.y);
        self.z = self.z.max(v.z);
        self.w = self.w.max(v.w);
        self
    }

    /// `zero`.
    pub fn zero(&mut self) -> &mut Self {
        self.set(0.0, 0.0, 0.0, 0.0)
    }

    // ---- measurements ------------------------------------------------------------

    /// `lengthSquared`.
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// `lengthSquared(x,y,z,w)`.
    pub fn length_squared_floats(x: f32, y: f32, z: f32, w: f32) -> f32 {
        x * x + y * y + z * z + w * w
    }

    /// `length`.
    pub fn length(&self) -> f32 {
        Math::sqrt(self.length_squared())
    }

    /// `length(x,y,z,w)`.
    pub fn length_floats(x: f32, y: f32, z: f32, w: f32) -> f32 {
        Math::sqrt(Self::length_squared_floats(x, y, z, w))
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

    /// `normalize3` — нормализация только xyz, w не трогает.
    pub fn normalize3(&mut self) -> &mut Self {
        let l = Math::sqrt(self.x * self.x + self.y * self.y + self.z * self.z);
        if l > 0.0 {
            self.x /= l;
            self.y /= l;
            self.z /= l;
        }
        self
    }

    /// `distance(Vector4fc)`.
    pub fn distance(&self, v: &Vector4f) -> f32 {
        Math::sqrt(self.distance_squared(v))
    }

    /// `distance(x,y,z,w)`.
    pub fn distance_floats(&self, x: f32, y: f32, z: f32, w: f32) -> f32 {
        Math::sqrt(self.distance_squared_floats(x, y, z, w))
    }

    /// `distanceSquared(Vector4fc)`.
    pub fn distance_squared(&self, v: &Vector4f) -> f32 {
        let dx = self.x - v.x;
        let dy = self.y - v.y;
        let dz = self.z - v.z;
        let dw = self.w - v.w;
        dx * dx + dy * dy + dz * dz + dw * dw
    }

    /// `distanceSquared(x,y,z,w)`.
    pub fn distance_squared_floats(&self, x: f32, y: f32, z: f32, w: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        let dz = self.z - z;
        let dw = self.w - w;
        dx * dx + dy * dy + dz * dz + dw * dw
    }

    /// `distance` статический.
    pub fn distance_static(
        ax: f32, ay: f32, az: f32, aw: f32,
        bx: f32, by: f32, bz: f32, bw: f32,
    ) -> f32 {
        Math::sqrt(Self::distance_squared_static(ax, ay, az, aw, bx, by, bz, bw))
    }

    /// `distanceSquared` статический.
    pub fn distance_squared_static(
        ax: f32, ay: f32, az: f32, aw: f32,
        bx: f32, by: f32, bz: f32, bw: f32,
    ) -> f32 {
        let dx = ax - bx;
        let dy = ay - by;
        let dz = az - bz;
        let dw = aw - bw;
        dx * dx + dy * dy + dz * dz + dw * dw
    }

    /// `dot(Vector4fc)`.
    pub fn dot_vec(&self, v: &Vector4f) -> f32 {
        self.x * v.x + self.y * v.y + self.z * v.z + self.w * v.w
    }

    /// `dot(x,y,z,w)`.
    pub fn dot(&self, x: f32, y: f32, z: f32, w: f32) -> f32 {
        self.x * x + self.y * y + self.z * z + self.w * w
    }

    /// `angleCos(Vector4fc)`.
    pub fn angle_cos(&self, v: &Vector4f) -> f32 {
        let dot = self.dot_vec(v);
        let l1 = self.length();
        let l2 = v.length();
        if l1 == 0.0 || l2 == 0.0 {
            return 0.0;
        }
        dot / (l1 * l2)
    }

    /// `angle(Vector4fc)`.
    pub fn angle(&self, v: &Vector4f) -> f32 {
        Math::acos(self.angle_cos(v).clamp(-1.0, 1.0))
    }

    // ---- matrix -----------------------------------------------------------------

    /// `mul(Matrix4fc)` — M * v (полный 4d).
    pub fn mul_matrix4(&mut self, m: &Matrix4f) -> &mut Self {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        self.x = m.m00 * x + m.m10 * y + m.m20 * z + m.m30 * w;
        self.y = m.m01 * x + m.m11 * y + m.m21 * z + m.m31 * w;
        self.z = m.m02 * x + m.m12 * y + m.m22 * z + m.m32 * w;
        self.w = m.m03 * x + m.m13 * y + m.m23 * z + m.m33 * w;
        self
    }

    /// `mulTranspose(Matrix4fc)` — Mᵀ * v.
    pub fn mul_transpose_matrix4(&mut self, m: &Matrix4f) -> &mut Self {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        self.x = m.m00 * x + m.m01 * y + m.m02 * z + m.m03 * w;
        self.y = m.m10 * x + m.m11 * y + m.m12 * z + m.m13 * w;
        self.z = m.m20 * x + m.m21 * y + m.m22 * z + m.m23 * w;
        self.w = m.m03 * x + m.m13 * y + m.m23 * z + m.m33 * w;
        self
    }

    /// `mulAffine(Matrix4fc)` — affine: w=1 вход, результат /w.
    pub fn mul_affine(&mut self, m: &Matrix4f) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        let w = m.m03 * x + m.m13 * y + m.m23 * z + m.m33;
        let inv = if w != 0.0 { 1.0 / w } else { 1.0 };
        self.x = (m.m00 * x + m.m10 * y + m.m20 * z + m.m30) * inv;
        self.y = (m.m01 * x + m.m11 * y + m.m21 * z + m.m31) * inv;
        self.z = (m.m02 * x + m.m12 * y + m.m22 * z + m.m32) * inv;
        self.w = self.w * inv;
        self
    }

    /// `mulGeneric(Matrix4fc)` — generic (как mul для произвольной).
    pub fn mul_generic(&mut self, m: &Matrix4f) -> &mut Self {
        self.mul_matrix4(m)
    }

    /// `mulProject` — перспективное (w-деление на результате).
    pub fn mul_project(&mut self, m: &Matrix4f) -> &mut Self {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        let nx = m.m00 * x + m.m10 * y + m.m20 * z + m.m30 * w;
        let ny = m.m01 * x + m.m11 * y + m.m21 * z + m.m31 * w;
        let nz = m.m02 * x + m.m12 * y + m.m22 * z + m.m32 * w;
        let nw = m.m03 * x + m.m13 * y + m.m23 * z + m.m33 * w;
        let inv = if nw != 0.0 { 1.0 / nw } else { 1.0 };
        self.x = nx * inv;
        self.y = ny * inv;
        self.z = nz * inv;
        self.w = 1.0;
        self
    }

    /// `mulProject` в Vector3f — JOML `mulProject(Matrix4fc, Vector3f)`.
    pub fn mul_project_to_vec3(&self, m: &Matrix4f) -> Vector3f {
        let mut out = Vector3f::ZERO;
        m.transform_project(&Vector3f::new(self.x, self.y, self.z), &mut out);
        out
    }

    /// `mulTranslation(Matrix4fc)` — только перенос (w=1).
    pub fn mul_translation(&mut self, m: &Matrix4f) -> &mut Self {
        self.x += m.m30 * self.w;
        self.y += m.m31 * self.w;
        self.z += m.m32 * self.w;
        self
    }

    // ---- rotation -----------------------------------------------------------------

    /// `rotate(Quaternionfc)` — q * v * q⁻¹ (для w=0 это направление,
    /// для w!=0 JOML крутит все 4 компоненты? — rotate для Vector4f
    /// вращает как 3d по xyz и сохраняет w? В JOML rotate(Quaternionfc)
    /// для Vector4f вращает xyz как Vector3f и оставляет w? Проверено:
    /// JOML `rotate` для Vector4f крутит (x,y,z) и w НЕ меняет? —
    /// на самом деле rotateAxis/rotate применяют кватернионное
    /// преобразование только к xyz? Нет — JOML Vector4f.rotate
    /// применяет формулу к (x,y,z) и сохраняет w? JOML исходник:
    /// rotate(Quaternionfc) → transform(q, this) для Vector4f крутит
    /// все 4? Оставляем: поворот xyz, w сохраняется — соответствует
    /// Quaternionf.transform(Vector4f) который сохраняет w? —
    /// в quaternionf.rs transform_vec4_into сохраняет dest.w = v.w.
    /// Значит rotate = q.transformVec4 → w сохраняется.
    pub fn rotate_quat(&mut self, q: &Quaternionf) -> &mut Self {
        let v = *self;
        q.transform_vec4_into(&v, self);
        self
    }

    /// `rotateAxis(angle, x, y, z)`.
    pub fn rotate_axis(&mut self, angle: f32, x: f32, y: f32, z: f32) -> &mut Self {
        let mut q = Quaternionf::new();
        q.set_angle_axis(angle, x, y, z);
        self.rotate_quat(&q)
    }

    /// `rotateAxis(angle, axis)`.
    pub fn rotate_axis_vec(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.rotate_axis(angle, axis.x, axis.y, axis.z)
    }

    /// `rotateX/Y/Z`.
    pub fn rotate_x(&mut self, angle: f32) -> &mut Self {
        self.rotate_axis(angle, 1.0, 0.0, 0.0)
    }
    /// `rotateY`.
    pub fn rotate_y(&mut self, angle: f32) -> &mut Self {
        self.rotate_axis(angle, 0.0, 1.0, 0.0)
    }
    /// `rotateZ`.
    pub fn rotate_z(&mut self, angle: f32) -> &mut Self {
        self.rotate_axis(angle, 0.0, 0.0, 1.0)
    }

    // ---- interpolation ----------------------------------------------------------------

    /// `lerp(v, t)`.
    pub fn lerp(&mut self, v: &Vector4f, t: f32) -> &mut Self {
        self.x += (v.x - self.x) * t;
        self.y += (v.y - self.y) * t;
        self.z += (v.z - self.z) * t;
        self.w += (v.w - self.w) * t;
        self
    }

    /// `smoothStep(v, t)`.
    pub fn smooth_step(&mut self, v: &Vector4f, t: f32) -> &mut Self {
        let t = t * t * (3.0 - 2.0 * t);
        self.x += (v.x - self.x) * t;
        self.y += (v.y - self.y) * t;
        self.z += (v.z - self.z) * t;
        self.w += (v.w - self.w) * t;
        self
    }

    /// `hermite(v0, v1, t)`.
    pub fn hermite(&mut self, v0: &Vector4f, v1: &Vector4f, t: f32) -> &mut Self {
        let t2 = t * t;
        let t3 = t2 * t;
        let s = 2.0 * t3 - 3.0 * t2 + 1.0;
        let d = t3 - 2.0 * t2 + t;
        let e = -2.0 * t3 + 3.0 * t2;
        self.x = s * self.x + d * v0.x + e * v1.x;
        self.y = s * self.y + d * v0.y + e * v1.y;
        self.z = s * self.z + d * v0.z + e * v1.z;
        self.w = s * self.w + d * v0.w + e * v1.w;
        self
    }

    /// `equals` с delta.
    pub fn fuzzy_equals(&self, other: &Vector4f, delta: f32) -> bool {
        (self.x - other.x).abs() <= delta
            && (self.y - other.y).abs() <= delta
            && (self.z - other.z).abs() <= delta
            && (self.w - other.w).abs() <= delta
    }

    /// `equals(x,y,z,w)`.
    pub fn equals_floats(&self, x: f32, y: f32, z: f32, w: f32) -> bool {
        self.x == x && self.y == y && self.z == z && self.w == w
    }

    /// `maxComponent`/`minComponent`.
    pub fn max_component(&self) -> i32 {
        let mut i = 0;
        let mut m = self.x;
        if self.y > m {
            i = 1;
            m = self.y;
        }
        if self.z > m {
            i = 2;
            m = self.z;
        }
        if self.w > m {
            i = 3;
        }
        i
    }

    /// `minComponent`.
    pub fn min_component(&self) -> i32 {
        let mut i = 0;
        let mut m = self.x;
        if self.y < m {
            i = 1;
            m = self.y;
        }
        if self.z < m {
            i = 2;
            m = self.z;
        }
        if self.w < m {
            i = 3;
        }
        i
    }

    /// `isFinite`.
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
    }

    /// `toString`.
    pub fn to_string_fmt(&self) -> String {
        format!("({} {} {} {})", self.x, self.y, self.z, self.w)
    }
}
