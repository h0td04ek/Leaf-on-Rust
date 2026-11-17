//! Полный перенос `org.joml.AxisAngle4f`.

use crate::joml::math::Math;
use crate::joml::matrix3f::Matrix3f;
use crate::joml::matrix4f::Matrix4f;
use crate::joml::quaternionf::Quaternionf;
use crate::joml::vector3f::Vector3f;
use crate::joml::vector4f::Vector4f;

/// `AxisAngle4f` — представление поворота: ось + угол (рад).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AxisAngle4f {
    /// `angle` — угол в радианах.
    pub angle: f32,
    /// `x`, `y`, `z` — компоненты оси.
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl AxisAngle4f {
    /// `new AxisAngle4f()` — идентичность.
    pub fn new() -> Self {
        Self { angle: 0.0, x: 0.0, y: 1.0, z: 0.0 }
    }

    /// `new AxisAngle4f(axisAngle)`.
    pub fn of(axis_angle: &AxisAngle4f) -> Self {
        *axis_angle
    }

    /// `new AxisAngle4f(quaternion)`.
    pub fn from_quat(q: &Quaternionf) -> Self {
        let mut out = Self::new();
        out.set_quat(q);
        out
    }

    /// `new AxisAngle4f(angle, x, y, z)`.
    pub fn from_angle_xyz(angle: f32, x: f32, y: f32, z: f32) -> Self {
        Self { angle, x, y, z }
    }

    /// `new AxisAngle4f(angle, axis)`.
    pub fn from_angle_axis(angle: f32, axis: &Vector3f) -> Self {
        Self { angle, x: axis.x, y: axis.y, z: axis.z }
    }

    /// `set(axisAngle)`.
    pub fn set(&mut self, axis_angle: &AxisAngle4f) -> &mut Self {
        self.angle = axis_angle.angle;
        self.x = axis_angle.x;
        self.y = axis_angle.y;
        self.z = axis_angle.z;
        self
    }

    /// `set(angle, x, y, z)`.
    pub fn set_angle_xyz(&mut self, angle: f32, x: f32, y: f32, z: f32) -> &mut Self {
        self.angle = angle;
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    /// `set(angle, axis)`.
    pub fn set_angle_axis(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.set_angle_xyz(angle, axis.x, axis.y, axis.z)
    }

    /// `set(Quaternionfc)` — формула JOML.
    pub fn set_quat(&mut self, q: &Quaternionf) -> &mut Self {
        let acos = Math::safe_acos(q.w);
        let inv_sqrt = Math::invsqrt(1.0 - q.w * q.w);
        if inv_sqrt.is_infinite() {
            self.x = 0.0;
            self.y = 0.0;
            self.z = 1.0;
        } else {
            self.x = q.x * inv_sqrt;
            self.y = q.y * inv_sqrt;
            self.z = q.z * inv_sqrt;
        }
        self.angle = acos + acos;
        self
    }

    /// `set(Matrix3fc)` — алгоритм JOML: нормализация столбцов,
    /// проверка симметрии, ось из кососимметричной части.
    pub fn set_matrix3(&mut self, m: &Matrix3f) -> &mut Self {
        let mut nm00 = m.m00;
        let mut nm01 = m.m01;
        let mut nm02 = m.m02;
        let mut nm10 = m.m10;
        let mut nm11 = m.m11;
        let mut nm12 = m.m12;
        let mut nm20 = m.m20;
        let mut nm21 = m.m21;
        let mut nm22 = m.m22;
        let len_x = Math::invsqrt(m.m00 * m.m00 + m.m01 * m.m01 + m.m02 * m.m02);
        let len_y = Math::invsqrt(m.m10 * m.m10 + m.m11 * m.m11 + m.m12 * m.m12);
        let len_z = Math::invsqrt(m.m20 * m.m20 + m.m21 * m.m21 + m.m22 * m.m22);
        nm00 *= len_x;
        nm01 *= len_x;
        nm02 *= len_x;
        nm10 *= len_y;
        nm11 *= len_y;
        nm12 *= len_y;
        nm20 *= len_z;
        nm21 *= len_z;
        nm22 *= len_z;
        let epsilon = 1.0e-4f32;
        let epsilon2 = 1.0e-3f32;
        if (nm10 - nm01).abs() < epsilon
            && (nm20 - nm02).abs() < epsilon
            && (nm21 - nm12).abs() < epsilon
        {
            if (nm10 + nm01).abs() < epsilon2
                && (nm20 + nm02).abs() < epsilon2
                && (nm21 + nm12).abs() < epsilon2
                && (nm00 + nm11 + nm22 - 3.0).abs() < epsilon2
            {
                self.x = 0.0;
                self.y = 0.0;
                self.z = 1.0;
                self.angle = 0.0;
                return self;
            }
            self.angle = std::f32::consts::PI;
            let xx = (nm00 + 1.0) / 2.0;
            let yy = (nm11 + 1.0) / 2.0;
            let zz = (nm22 + 1.0) / 2.0;
            let xy = (nm10 + nm01) / 4.0;
            let xz = (nm20 + nm02) / 4.0;
            let yz = (nm21 + nm12) / 4.0;
            if xx > yy && xx > zz {
                self.x = Math::sqrt(xx);
                let inv_x = 1.0 / self.x;
                self.y = xy * inv_x;
                self.z = xz * inv_x;
            } else if yy > zz {
                // NB: в JOML здесь invZ = 1/z по СТАРОМУ значению поля z
                // (upstream-особенность ветки угла PI) — переносим как есть.
                self.y = Math::sqrt(yy);
                let inv_z = 1.0 / self.z;
                self.x = xy * inv_z;
                self.z = yz * inv_z;
            } else {
                self.z = Math::sqrt(zz);
                let inv_y = 1.0 / self.y;
                self.x = xz * inv_y;
                self.y = yz * inv_y;
            }
            return self;
        }
        let s = Math::invsqrt(
            (nm12 - nm21) * (nm12 - nm21)
                + (nm20 - nm02) * (nm20 - nm02)
                + (nm01 - nm10) * (nm01 - nm10),
        );
        self.angle = Math::safe_acos((nm00 + nm11 + nm22 - 1.0) / 2.0);
        self.x = (nm12 - nm21) * s;
        self.y = (nm20 - nm02) * s;
        self.z = (nm01 - nm10) * s;
        self
    }

    /// `set(matrix4)` — через `Matrix3f` верхний левый блок? В JOML:
    /// извлекает из Matrix4f как Matrix3f.
    pub fn set_matrix4(&mut self, m: &Matrix4f) -> &mut Self {
        let m3 = m.to_matrix3f();
        self.set_matrix3(&m3)
    }

    /// `get(Quaternionf)`.
    pub fn get_quat(&self) -> Quaternionf {
        let mut q = Quaternionf::new();
        self.write_quat(&mut q);
        q
    }

    /// `get(Quaternionf)` — в dest.
    pub fn write_quat(&self, dest: &mut Quaternionf) -> Quaternionf {
        let s = self.angle * 0.5;
        let sin = Math::sin(s);
        let cos = Math::cos(s);
        dest.set(self.x * sin, self.y * sin, self.z * sin, cos);
        *dest
    }

    /// `get(Matrix4f)` — Matrix4f.set(AxisAngle4f).
    pub fn get_matrix4(&self, dest: &mut Matrix4f) -> Matrix4f {
        dest.set_axis_angle(self);
        *dest
    }

    /// `get(Matrix3f)` — Matrix3f.set(AxisAngle4f).
    pub fn get_matrix3(&self, dest: &mut Matrix3f) -> Matrix3f {
        dest.set_axis_angle(self);
        *dest
    }

    /// `get(AxisAngle4f)` — копия.
    pub fn get(&self) -> Self {
        *self
    }

    /// `normalize` — нормализация оси.
    pub fn normalize(&mut self) -> &mut Self {
        let l = Math::sqrt(self.x * self.x + self.y * self.y + self.z * self.z);
        if l != 0.0 {
            let inv = 1.0 / l;
            self.x *= inv;
            self.y *= inv;
            self.z *= inv;
        }
        self
    }

    /// `rotate(angle)` — умножает текущий угол.
    pub fn rotate(&mut self, angle: f32) -> &mut Self {
        self.angle += angle;
        self
    }

    /// `transform(Vector3f)` — поворот вектора.
    pub fn transform_vec3(&self, v: &Vector3f) -> Vector3f {
        let q = self.get_quat();
        let mut out = Vector3f::ZERO;
        q.transform_vec3_into(v, &mut out);
        out
    }

    /// `transform(Vector3fc, Vector3f)`.
    pub fn transform_vec3_into(&self, v: &Vector3f, dest: &mut Vector3f) -> Vector3f {
        let q = self.get_quat();
        q.transform_vec3_into(v, dest);
        *dest
    }

    /// `transform(Vector4f)`.
    pub fn transform_vec4(&self, v: &Vector4f) -> Vector4f {
        let q = self.get_quat();
        let mut out = Vector4f::ZERO;
        q.transform_vec4_into(v, &mut out);
        out
    }

    /// `transform(Vector4fc, Vector4f)`.
    pub fn transform_vec4_into(&self, v: &Vector4f, dest: &mut Vector4f) -> Vector4f {
        let q = self.get_quat();
        q.transform_vec4_into(v, dest);
        *dest
    }

    /// `toString`.
    pub fn to_string_fmt(&self) -> String {
        format!("AxisAngle4f({} {} {} {})", self.angle, self.x, self.y, self.z)
    }
}
