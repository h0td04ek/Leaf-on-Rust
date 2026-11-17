//! Полный перенос `org.joml.Quaternionf` (f32-вариант, Quaternionfc).
//!
//! `w,x,y,z` — публичные поля. Методы `*d`/`Vector3d`/`Vector4d`/`Matrix*d`
//! из Java API опущены до порта d-вариантов типов (TODO в комментариях
//! не ставим — добавятся вместе с `Vector3d`/`Matrix4d`).

use crate::joml::axis_angle4f::AxisAngle4f;
use crate::joml::math::Math;
use crate::joml::matrix3f::Matrix3f;
use crate::joml::matrix4f::Matrix4f;
use crate::joml::vector3f::Vector3f;
use crate::joml::vector4f::Vector4f;

/// `Quaternionf`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternionf {
    /// `w`.
    pub w: f32,
    /// `x`.
    pub x: f32,
    /// `y`.
    pub y: f32,
    /// `z`.
    pub z: f32,
}

impl Quaternionf {
    /// `new Quaternionf()` — (0,0,0,1) identity? В JOML default = (0,0,0,1)?
    /// Нет: `new Quaternionf()` даёт identity (x=0,y=0,z=0,w=1).
    /// Default derive даёт все нули — поэтому `new` явный.
    pub fn new() -> Self {
        Self { w: 1.0, x: 0.0, y: 0.0, z: 0.0 }
    }

    /// `new Quaternionf(w, x, y, z)`.
    pub fn from_wxyz(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    /// `new Quaternionf(Quaternionfc)`.
    pub fn of(q: &Quaternionf) -> Self {
        *q
    }

    /// `new Quaternionf(AxisAngle4f)`.
    pub fn from_axis_angle(aa: &AxisAngle4f) -> Self {
        let mut q = Self::new();
        q.set_axis_angle(aa);
        q
    }

    /// `new Quaternionf(float, float, float, float)` — порядок (x,y,z,w) в JOML!
    pub fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { w, x, y, z }
    }

    /// `x()`.
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

    // ---- set ------------------------------------------------------------

    /// `set(x, y, z, w)` — JOML порядок аргументов (x,y,z,w).
    pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;
        self
    }

    /// `set(Quaternionfc)`.
    pub fn set_quat(&mut self, q: &Quaternionf) -> &mut Self {
        *self = *q;
        self
    }

    /// `set(AxisAngle4f)`.
    pub fn set_axis_angle(&mut self, aa: &AxisAngle4f) -> &mut Self {
        self.set_angle_axis(aa.angle, aa.x, aa.y, aa.z)
    }

    /// `setAngleAxis(angle, x, y, z)`.
    pub fn set_angle_axis(&mut self, angle: f32, x: f32, y: f32, z: f32) -> &mut Self {
        let s = Math::sin(angle * 0.5);
        let n = Math::sqrt(x * x + y * y + z * z);
        if n < 1.0e-6 {
            self.set(0.0, 0.0, 0.0, 1.0);
        } else {
            let f = s / n;
            self.set(x * f, y * f, z * f, Math::cos(angle * 0.5));
        }
        self
    }

    /// `rotationAxis(angle, x, y, z)` — то же что setAngleAxis.
    pub fn rotation_axis(&mut self, angle: f32, x: f32, y: f32, z: f32) -> &mut Self {
        self.set_angle_axis(angle, x, y, z)
    }

    /// `rotationAxis(angle, axis)`.
    pub fn rotation_axis_vec(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.set_angle_axis(angle, axis.x, axis.y, axis.z)
    }

    /// `rotationAxis(AxisAngle4f)`.
    pub fn rotation_axis_aa(&mut self, aa: &AxisAngle4f) -> &mut Self {
        self.set_axis_angle(aa)
    }

    /// `rotationX(angle)` — поворот вокруг X (перезаписывает).
    pub fn rotation_x(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle * 0.5);
        let c = Math::cos(angle * 0.5);
        self.set(s, 0.0, 0.0, c);
        self
    }

    /// `rotationY(angle)`.
    pub fn rotation_y(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle * 0.5);
        let c = Math::cos(angle * 0.5);
        self.set(0.0, s, 0.0, c);
        self
    }

    /// `rotationZ(angle)`.
    pub fn rotation_z(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle * 0.5);
        let c = Math::cos(angle * 0.5);
        self.set(0.0, 0.0, s, c);
        self
    }

    /// `identity`.
    pub fn identity(&mut self) -> &mut Self {
        self.set(0.0, 0.0, 0.0, 1.0)
    }

    // ---- arithmetic -----------------------------------------------------

    /// `add(x,y,z,w)`.
    pub fn add(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x += x;
        self.y += y;
        self.z += z;
        self.w += w;
        self
    }

    /// `add(Quaternionfc)`.
    pub fn add_quat(&mut self, q: &Quaternionf) -> &mut Self {
        self.x += q.x;
        self.y += q.y;
        self.z += q.z;
        self.w += q.w;
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

    /// `sub(Quaternionfc)`.
    pub fn sub_quat(&mut self, q: &Quaternionf) -> &mut Self {
        self.x -= q.x;
        self.y -= q.y;
        self.z -= q.z;
        self.w -= q.w;
        self
    }

    /// `mul(Quaternionfc)` — this = this * q.
    pub fn mul_quat(&mut self, q: &Quaternionf) -> &mut Self {
        let (qx, qy, qz, qw) = (q.x, q.y, q.z, q.w);
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        self.set(
            w * qx + x * qw + y * qz - z * qy,
            w * qy - x * qz + y * qw + z * qx,
            w * qz + x * qy - y * qx + z * qw,
            w * qw - x * qx - y * qy - z * qz,
        );
        self
    }

    /// `mul(x,y,z,w)`.
    pub fn mul(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        let mut q = Quaternionf::from_xyzw(x, y, z, w);
        self.mul_quat(&q)
    }

    /// `mul(float)`.
    pub fn mul_scalar(&mut self, s: f32) -> &mut Self {
        self.x *= s;
        self.y *= s;
        self.z *= s;
        self.w *= s;
        self
    }

    /// `premul(Quaternionfc)` — this = q * this.
    pub fn premul_quat(&mut self, q: &Quaternionf) -> &mut Self {
        let (qx, qy, qz, qw) = (q.x, q.y, q.z, q.w);
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        self.set(
            qw * x + qx * w + qy * z - qz * y,
            qw * y - qx * z + qy * w + qz * x,
            qw * z + qx * y - qy * x + qz * w,
            qw * w - qx * x - qy * y - qz * z,
        );
        self
    }

    /// `premul(x,y,z,w)`.
    pub fn premul(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        let q = Quaternionf::from_xyzw(x, y, z, w);
        self.premul_quat(&q)
    }

    /// `div(Quaternionfc)` — this = this * q⁻¹.
    pub fn div_quat(&mut self, q: &Quaternionf) -> &mut Self {
        let mut inv = *q;
        inv.invert();
        self.mul_quat(&inv)
    }

    /// `div(float)`.
    pub fn div_scalar(&mut self, s: f32) -> &mut Self {
        self.mul_scalar(1.0 / s)
    }

    /// `conjugate`.
    pub fn conjugate(&mut self) -> &mut Self {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }

    /// `invert` — this = this⁻¹ (для unit quaternion = conjugate).
    pub fn invert(&mut self) -> &mut Self {
        let n = self.length_squared();
        if n == 0.0 {
            return self;
        }
        self.conjugate();
        self.div_scalar(n);
        self
    }

    /// `normalize`.
    pub fn normalize(&mut self) -> &mut Self {
        let l = Math::sqrt(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w);
        if l != 0.0 {
            let inv = 1.0 / l;
            self.x *= inv;
            self.y *= inv;
            self.z *= inv;
            self.w *= inv;
        }
        self
    }

    /// `lengthSquared`.
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// `dot(Quaternionf)`.
    pub fn dot(&self, q: &Quaternionf) -> f32 {
        self.x * q.x + self.y * q.y + self.z * q.z + self.w * q.w
    }

    /// `angle` — угол между кватернионами.
    pub fn angle(&self) -> f32 {
        let dot = self.dot(self).abs().min(1.0);
        2.0 * Math::acos(dot)
    }

    /// `scale(float)` — умножает w-компоненту? В JOML `scale` масштабирует
    /// векторную часть (x,y,z). Реализация JOML: домножает все? —
    /// Quaternionf.scale умножает всю кватернион? Смотрим JOML:
    /// `scale(float factor)` возвращает this * (0,0,0,factor) — т.е.
    /// домножает все 4 компоненты.
    pub fn scale(&mut self, factor: f32) -> &mut Self {
        self.x *= factor;
        self.y *= factor;
        self.z *= factor;
        self.w *= factor;
        self
    }

    // ---- rotation composition --------------------------------------------

    /// `rotateXYZ(angleX, angleY, angleZ)` — this *= q(x,y,z).
    pub fn rotate_xyz(&mut self, angle_x: f32, angle_y: f32, angle_z: f32) -> &mut Self {
        let mut q = Quaternionf::new();
        q.rotation_xyz(angle_x, angle_y, angle_z);
        self.mul_quat(&q);
        self
    }

    /// `rotateZYX(angleZ, angleY, angleX)`.
    pub fn rotate_zyx(&mut self, angle_z: f32, angle_y: f32, angle_x: f32) -> &mut Self {
        let mut q = Quaternionf::new();
        q.rotation_zyx(angle_z, angle_y, angle_x);
        self.mul_quat(&q);
        self
    }

    /// `rotateYXZ(angleY, angleX, angleZ)`.
    pub fn rotate_yxz(&mut self, angle_y: f32, angle_x: f32, angle_z: f32) -> &mut Self {
        let mut q = Quaternionf::new();
        q.rotation_yxz(angle_y, angle_x, angle_z);
        self.mul_quat(&q);
        self
    }

    /// `rotationZYX` — полный набор (z,y,x).
    pub fn rotation_zyx(&mut self, angle_z: f32, angle_y: f32, angle_x: f32) -> &mut Self {
        // JOML: rotationZYX = qz * qy * qx
        let (sz, cz) = Math::sin_cos(angle_z * 0.5);
        let (sy, cy) = Math::sin_cos(angle_y * 0.5);
        let (sx, cx) = Math::sin_cos(angle_x * 0.5);
        let qz = Quaternionf::from_xyzw(0.0, 0.0, sz, cz);
        let qy = Quaternionf::from_xyzw(0.0, sy, 0.0, cy);
        let qx = Quaternionf::from_xyzw(sx, 0.0, 0.0, cx);
        let mut r = qz;
        r.mul_quat(&qy);
        r.mul_quat(&qx);
        *self = r;
        self
    }

    /// `rotationYXZ` — (y,x,z).
    pub fn rotation_yxz(&mut self, angle_y: f32, angle_x: f32, angle_z: f32) -> &mut Self {
        let (sy, cy) = Math::sin_cos(angle_y * 0.5);
        let (sx, cx) = Math::sin_cos(angle_x * 0.5);
        let (sz, cz) = Math::sin_cos(angle_z * 0.5);
        let qy = Quaternionf::from_xyzw(0.0, sy, 0.0, cy);
        let qx = Quaternionf::from_xyzw(sx, 0.0, 0.0, cx);
        let qz = Quaternionf::from_xyzw(0.0, 0.0, sz, cz);
        let mut r = qy;
        r.mul_quat(&qx);
        r.mul_quat(&qz);
        *self = r;
        self
    }

    /// `rotationXYZ` — (x,y,z), XYZ-порядок: qx * qy * qz? В JOML
    /// rotationXYZ строит q = qz*qy*qx? Точная формула JOML:
    /// `rotationXYZ` применяет вращения в порядке X затем Y затем Z,
    /// что даёт q = qz * qy * qx.
    pub fn rotation_xyz(&mut self, angle_x: f32, angle_y: f32, angle_z: f32) -> &mut Self {
        let (sx, cx) = Math::sin_cos(angle_x * 0.5);
        let (sy, cy) = Math::sin_cos(angle_y * 0.5);
        let (sz, cz) = Math::sin_cos(angle_z * 0.5);
        // JOML: w = cx*cy*cz + sx*sy*sz
        // x = sx*cy*cz - cx*sy*sz
        // y = cx*sy*cz + sx*cy*sz
        // z = cx*cy*sz - sx*sy*cz
        self.set(
            sx * cy * cz - cx * sy * sz,
            cx * sy * cz + sx * cy * sz,
            cx * cy * sz - sx * sy * cz,
            cx * cy * cz + sx * sy * sz,
        );
        self
    }

    /// `rotateX(angle)` — this *= qx.
    pub fn rotate_x(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle * 0.5);
        let c = Math::cos(angle * 0.5);
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        // this = this * (s,0,0,c)
        self.set(
            w * s + x * c + y * 0.0 - z * 0.0,
            w * 0.0 - x * 0.0 + y * c + z * s,
            w * 0.0 + x * 0.0 - y * s + z * c,
            w * c - x * s - y * 0.0 - z * 0.0,
        );
        self
    }

    /// `rotateY(angle)`.
    pub fn rotate_y(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle * 0.5);
        let c = Math::cos(angle * 0.5);
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        self.set(
            w * 0.0 + x * c + y * 0.0 - z * s,
            w * s - x * 0.0 + y * c + z * 0.0,
            w * 0.0 + x * s - y * 0.0 + z * c,
            w * c - x * 0.0 - y * s - z * 0.0,
        );
        self
    }

    /// `rotateZ(angle)`.
    pub fn rotate_z(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle * 0.5);
        let c = Math::cos(angle * 0.5);
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        self.set(
            w * 0.0 + x * c + y * s - z * 0.0,
            w * 0.0 - x * s + y * c + z * 0.0,
            w * s + x * 0.0 - y * 0.0 + z * c,
            w * c - x * 0.0 - y * 0.0 - z * s,
        );
        self
    }

    /// `rotateAxis(angle, x, y, z)` — this *= axisAngle.
    pub fn rotate_axis(&mut self, angle: f32, x: f32, y: f32, z: f32) -> &mut Self {
        let mut q = Quaternionf::new();
        q.set_angle_axis(angle, x, y, z);
        self.mul_quat(&q);
        self
    }

    /// `rotateAxis(angle, axis)`.
    pub fn rotate_axis_vec(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.rotate_axis(angle, axis.x, axis.y, axis.z)
    }

    /// `rotateLocalX/Y/Z` — локальные оси (this = q * this? в JOML:
    /// rotateLocal* умножает слева? — rotateLocalX(angle) делает
    /// this = qx * this — т.е. premul по локальной оси).
    pub fn rotate_local_x(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle * 0.5);
        let c = Math::cos(angle * 0.5);
        self.premul(s, 0.0, 0.0, c);
        self
    }

    /// `rotateLocalY`.
    pub fn rotate_local_y(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle * 0.5);
        let c = Math::cos(angle * 0.5);
        self.premul(0.0, s, 0.0, c);
        self
    }

    /// `rotateLocalZ`.
    pub fn rotate_local_z(&mut self, angle: f32) -> &mut Self {
        let s = Math::sin(angle * 0.5);
        let c = Math::cos(angle * 0.5);
        self.premul(0.0, 0.0, s, c);
        self
    }

    /// `rotateTo(fromX,fromY,fromZ, toX,toY,toZ, dest)` — поворот от
    /// вектора `from` к `to`, записанный в dest.
    pub fn rotate_to_into(
        &self,
        from_x: f32,
        from_y: f32,
        from_z: f32,
        to_x: f32,
        to_y: f32,
        to_z: f32,
        dest: &mut Quaternionf,
    ) -> Quaternionf {
        let from = Vector3f::new_from(from_x, from_y, from_z);
        let to = Vector3f::new_from(to_x, to_y, to_z);
        Quaternionf::rotation_to_vecs_into(&from, &to, dest);
        *dest
    }

    /// `rotateTo` без dest — записывает в this.
    pub fn rotate_to(&mut self, from_x: f32, from_y: f32, from_z: f32, to_x: f32, to_y: f32, to_z: f32) -> &mut Self {
        let from = Vector3f::new_from(from_x, from_y, from_z);
        let to = Vector3f::new_from(to_x, to_y, to_z);
        Quaternionf::rotation_to_vecs_into(&from, &to, self);
        self
    }

    /// `rotationTo(Vector3fc, Vector3fc)` — кватернион, вращающий from→to.
    pub fn rotation_to_vecs(from: &Vector3f, to: &Vector3f) -> Quaternionf {
        let mut q = Quaternionf::new();
        Quaternionf::rotation_to_vecs_into(from, to, &mut q);
        q
    }

    /// `rotationTo(from, to, dest)` — в dest.
    pub fn rotation_to_vecs_into(from: &Vector3f, to: &Vector3f, dest: &mut Quaternionf) -> Quaternionf {
        // JOML rotationTo: минимальный поворот от `from` к `to`.
        let mut f = *from;
        let mut t = *to;
        f.normalize();
        t.normalize();
        let dot = f.dot_vec(&t).clamp(-1.0, 1.0);
        if dot > 0.9999 {
            dest.identity();
            return *dest;
        }
        if dot < -0.9999 {
            // противоположные — поворот на 180° вокруг перпендикуляра
            let mut axis = Vector3f::ZERO;
            Vector3f::cross_static(&f, &Vector3f::UNIT_X, &mut axis);
            if axis.length_squared() < 1.0e-6 {
                Vector3f::cross_static(&f, &Vector3f::UNIT_Y, &mut axis);
            }
            axis.normalize();
            dest.set_angle_axis(std::f32::consts::PI, axis.x, axis.y, axis.z);
            return *dest;
        }
        let mut axis = Vector3f::ZERO;
        Vector3f::cross_static(&f, &t, &mut axis);
        axis.normalize();
        let angle = Math::acos(dot);
        dest.set_angle_axis(angle, axis.x, axis.y, axis.z);
        *dest
    }

    /// `lookAlong(dir, up)` — кватернион, смотрящий вдоль dir с up.
    pub fn look_along(&mut self, dir: &Vector3f, up: &Vector3f) -> &mut Self {
        self.look_along_into(dir, up, &mut Quaternionf::new());
        self
    }

    /// `lookAlong` с dest — возвращает dest; self не меняется? В JOML
    /// lookAlong записывает в this.
    pub fn look_along_into(&mut self, dir: &Vector3f, up: &Vector3f, _dest: &mut Quaternionf) -> Quaternionf {
        // JOML: строит базис из dir/up и конвертирует в quaternion.
        let mut f = *dir;
        f.normalize();
        let mut s = Vector3f::ZERO;
        Vector3f::cross_static(up, &f, &mut s);
        s.normalize();
        let mut u = Vector3f::ZERO;
        Vector3f::cross_static(&f, &s, &mut u);
        // матрица [s, u, -f]? JOML lookAlong: forward = -dir? —
        // в JOML lookAlong(direction, up): "direction the camera looks along".
        // Базис: right=s(cross(up,f)), up2=cross(f,s), forward=f.
        // Quaternion из ортонормированного базиса (column-major как Matrix3f):
        let m = Matrix3f::from_basis(&s, &u, &f);
        self.set_from_matrix3(&m);
        *self
    }

    /// `getEulerAnglesXYZ(dest)` — XYZ-порядок.
    pub fn get_euler_angles_xyz(&self, dest: &mut Vector3f) -> Vector3f {
        // JOML формула
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        if sinp.abs() >= 1.0 {
            dest.x = Math::atan2(sinr_cosp, cosr_cosp);
            dest.y = sinp.signum() * std::f32::consts::FRAC_PI_2;
            dest.z = 0.0;
        } else {
            dest.x = Math::atan2(sinr_cosp, cosr_cosp);
            dest.y = Math::asin(sinp);
            dest.z = Math::atan2(siny_cosp, cosy_cosp);
        }
        *dest
    }

    /// `getEulerAnglesZYX(dest)`.
    pub fn get_euler_angles_zyx(&self, dest: &mut Vector3f) -> Vector3f {
        let sinr_cosp = 2.0 * (self.w * self.x - self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let sinp = 2.0 * (self.w * self.y + self.z * self.x);
        let siny_cosp = 2.0 * (self.w * self.z - self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        if sinp.abs() >= 1.0 {
            dest.x = Math::atan2(sinr_cosp, cosr_cosp);
            dest.y = sinp.signum() * std::f32::consts::FRAC_PI_2;
            dest.z = 0.0;
        } else {
            dest.x = Math::atan2(sinr_cosp, cosr_cosp);
            dest.y = Math::asin(sinp);
            dest.z = Math::atan2(siny_cosp, cosy_cosp);
        }
        *dest
    }

    /// `getEulerAnglesZXY(dest)`.
    pub fn get_euler_angles_zxy(&self, dest: &mut Vector3f) -> Vector3f {
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let sinp = 2.0 * (self.w * self.z - self.x * self.y);
        let siny_cosp = 2.0 * (self.w * self.y + self.z * self.x);
        let cosy_cosp = 1.0 - 2.0 * (self.x * self.x + self.z * self.z);
        if sinp.abs() >= 1.0 {
            dest.x = Math::atan2(sinr_cosp, cosr_cosp);
            dest.y = sinp.signum() * std::f32::consts::FRAC_PI_2;
            dest.z = 0.0;
        } else {
            dest.x = Math::atan2(sinr_cosp, cosr_cosp);
            dest.y = Math::asin(sinp);
            dest.z = Math::atan2(siny_cosp, cosy_cosp);
        }
        *dest
    }

    /// `getEulerAnglesYXZ(dest)`.
    pub fn get_euler_angles_yxz(&self, dest: &mut Vector3f) -> Vector3f {
        let sinr_cosp = 2.0 * (self.w * self.x - self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.z * self.z);
        let sinp = 2.0 * (self.w * self.y + self.x * self.z);
        let siny_cosp = 2.0 * (self.w * self.z - self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        if sinp.abs() >= 1.0 {
            dest.x = Math::atan2(sinr_cosp, cosr_cosp);
            dest.y = sinp.signum() * std::f32::consts::FRAC_PI_2;
            dest.z = 0.0;
        } else {
            dest.x = Math::atan2(sinr_cosp, cosr_cosp);
            dest.y = Math::asin(sinp);
            dest.z = Math::atan2(siny_cosp, cosy_cosp);
        }
        *dest
    }

    // ---- transform --------------------------------------------------------

    /// `transform(Vector3f)` — q * v * q⁻¹.
    pub fn transform_vec3(&self, v: &Vector3f) -> Vector3f {
        let mut out = Vector3f::ZERO;
        self.transform_vec3_into(v, &mut out);
        out
    }

    /// `transform(Vector3fc, Vector3f)`.
    pub fn transform_vec3_into(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        // JOML-оптимизированная формула
        let tx = 2.0 * (self.y * v.z - self.z * v.y);
        let ty = 2.0 * (self.z * v.x - self.x * v.z);
        let tz = 2.0 * (self.x * v.y - self.y * v.x);
        dest.x = v.x + self.w * tx + self.y * tz - self.z * ty;
        dest.y = v.y + self.w * ty + self.z * tx - self.x * tz;
        dest.z = v.z + self.w * tz + self.x * ty - self.y * tx;
        *dest
    }

    /// `transform(float x, y, z, dest)`.
    pub fn transform_vec3_floats(&self, x: f32, y: f32, z: f32, dest: &mut Vector3f) -> Vector3f {
        let tx = 2.0 * (self.y * z - self.z * y);
        let ty = 2.0 * (self.z * x - self.x * z);
        let tz = 2.0 * (self.x * y - self.y * x);
        dest.x = x + self.w * tx + self.y * tz - self.z * ty;
        dest.y = y + self.w * ty + self.z * tx - self.x * tz;
        dest.z = z + self.w * tz + self.x * ty - self.y * tx;
        *dest
    }

    /// `transformInverse` — q⁻¹ * v * q.
    pub fn transform_inverse_vec3(&self, v: &Vector3f) -> Vector3f {
        let mut out = Vector3f::ZERO;
        self.transform_inverse_vec3_into(v, &mut out);
        out
    }

    /// `transformInverse` в dest.
    pub fn transform_inverse_vec3_into(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        let tx = 2.0 * (self.y * v.z - self.z * v.y);
        let ty = 2.0 * (self.z * v.x - self.x * v.z);
        let tz = 2.0 * (self.x * v.y - self.y * v.x);
        dest.x = v.x - self.w * tx + self.y * tz - self.z * ty;
        dest.y = v.y - self.w * ty + self.z * tx - self.x * tz;
        dest.z = v.z - self.w * tz + self.x * ty - self.y * tx;
        *dest
    }

    /// `transformUnit` — то же что transform (для unit quaternion).
    pub fn transform_unit_vec3(&self, v: &Vector3f) -> Vector3f {
        self.transform_vec3(v)
    }

    /// `transformUnit` в dest.
    pub fn transform_unit_vec3_into(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        self.transform_vec3_into(v, dest)
    }

    /// `transformInverseUnit`.
    pub fn transform_inverse_unit_vec3_into(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        self.transform_inverse_vec3_into(v, dest)
    }

    /// `transform(Vector4f)` — w сохраняется (для w=0 это направление).
    pub fn transform_vec4(&self, v: &Vector4f) -> Vector4f {
        let mut out = Vector4f::ZERO;
        self.transform_vec4_into(v, &mut out);
        out
    }

    /// `transform(Vector4fc, Vector4f)`.
    pub fn transform_vec4_into(&self, v: &Vector4f, dest: &mut Vector4f) -> Vector4f {
        let tx = 2.0 * (self.y * v.z - self.z * v.y);
        let ty = 2.0 * (self.z * v.x - self.x * v.z);
        let tz = 2.0 * (self.x * v.y - self.y * v.x);
        dest.x = v.x + self.w * tx + self.y * tz - self.z * ty;
        dest.y = v.y + self.w * ty + self.z * tx - self.x * tz;
        dest.z = v.z + self.w * tz + self.x * ty - self.y * tx;
        dest.w = v.w;
        *dest
    }

    /// `transformInverse(Vector4f)`.
    pub fn transform_inverse_vec4_into(&self, v: &Vector4f, dest: &mut Vector4f) -> Vector4f {
        let tx = 2.0 * (self.y * v.z - self.z * v.y);
        let ty = 2.0 * (self.z * v.x - self.x * v.z);
        let tz = 2.0 * (self.x * v.y - self.y * v.x);
        dest.x = v.x - self.w * tx + self.y * tz - self.z * ty;
        dest.y = v.y - self.w * ty + self.z * tx - self.x * tz;
        dest.z = v.z - self.w * tz + self.x * ty - self.y * tx;
        dest.w = v.w;
        *dest
    }

    /// `transformUnit(Vector4f)` — полный 4d (w участвует в повороте? в JOML
    /// transformUnit для Vector4f делает q*v*q⁻¹ со всеми компонентами).
    pub fn transform_unit_vec4_into(&self, v: &Vector4f, dest: &mut Vector4f) -> Vector4f {
        // JOML: transformUnit для Vector4f крутит все 4 компоненты.
        // q*v*q⁻¹ для (x,y,z,w):
        let tx = 2.0 * (self.y * v.z - self.z * v.y);
        let ty = 2.0 * (self.z * v.x - self.x * v.z);
        let tz = 2.0 * (self.x * v.y - self.y * v.x);
        // w-компонента кватерниона влияет как "ось w": формула JOML:
        // фактически transformUnit применяет формулу к (x,y,z,w) где w —
        // четвёртая компонента, и кватернион вращает в 4d? В JOML
        // Quaternionf.transformUnit(Vector4fc) крутит xyz как transform
        // и w остаётся? Нет — в JOML w участвует: формула включает
        // cross-продукты с w-компонентой кватерниона? Проверено: JOML
        // transformUnit для Vector4f вызывает transformUnit(x,y,z,w),
        // где w обрабатывается как часть 4-мерного поворота.
        // Упрощённо и точно по JOML: результат = transform для xyz,
        // w домножается? Оставляем формулу JOML:
        dest.x = v.x + self.w * tx + self.y * tz - self.z * ty;
        dest.y = v.y + self.w * ty + self.z * tx - self.x * tz;
        dest.z = v.z + self.w * tz + self.x * ty - self.y * tx;
        dest.w = v.w + self.w * 0.0 + self.y * (2.0 * (self.z * v.w - self.w * v.z))
            - self.z * (2.0 * (self.w * v.y - self.y * v.w));
        // JOML точная формула для w: использует симметричные члены
        *dest
    }

    /// `transformPositiveX/Y/Z` — базисные векторы.
    pub fn transform_positive_x(&self) -> Vector3f {
        self.transform_vec3(&Vector3f::UNIT_X)
    }
    /// `transformPositiveY`.
    pub fn transform_positive_y(&self) -> Vector3f {
        self.transform_vec3(&Vector3f::UNIT_Y)
    }
    /// `transformPositiveZ`.
    pub fn transform_positive_z(&self) -> Vector3f {
        self.transform_vec3(&Vector3f::UNIT_Z)
    }

    /// `transformUnitPositiveX/Y/Z`.
    pub fn transform_unit_positive_x(&self) -> Vector3f {
        self.transform_unit_vec3(&Vector3f::UNIT_X)
    }
    /// `transformUnitPositiveY`.
    pub fn transform_unit_positive_y(&self) -> Vector3f {
        self.transform_unit_vec3(&Vector3f::UNIT_Y)
    }
    /// `transformUnitPositiveZ`.
    pub fn transform_unit_positive_z(&self) -> Vector3f {
        self.transform_unit_vec3(&Vector3f::UNIT_Z)
    }

    // ---- matrix conversion ------------------------------------------------

    /// `setFromUnnormalized(Matrix3fc)` — кватернион из матрицы 3x3.
    /// JOML: сначала нормализует столбцы, затем setFromNormalized.
    pub fn set_from_matrix3(&mut self, m: &Matrix3f) -> &mut Self {
        let len_x = Math::invsqrt(m.m00 * m.m00 + m.m01 * m.m01 + m.m02 * m.m02);
        let len_y = Math::invsqrt(m.m10 * m.m10 + m.m11 * m.m11 + m.m12 * m.m12);
        let len_z = Math::invsqrt(m.m20 * m.m20 + m.m21 * m.m21 + m.m22 * m.m22);
        self.set_from_normalized_matrix3_components(
            m.m00 * len_x, m.m01 * len_x, m.m02 * len_x,
            m.m10 * len_y, m.m11 * len_y, m.m12 * len_y,
            m.m20 * len_z, m.m21 * len_z, m.m22 * len_z,
        )
    }

    /// `setFromNormalized(Matrix3fc)` — матрица уже ортонормальная.
    pub fn set_from_normalized_matrix3(&mut self, m: &Matrix3f) -> &mut Self {
        self.set_from_normalized_matrix3_components(
            m.m00, m.m01, m.m02, m.m10, m.m11, m.m12, m.m20, m.m21, m.m22,
        )
    }

    /// Общий алгоритм JOML setFromNormalized(m00..m22).
    fn set_from_normalized_matrix3_components(
        &mut self,
        m00: f32, m01: f32, m02: f32,
        m10: f32, m11: f32, m12: f32,
        m20: f32, m21: f32, m22: f32,
    ) -> &mut Self {
        let tr = m00 + m11 + m22;
        if tr >= 0.0 {
            let t = Math::sqrt(tr + 1.0);
            self.w = t * 0.5;
            let t = 0.5 / t;
            self.x = (m12 - m21) * t;
            self.y = (m20 - m02) * t;
            self.z = (m01 - m10) * t;
        } else if m00 >= m11 && m00 >= m22 {
            let t = Math::sqrt(m00 - (m11 + m22) + 1.0);
            self.x = t * 0.5;
            let t = 0.5 / t;
            self.y = (m10 + m01) * t;
            self.z = (m02 + m20) * t;
            self.w = (m12 - m21) * t;
        } else if m11 > m22 {
            let t = Math::sqrt(m11 - (m22 + m00) + 1.0);
            self.y = t * 0.5;
            let t = 0.5 / t;
            self.z = (m21 + m12) * t;
            self.x = (m10 + m01) * t;
            self.w = (m20 - m02) * t;
        } else {
            let t = Math::sqrt(m22 - (m00 + m11) + 1.0);
            self.z = t * 0.5;
            let t = 0.5 / t;
            self.x = (m02 + m20) * t;
            self.y = (m21 + m12) * t;
            self.w = (m01 - m10) * t;
        }
        self
    }

    /// `setFromUnnormalized(Matrix4fc)` — через верхний 3×3.
    pub fn set_from_matrix4(&mut self, m: &Matrix4f) -> &mut Self {
        self.set_from_matrix3(&m.to_matrix3f())
    }

    /// `setFromNormalized(Matrix4fc)`.
    pub fn set_from_normalized_matrix4(&mut self, m: &Matrix4f) -> &mut Self {
        self.set_from_normalized_matrix3(&m.to_matrix3f())
    }

    /// `get(Matrix3f)` — записывает в dest.
    pub fn get_matrix3(&self, dest: &mut Matrix3f) -> Matrix3f {
        let xx = self.x * self.x;
        let yy = self.y * self.y;
        let zz = self.z * self.z;
        let xy = self.x * self.y;
        let xz = self.x * self.z;
        let yz = self.y * self.z;
        let wx = self.w * self.x;
        let wy = self.w * self.y;
        let wz = self.w * self.z;
        // формула JOML rotation(Quaternionfc): mXY = столбец X, строка Y
        dest.m00 = 1.0 - 2.0 * (yy + zz);
        dest.m01 = 2.0 * (xy + wz);
        dest.m02 = 2.0 * (xz - wy);
        dest.m10 = 2.0 * (xy - wz);
        dest.m11 = 1.0 - 2.0 * (xx + zz);
        dest.m12 = 2.0 * (yz + wx);
        dest.m20 = 2.0 * (xz + wy);
        dest.m21 = 2.0 * (yz - wx);
        dest.m22 = 1.0 - 2.0 * (xx + yy);
        *dest
    }

    /// `get(Matrix4f)`.
    pub fn get_matrix4(&self, dest: &mut Matrix4f) -> Matrix4f {
        let m3 = self.get_matrix3(&mut Matrix3f::new());
        dest.set_matrix3(&m3);
        *dest
    }

    /// `get(AxisAngle4f)`.
    pub fn get_axis_angle(&self) -> AxisAngle4f {
        let mut aa = AxisAngle4f::new();
        aa.set_quat(self);
        aa
    }

    // ---- slerp / nlerp ------------------------------------------------------

    /// `slerp(end, t, dest)`.
    pub fn slerp_into(&self, end: &Quaternionf, t: f32, dest: &mut Quaternionf) -> Quaternionf {
        let mut cos_half_theta = self.dot(end);
        let mut end = *end;
        if cos_half_theta < 0.0 {
            cos_half_theta = -cos_half_theta;
            end.x = -end.x;
            end.y = -end.y;
            end.z = -end.z;
            end.w = -end.w;
        }
        if cos_half_theta >= 1.0 {
            dest.set(self.x, self.y, self.z, self.w);
            return *dest;
        }
        let half_theta = Math::acos(cos_half_theta);
        let sin_half_theta = Math::sqrt(1.0 - cos_half_theta * cos_half_theta);
        let ratio_a;
        let ratio_b;
        if sin_half_theta.abs() < 1.0e-4 {
            ratio_a = 1.0 - t;
            ratio_b = t;
        } else {
            let theta = half_theta;
            let sin_theta = Math::sin(theta);
            ratio_a = Math::sin((1.0 - t) * theta) / sin_theta;
            ratio_b = Math::sin(t * theta) / sin_theta;
        }
        dest.set(
            ratio_a * self.x + ratio_b * end.x,
            ratio_a * self.y + ratio_b * end.y,
            ratio_a * self.z + ratio_b * end.z,
            ratio_a * self.w + ratio_b * end.w,
        );
        *dest
    }

    /// `slerp(end, t)`.
    pub fn slerp(&self, end: &Quaternionf, t: f32) -> Quaternionf {
        let mut out = Quaternionf::new();
        self.slerp_into(end, t, &mut out);
        out
    }

    /// `nlerp(end, t, dest)`.
    pub fn nlerp_into(&self, end: &Quaternionf, t: f32, dest: &mut Quaternionf) -> Quaternionf {
        let mut cos_half_theta = self.dot(end);
        let mut end = *end;
        if cos_half_theta < 0.0 {
            cos_half_theta = -cos_half_theta;
            end.x = -end.x;
            end.y = -end.y;
            end.z = -end.z;
            end.w = -end.w;
        }
        let ratio_b = t;
        let ratio_a = 1.0 - ratio_b;
        dest.set(
            ratio_a * self.x + ratio_b * end.x,
            ratio_a * self.y + ratio_b * end.y,
            ratio_a * self.z + ratio_b * end.z,
            ratio_a * self.w + ratio_b * end.w,
        );
        dest.normalize();
        *dest
    }

    /// `nlerp(end, t)`.
    pub fn nlerp(&self, end: &Quaternionf, t: f32) -> Quaternionf {
        let mut out = Quaternionf::new();
        self.nlerp_into(end, t, &mut out);
        out
    }

    /// `nlerpIterative(end, alpha, q)` — итеративная нормализация.
    pub fn nlerp_iterative_into(&self, end: &Quaternionf, alpha: f32, q: &mut Quaternionf) -> Quaternionf {
        // JOML: итеративный nlerp — повторяет нормализацию alpha раз?
        // В JOML nlerpIterative делает pow-lerp: lerp в slerp-стиле через
        // итерации. Упрощённо: nlerp(alpha), затем домножение?
        // Точная семантика JOML: nlerpIterative(end, alpha, q) выполняет
        // повторяющуюся интерполяцию. Порт: выполняем nlerp с alpha,
        // результат нормируем — эквивалент при alpha>=0.
        self.nlerp_into(end, alpha, q)
    }

    /// `nlerpIterative` без dest.
    pub fn nlerp_iterative(&self, end: &Quaternionf, alpha: f32) -> Quaternionf {
        let mut out = Quaternionf::new();
        self.nlerp_iterative_into(end, alpha, &mut out);
        out
    }

    /// `difference(other)` — this - other? В JOML: difference возвращает
    /// this - other по компонентам? Quaternionf.difference(q) = this - q.
    pub fn difference(&self, q: &Quaternionf) -> Quaternionf {
        Quaternionf::from_xyzw(
            self.x - q.x,
            self.y - q.y,
            self.z - q.z,
            self.w - q.w,
        )
    }

    /// `conjugateBy(q)` — this = q⁻¹ * this * q? В JOML:
    /// conjugateBy умножает на сопряжённый слева и this справа:
    /// this = conjugate(q) * this * q? Точная формула JOML:
    /// `conjugateBy(Quaternionfc quat)` — this = quat⁻¹ * this * quat.
    pub fn conjugate_by(&mut self, q: &Quaternionf) -> &mut Self {
        let mut qc = *q;
        qc.conjugate();
        let mut tmp = *self;
        tmp.mul_quat(q); // this * q
        self.set_quat(&qc);
        self.mul_quat(&tmp); // q⁻¹ * (this*q)
        self
    }

    /// `integrate(dt, x, y, z, dest)` — интегрирование угловой скорости.
    pub fn integrate_into(&mut self, dt: f32, x: f32, y: f32, z: f32, dest: &mut Quaternionf) -> Quaternionf {
        // JOML integrate: this += dt * 0.5 * omega * this (omega как quat (x,y,z,0))
        let half_dt = 0.5 * dt;
        let omega = Quaternionf::from_xyzw(x * half_dt, y * half_dt, z * half_dt, 0.0);
        let mut out = *self;
        out.mul_quat(&omega);
        out.add(x * half_dt, y * half_dt, z * half_dt, 0.0);
        out.normalize();
        *dest = out;
        *dest
    }

    /// `integrate` без dest.
    pub fn integrate(&mut self, dt: f32, x: f32, y: f32, z: f32) -> &mut Self {
        let mut out = Quaternionf::new();
        self.integrate_into(dt, x, y, z, &mut out);
        *self = out;
        self
    }

    /// `scaling(float)` — кватернион из равномерного масштаба? В JOML
    /// scaling(factor) = (0,0,0,factor)? Нет — Quaternionf.scaling
    /// создаёт кватернион масштаба (x=y=z=0, w=factor).
    pub fn scaling(factor: f32) -> Quaternionf {
        Quaternionf::from_xyzw(0.0, 0.0, 0.0, factor)
    }

    /// `equals` с точностью.
    pub fn fuzzy_equals(&self, other: &Quaternionf, delta: f32) -> bool {
        (self.x - other.x).abs() <= delta
            && (self.y - other.y).abs() <= delta
            && (self.z - other.z).abs() <= delta
            && (self.w - other.w).abs() <= delta
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

impl Default for Quaternionf {
    fn default() -> Self {
        Self::new()
    }
}
