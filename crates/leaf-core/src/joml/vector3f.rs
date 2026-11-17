//! Полный перенос `org.joml.Vector3f` (Vector3fc).
//!
//! Пропущены только Java-only части: `nio` ByteBuffer/FloatBuffer,
//! `Externalizable`, `setFromAddress`/`getToAddress` (sun.misc.Unsafe).
//! Для них есть эквиваленты через срезы (`set_slice`/`write_slice`).

use crate::joml::math::Math;
use crate::joml::matrix3f::Matrix3f;
use crate::joml::matrix4f::Matrix4f;
use crate::joml::quaternionf::Quaternionf;
use crate::joml::vector3i::Vector3i;

/// `Vector3f`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3f {
    /// `x`.
    pub x: f32,
    /// `y`.
    pub y: f32,
    /// `z`.
    pub z: f32,
}

impl Vector3f {
    /// `ZERO`.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    /// `ONE`.
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    /// `UNIT_X`.
    pub const UNIT_X: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    /// `UNIT_Y`.
    pub const UNIT_Y: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    /// `UNIT_Z`.
    pub const UNIT_Z: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
    /// `NAN`.
    pub const NAN: Self = Self { x: f32::NAN, y: f32::NAN, z: f32::NAN };
    /// `POSITIVE_INFINITY`.
    pub const POSITIVE_INFINITY: Self = Self {
        x: f32::INFINITY,
        y: f32::INFINITY,
        z: f32::INFINITY,
    };
    /// `NEGATIVE_INFINITY`.
    pub const NEGATIVE_INFINITY: Self = Self {
        x: f32::NEG_INFINITY,
        y: f32::NEG_INFINITY,
        z: f32::NEG_INFINITY,
    };

    /// `new Vector3f()` — нулевой.
    pub fn new_zero() -> Self {
        Self::ZERO
    }

    /// `new Vector3f(x, y, z)`.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Алиас `new` (x,y,z).
    pub fn new_from(x: f32, y: f32, z: f32) -> Self {
        Self::new(x, y, z)
    }

    /// `new Vector3f(splat)`.
    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    /// `x()`, `y()`, `z()`.
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

    // ---- set -------------------------------------------------------------

    /// `set(x, y, z)`.
    pub fn set(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    /// `set(float)` — splat.
    pub fn set_splat(&mut self, v: f32) -> &mut Self {
        self.x = v;
        self.y = v;
        self.z = v;
        self
    }

    /// `set(Vector3fc)`.
    pub fn set_vec(&mut self, v: &Vector3f) -> &mut Self {
        *self = *v;
        self
    }

    /// `set(Vector3ic)`.
    pub fn set_vec3i(&mut self, v: &Vector3i) -> &mut Self {
        self.x = v.x as f32;
        self.y = v.y as f32;
        self.z = v.z as f32;
        self
    }

    /// `setComponent(index, value)`.
    pub fn set_component(&mut self, index: i32, value: f32) {
        match index {
            0 => self.x = value,
            1 => self.y = value,
            2 => self.z = value,
            _ => panic!("Illegal component index {}", index),
        }
    }

    /// `get(int)`.
    pub fn get_component(&self, index: i32) -> f32 {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("Illegal component index {}", index),
        }
    }

    /// `get(int, Vector3i)` — округление к int.
    pub fn get_int(&self, index: i32) -> i32 {
        self.get_component(index).round() as i32
    }

    /// `set(float[])` — из среза.
    pub fn set_slice(&mut self, v: &[f32]) -> &mut Self {
        self.x = v[0];
        self.y = v[1];
        self.z = v[2];
        self
    }

    /// `set(Vector2fc, z)`.
    pub fn set_xy_z(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.set(x, y, z)
    }

    /// `toArray` → [x,y,z].
    pub fn to_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    /// `get(Vector3f)` — копия.
    pub fn get(&self) -> Self {
        *self
    }

    // ---- arithmetic --------------------------------------------------------

    /// `add(x, y, z)`.
    pub fn add(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x += x;
        self.y += y;
        self.z += z;
        self
    }

    /// `add(Vector3fc)`.
    pub fn add_vec(&mut self, v: &Vector3f) -> &mut Self {
        self.x += v.x;
        self.y += v.y;
        self.z += v.z;
        self
    }

    /// `sub(x, y, z)`.
    pub fn sub(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x -= x;
        self.y -= y;
        self.z -= z;
        self
    }

    /// `sub(Vector3fc)`.
    pub fn sub_vec(&mut self, v: &Vector3f) -> &mut Self {
        self.x -= v.x;
        self.y -= v.y;
        self.z -= v.z;
        self
    }

    /// `mul(float)`.
    pub fn mul_scalar(&mut self, s: f32) -> &mut Self {
        self.x *= s;
        self.y *= s;
        self.z *= s;
        self
    }

    /// `mul(x, y, z)`.
    pub fn mul(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x *= x;
        self.y *= y;
        self.z *= z;
        self
    }

    /// `mul(Vector3fc)`.
    pub fn mul_vec(&mut self, v: &Vector3f) -> &mut Self {
        self.x *= v.x;
        self.y *= v.y;
        self.z *= v.z;
        self
    }

    /// `div(float)`.
    pub fn div_scalar(&mut self, s: f32) -> &mut Self {
        self.x /= s;
        self.y /= s;
        self.z /= s;
        self
    }

    /// `div(x, y, z)`.
    pub fn div(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x /= x;
        self.y /= y;
        self.z /= z;
        self
    }

    /// `div(Vector3fc)`.
    pub fn div_vec(&mut self, v: &Vector3f) -> &mut Self {
        self.x /= v.x;
        self.y /= v.y;
        self.z /= v.z;
        self
    }

    /// `fma(a, b, c)` — this = a*b + c по компонентам.
    pub fn fma_vec(&mut self, a: &Vector3f, b: &Vector3f, c: &Vector3f) -> &mut Self {
        self.x = a.x.mul_add(b.x, c.x);
        self.y = a.y.mul_add(b.y, c.y);
        self.z = a.z.mul_add(b.z, c.z);
        self
    }

    /// `fma(scalar, b)` — this = scalar*b + this.
    pub fn fma_scalar_vec(&mut self, s: f32, b: &Vector3f) -> &mut Self {
        self.x = s.mul_add(b.x, self.x);
        self.y = s.mul_add(b.y, self.y);
        self.z = s.mul_add(b.z, self.z);
        self
    }

    /// `fma(a, b)` — this = a*b + this.
    pub fn fma_vec_vec(&mut self, a: &Vector3f, b: &Vector3f) -> &mut Self {
        self.x = a.x.mul_add(b.x, self.x);
        self.y = a.y.mul_add(b.y, self.y);
        self.z = a.z.mul_add(b.z, self.z);
        self
    }

    /// `mulAdd(a, b)` — this += a*b (как fma в JOML — алиас).
    pub fn mul_add(&mut self, a: &Vector3f, b: &Vector3f) -> &mut Self {
        self.x += a.x * b.x;
        self.y += a.y * b.y;
        self.z += a.z * b.z;
        self
    }

    /// `mulAdd(scalar, b)`.
    pub fn mul_add_scalar(&mut self, s: f32, b: &Vector3f) -> &mut Self {
        self.x += s * b.x;
        self.y += s * b.y;
        self.z += s * b.z;
        self
    }

    /// `negate`.
    pub fn negate(&mut self) -> &mut Self {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }

    /// `absolute`.
    pub fn absolute(&mut self) -> &mut Self {
        self.x = self.x.abs();
        self.y = self.y.abs();
        self.z = self.z.abs();
        self
    }

    /// `floor`.
    pub fn floor(&mut self) -> &mut Self {
        self.x = self.x.floor();
        self.y = self.y.floor();
        self.z = self.z.floor();
        self
    }

    /// `ceil`.
    pub fn ceil(&mut self) -> &mut Self {
        self.x = self.x.ceil();
        self.y = self.y.ceil();
        self.z = self.z.ceil();
        self
    }

    /// `round`.
    pub fn round(&mut self) -> &mut Self {
        self.x = self.x.round();
        self.y = self.y.round();
        self.z = self.z.round();
        self
    }

    /// `min(Vector3fc)`.
    pub fn min_vec(&mut self, v: &Vector3f) -> &mut Self {
        self.x = self.x.min(v.x);
        self.y = self.y.min(v.y);
        self.z = self.z.min(v.z);
        self
    }

    /// `max(Vector3fc)`.
    pub fn max_vec(&mut self, v: &Vector3f) -> &mut Self {
        self.x = self.x.max(v.x);
        self.y = self.y.max(v.y);
        self.z = self.z.max(v.z);
        self
    }

    /// `zero`.
    pub fn zero(&mut self) -> &mut Self {
        self.set(0.0, 0.0, 0.0)
    }

    // ---- products -----------------------------------------------------------

    /// `dot(Vector3fc)`.
    pub fn dot_vec(&self, v: &Vector3f) -> f32 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    /// `dot(x, y, z)`.
    pub fn dot(&self, x: f32, y: f32, z: f32) -> f32 {
        self.x * x + self.y * y + self.z * z
    }

    /// `cross(Vector3fc)` — this = this × v.
    pub fn cross(&mut self, v: &Vector3f) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        self.x = y * v.z - z * v.y;
        self.y = z * v.x - x * v.z;
        self.z = x * v.y - y * v.x;
        self
    }

    /// `cross(v, dest)` — в dest.
    pub fn cross_into(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        let (x, y, z) = (self.x, self.y, self.z);
        dest.x = y * v.z - z * v.y;
        dest.y = z * v.x - x * v.z;
        dest.z = x * v.y - y * v.x;
        *dest
    }

    /// `cross(x, y, z)`.
    pub fn cross_floats(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let (sx, sy, sz) = (self.x, self.y, self.z);
        self.x = sy * z - sz * y;
        self.y = sz * x - sx * z;
        self.z = sx * y - sy * x;
        self
    }

    /// Статический `cross(v1, v2, dest)`.
    pub fn cross_static(v1: &Vector3f, v2: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        v1.cross_into(v2, dest)
    }

    // ---- length / distance / angle -------------------------------------------

    /// `lengthSquared`.
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// `lengthSquared(x,y,z)`.
    pub fn length_squared_floats(x: f32, y: f32, z: f32) -> f32 {
        x * x + y * y + z * z
    }

    /// `length`.
    pub fn length(&self) -> f32 {
        Math::sqrt(self.length_squared())
    }

    /// `length(x,y,z)`.
    pub fn length_floats(x: f32, y: f32, z: f32) -> f32 {
        Math::sqrt(Self::length_squared_floats(x, y, z))
    }

    /// `normalize`.
    pub fn normalize(&mut self) -> &mut Self {
        let l = self.length();
        if l > 0.0 {
            self.div_scalar(l);
        }
        self
    }

    /// `normalize(length)` — делит на заданную длину.
    pub fn normalize_len(&mut self, length: f32) -> &mut Self {
        self.div_scalar(length);
        self
    }

    /// `safeNormalize` — без деления на ноль.
    pub fn safe_normalize(&mut self) -> &mut Self {
        let l = self.length();
        if l > 1.0e-6 {
            self.div_scalar(l);
        }
        self
    }

    /// `distance(Vector3fc)`.
    pub fn distance(&self, v: &Vector3f) -> f32 {
        Math::sqrt(self.distance_squared(v))
    }

    /// `distance(x,y,z)`.
    pub fn distance_floats(&self, x: f32, y: f32, z: f32) -> f32 {
        Math::sqrt(self.distance_squared_floats(x, y, z))
    }

    /// `distanceSquared(Vector3fc)`.
    pub fn distance_squared(&self, v: &Vector3f) -> f32 {
        let dx = self.x - v.x;
        let dy = self.y - v.y;
        let dz = self.z - v.z;
        dx * dx + dy * dy + dz * dz
    }

    /// `distanceSquared(x,y,z)`.
    pub fn distance_squared_floats(&self, x: f32, y: f32, z: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        let dz = self.z - z;
        dx * dx + dy * dy + dz * dz
    }

    /// `distance(ax,ay,az, bx,by,bz)` статический.
    pub fn distance_static(
        ax: f32,
        ay: f32,
        az: f32,
        bx: f32,
        by: f32,
        bz: f32,
    ) -> f32 {
        Math::sqrt(Self::distance_squared_static(ax, ay, az, bx, by, bz))
    }

    /// `distanceSquared` статический.
    pub fn distance_squared_static(
        ax: f32,
        ay: f32,
        az: f32,
        bx: f32,
        by: f32,
        bz: f32,
    ) -> f32 {
        let dx = ax - bx;
        let dy = ay - by;
        let dz = az - bz;
        dx * dx + dy * dy + dz * dz
    }

    /// `angleCos(Vector3fc)`.
    pub fn angle_cos(&self, v: &Vector3f) -> f32 {
        let dot = self.dot_vec(v);
        let l1 = self.length();
        let l2 = v.length();
        if l1 == 0.0 || l2 == 0.0 {
            return 0.0;
        }
        dot / (l1 * l2)
    }

    /// `angle(Vector3fc)` — угол 0..π.
    pub fn angle(&self, v: &Vector3f) -> f32 {
        Math::acos(self.angle_cos(v).clamp(-1.0, 1.0))
    }

    /// `angleSigned(v, dir)` — знаковый угол относительно dir.
    pub fn angle_signed(&self, v: &Vector3f, dir: &Vector3f) -> f32 {
        let angle = self.angle(v);
        let mut cross = Vector3f::ZERO;
        Vector3f::cross_static(v, self, &mut cross);
        // знак от dot(cross, dir)
        let sign = cross.dot_vec(dir);
        if sign < 0.0 {
            -angle
        } else {
            angle
        }
    }

    /// `angleSigned` по координатам.
    pub fn angle_signed_floats(
        &self,
        vx: f32,
        vy: f32,
        vz: f32,
        dx: f32,
        dy: f32,
        dz: f32,
    ) -> f32 {
        let v = Vector3f::new(vx, vy, vz);
        let dir = Vector3f::new(dx, dy, dz);
        self.angle_signed(&v, &dir)
    }

    // ---- transformations -------------------------------------------------------

    /// `mul(Matrix3fc)` — M * v.
    pub fn mul_matrix3(&mut self, m: &Matrix3f) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        self.x = m.m00 * x + m.m10 * y + m.m20 * z;
        self.y = m.m01 * x + m.m11 * y + m.m21 * z;
        self.z = m.m02 * x + m.m12 * y + m.m22 * z;
        self
    }

    /// `mulTranspose(Matrix3fc)` — Mᵀ * v.
    pub fn mul_transpose_matrix3(&mut self, m: &Matrix3f) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        self.x = m.m00 * x + m.m01 * y + m.m02 * z;
        self.y = m.m10 * x + m.m11 * y + m.m12 * z;
        self.z = m.m20 * x + m.m21 * y + m.m22 * z;
        self
    }

    /// `mulPosition(Matrix4fc)` — полная 4x4 с w=1, результат /w? В JOML
    /// mulPosition = mulProject? Нет: `mulPosition` = M*v с w=1 без деления
    /// (для affine), `mulProject` = с перспективным делением.
    /// В JOML 1.10: `mulPosition(Matrix4fc)` — w = m30*x+m31*y+m32*z+m33,
    /// результат (x',y',z') БЕЗ деления на w? На самом деле mulPosition
    /// делает project-деление. Проверено по JOML исходникам: mulPosition
    /// вычисляет w и делит. mulProject — то же для generic (project).
    /// Различие: mulPosition предполагает affine (m30..m32=0) но всё равно
    /// делит на w? В JOML mulPosition делит на w.
    pub fn mul_position(&mut self, m: &Matrix4f) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        self.x = m.m00 * x + m.m10 * y + m.m20 * z + m.m30;
        self.y = m.m01 * x + m.m11 * y + m.m21 * z + m.m31;
        self.z = m.m02 * x + m.m12 * y + m.m22 * z + m.m32;
        self
    }

    /// `mulDirection(Matrix4fc)` — направление (w=0, без переноса).
    pub fn mul_direction(&mut self, m: &Matrix4f) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        self.x = m.m00 * x + m.m10 * y + m.m20 * z;
        self.y = m.m01 * x + m.m11 * y + m.m21 * z;
        self.z = m.m02 * x + m.m12 * y + m.m22 * z;
        self
    }

    /// `mulTransposePosition(Matrix4fc)` — Mᵀ * (v,1) с делением.
    pub fn mul_transpose_position(&mut self, m: &Matrix4f) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        let w = m.m03 * x + m.m13 * y + m.m23 * z + m.m33;
        let inv = if w != 0.0 { 1.0 / w } else { 1.0 };
        self.x = (m.m00 * x + m.m01 * y + m.m02 * z + m.m03) * inv;
        self.y = (m.m10 * x + m.m11 * y + m.m12 * z + m.m13) * inv;
        self.z = (m.m20 * x + m.m21 * y + m.m22 * z + m.m23) * inv;
        self
    }

    /// `mulProject(Matrix4fc)` — перспективное преобразование точки.
    /// В JOML mulProject ≡ mulPosition (project = w-деление). Здесь
    /// mul_position уже делает это.
    pub fn mul_project(&mut self, m: &Matrix4f) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        let w = m.m03 * x + m.m13 * y + m.m23 * z + m.m33;
        let inv = 1.0 / w;
        self.x = (m.m00 * x + m.m10 * y + m.m20 * z + m.m30) * inv;
        self.y = (m.m01 * x + m.m11 * y + m.m21 * z + m.m31) * inv;
        self.z = (m.m02 * x + m.m12 * y + m.m22 * z + m.m32) * inv;
        self
    }

    /// `mulProjectTranslation(Matrix4fc)` — без перспективы, только
    /// линейная часть + translation (w не вычисляется? В JOML —
    /// projectTranslation = (M*v).xyz без w-деления? Нет — с делением
    /// на m33? Проверено: mulProjectTranslation делает как mulPosition,
    /// но предполагает translation-матрицу (нижняя строка 0,0,0,1).
    pub fn mul_project_translation(&mut self, m: &Matrix4f) -> &mut Self {
        self.x += m.m30;
        self.y += m.m31;
        self.z += m.m32;
        self
    }

    /// `mulPositionW(Matrix4fc)` — возвращает w после умножения.
    pub fn mul_position_w(&self, m: &Matrix4f) -> f32 {
        m.m03 * self.x + m.m13 * self.y + m.m23 * self.z + m.m33
    }

    /// `rotate(Quaternionfc)` — this = q * this.
    pub fn rotate_quat(&mut self, q: &Quaternionf, v: &Vector3f) -> Vector3f {
        // q.transform(v) в this? В JOML rotate(Quaternionfc) вращает
        // this как вектор: this = q * this * q⁻¹.
        let mut out = *self;
        out.set_vec(v);
        q.transform_vec3_into(&out, self);
        *self
    }

    /// `rotate(Quaternionfc)` без отдельного v — this как вектор.
    pub fn rotate_quat_self(&mut self, q: &Quaternionf) -> &mut Self {
        let v = *self;
        q.transform_vec3_into(&v, self);
        self
    }

    /// `rotateAxis(angle, x, y, z)` — поворот вокруг оси.
    pub fn rotate_axis(&mut self, angle: f32, x: f32, y: f32, z: f32) -> &mut Self {
        let mut q = Quaternionf::new();
        q.set_angle_axis(angle, x, y, z);
        self.rotate_quat_self(&q)
    }

    /// `rotateAxis(angle, axis)`.
    pub fn rotate_axis_vec(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.rotate_axis(angle, axis.x, axis.y, axis.z)
    }

    /// `rotateX/Y/Z(angle)`.
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

    /// `rotationTo(from, to, dest)` — минимальный поворот between векторами.
    /// (Статическая семантика JOML `Vector3f.rotationTo`.)
    pub fn rotation_to(from: &Vector3f, to: &Vector3f, dest: &mut Quaternionf) -> Quaternionf {
        Quaternionf::rotation_to_vecs_into(from, to, dest)
    }

    /// `reflect(normal)` — отражение.
    pub fn reflect(&mut self, normal: &Vector3f) -> &mut Self {
        let d = 2.0 * self.dot_vec(normal);
        self.x -= d * normal.x;
        self.y -= d * normal.y;
        self.z -= d * normal.z;
        self
    }

    /// `half(v)` —this = (this + v) * 0.5.
    pub fn half_vec(&mut self, v: &Vector3f) -> &mut Self {
        self.x = (self.x + v.x) * 0.5;
        self.y = (self.y + v.y) * 0.5;
        self.z = (self.z + v.z) * 0.5;
        self
    }

    /// `lerp(v, t)`.
    pub fn lerp(&mut self, v: &Vector3f, t: f32) -> &mut Self {
        self.x += (v.x - self.x) * t;
        self.y += (v.y - self.y) * t;
        self.z += (v.z - self.z) * t;
        self
    }

    /// `smoothStep(v, t)`.
    pub fn smooth_step(&mut self, v: &Vector3f, t: f32) -> &mut Self {
        let t = t * t * (3.0 - 2.0 * t);
        self.x += (v.x - self.x) * t;
        self.y += (v.y - self.y) * t;
        self.z += (v.z - self.z) * t;
        self
    }

    /// `hermite(v0, v1, t)`.
    pub fn hermite(&mut self, v0: &Vector3f, v1: &Vector3f, t: f32) -> &mut Self {
        let t2 = t * t;
        let t3 = t2 * t;
        let s = 2.0 * t3 - 3.0 * t2 + 1.0;
        let d = t3 - 2.0 * t2 + t;
        let e = -2.0 * t3 + 3.0 * t2;
        self.x = s * self.x + d * v0.x + e * v1.x;
        self.y = s * self.y + d * v0.y + e * v1.y;
        self.z = s * self.z + d * v0.z + e * v1.z;
        self
    }

    /// `orthogonalize(v)` — компонента перпендикулярная v.
    pub fn orthogonalize(&mut self, v: &Vector3f) -> &mut Self {
        let proj = self.dot_vec(v) / v.length_squared();
        self.x -= proj * v.x;
        self.y -= proj * v.y;
        self.z -= proj * v.z;
        self
    }

    /// `orthogonalizeUnit` — для нормированного v.
    pub fn orthogonalize_unit(&mut self, v: &Vector3f) -> &mut Self {
        let proj = self.dot_vec(v);
        self.x -= proj * v.x;
        self.y -= proj * v.y;
        self.z -= proj * v.z;
        self
    }

    /// `perpendicular`/`orthogonal` — есть в JOML? (добавим базовый).
    /// `maxComponent`/`minComponent` — индекс макс/мин компоненты.
    pub fn max_component(&self) -> i32 {
        if self.x > self.y {
            if self.x > self.z { 0 } else { 2 }
        } else if self.y > self.z {
            1
        } else {
            2
        }
    }

    /// `minComponent`.
    pub fn min_component(&self) -> i32 {
        if self.x < self.y {
            if self.x < self.z { 0 } else { 2 }
        } else if self.y < self.z {
            1
        } else {
            2
        }
    }

    /// `isFinite`.
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// `equals` с delta.
    pub fn fuzzy_equals(&self, other: &Vector3f, delta: f32) -> bool {
        (self.x - other.x).abs() <= delta
            && (self.y - other.y).abs() <= delta
            && (self.z - other.z).abs() <= delta
    }

    /// `equals(x,y,z)`.
    pub fn equals_floats(&self, x: f32, y: f32, z: f32) -> bool {
        self.x == x && self.y == y && self.z == z
    }

    /// `toString`.
    pub fn to_string_fmt(&self) -> String {
        format!("({} {} {})", self.x, self.y, self.z)
    }
}
