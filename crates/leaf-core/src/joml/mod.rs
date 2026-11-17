//! Полный перенос пакета `org.joml` — математические типы JOML.
//!
//! Структура зеркалит Java-пакет: Vector2f/3f/3i/4f, Quaternionf,
//! AxisAngle4f, Matrix2f/3f/3x2f/4f, Math. Методы повторяют JOML API
//! (snake_case имена, как в остальных модулях leaf-core).

pub mod axis_angle4f;
pub mod math;
pub mod matrix2f;
pub mod matrix3f;
pub mod matrix3x2f;
pub mod matrix4f;
pub mod quaternionf;
pub mod vector2f;
pub mod vector3f;
pub mod vector3i;
pub mod vector4f;

pub use axis_angle4f::AxisAngle4f;
pub use matrix2f::Matrix2f;
pub use matrix3f::Matrix3f;
pub use matrix3x2f::Matrix3x2f;
pub use matrix4f::Matrix4f;
pub use quaternionf::Quaternionf;
pub use vector2f::Vector2f;
pub use vector3f::Vector3f;
pub use vector3i::Vector3i;
pub use vector4f::Vector4f;

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn vector3f_arithmetic() {
        let mut v = Vector3f::new(1.0, 2.0, 3.0);
        v.add_vec(&Vector3f::new(4.0, 5.0, 6.0));
        assert!(approx(v.x, 5.0) && approx(v.y, 7.0) && approx(v.z, 9.0));

        v.sub(1.0, 1.0, 1.0);
        assert!(approx(v.x, 4.0) && approx(v.z, 8.0));

        v.mul_scalar(0.5);
        assert!(approx(v.y, 3.0));

        v.div_scalar(2.0);
        assert!(approx(v.x, 1.0) && approx(v.y, 1.5) && approx(v.z, 2.0));
    }

    #[test]
    fn vector3f_cross_dot_length() {
        let a = Vector3f::new(1.0, 0.0, 0.0);
        let b = Vector3f::new(0.0, 1.0, 0.0);
        let mut c = Vector3f::ZERO;
        Vector3f::cross_static(&a, &b, &mut c);
        assert!(approx(c.x, 0.0) && approx(c.y, 0.0) && approx(c.z, 1.0));

        assert!(approx(a.dot_vec(&b), 0.0));
        assert!(approx(a.dot_vec(&a), 1.0));

        let mut n = Vector3f::new(0.0, 0.0, -10.0);
        n.normalize();
        assert!(approx(n.z, -1.0) && approx(n.length(), 1.0));

        assert!(approx(Vector3f::new(1.0, 0.0, 0.0).angle(&Vector3f::new(0.0, 1.0, 0.0)), FRAC_PI_2));
    }

    #[test]
    fn vector3f_lerp_and_components() {
        let mut v = Vector3f::new(0.0, 0.0, 0.0);
        v.lerp(&Vector3f::new(10.0, 20.0, 30.0), 0.5);
        assert!(approx(v.x, 5.0) && approx(v.y, 10.0) && approx(v.z, 15.0));

        let mut w = Vector3f::new(1.0, 2.0, 3.0);
        w.set_component(1, 9.0);
        assert!(approx(w.y, 9.0) && w.get_component(2) == 3.0);
    }

    #[test]
    fn quaternion_angle_axis_rotation() {
        // Поворот на 90° вокруг Z: (1,0,0) -> (0,1,0)
        let mut q = Quaternionf::new();
        q.set_angle_axis(FRAC_PI_2, 0.0, 0.0, 1.0);

        let v = Vector3f::new(1.0, 0.0, 0.0);
        let out = q.transform_vec3(&v);
        assert!(approx(out.x, 0.0) && approx(out.y, 1.0), "got ({}, {}, {})", out.x, out.y, out.z);
    }

    #[test]
    fn quaternion_identity_and_normalize() {
        let q = Quaternionf::new();
        assert!(approx(q.w, 1.0) && approx(q.x, 0.0));

        let mut q2 = Quaternionf::from_wxyz(2.0, 0.0, 0.0, 0.0);
        q2.normalize();
        assert!(approx(q2.length_squared(), 1.0));
    }

    #[test]
    fn quaternion_mul_order() {
        // qx * qy applied to vector
        let mut q = Quaternionf::new();
        q.rotation_x(PI);
        let mut r = Quaternionf::new();
        r.rotation_y(PI);
        let mut both = q;
        both.mul_quat(&r);

        let v = Vector3f::new(1.0, 0.0, 0.0);
        let out = both.transform_vec3(&v);
        assert!(approx(out.length(), 1.0));
    }

    #[test]
    fn quaternion_axis_angle_roundtrip() {
        let mut q = Quaternionf::new();
        q.set_angle_axis(0.7, 0.0, 1.0, 0.0);
        q.normalize();
        let aa = q.get_axis_angle();
        assert!(approx(aa.angle, 0.7));
    }

    #[test]
    fn matrix4f_translation_transform() {
        let m = Matrix4f::translation(10.0, 20.0, 30.0);
        let mut v = Vector3f::new(1.0, 2.0, 3.0);
        let mut out = Vector3f::ZERO;
        m.transform_vec3(&v, &mut out);
        assert!(approx(out.x, 11.0) && approx(out.y, 22.0) && approx(out.z, 33.0));

        // перевод обратно через инверсию
        let mut inv = Matrix4f::of(&m);
        inv.invert();
        let mut back = Vector3f::ZERO;
        inv.transform_vec3(&out, &mut back);
        assert!(approx(back.x, 1.0) && approx(back.y, 2.0) && approx(back.z, 3.0));
    }

    #[test]
    fn matrix4f_scale_determinant() {
        let mut m = Matrix4f::identity();
        m.scale(2.0, 3.0, 4.0);
        assert!(approx(m.determinant(), 24.0));

        let mut v = Vector3f::new(1.0, 1.0, 1.0);
        let mut out = Vector3f::ZERO;
        m.transform_vec3(&v, &mut out);
        assert!(approx(out.x, 2.0) && approx(out.y, 3.0) && approx(out.z, 4.0));
    }

    #[test]
    fn matrix4f_rotate_quat() {
        let mut m = Matrix4f::identity();
        let mut q = Quaternionf::new();
        q.set_angle_axis(FRAC_PI_2, 0.0, 0.0, 1.0);
        m.rotate_quat(&q);

        let mut v = Vector3f::new(1.0, 0.0, 0.0);
        let mut out = Vector3f::ZERO;
        m.transform_vec3(&v, &mut out);
        assert!(approx(out.x, 0.0) && approx(out.y, 1.0), "got ({}, {}, {})", out.x, out.y, out.z);
    }

    #[test]
    fn matrix3f_determinant() {
        let mut m = Matrix3f::identity();
        assert!(approx(m.determinant(), 1.0));
        m.scale(2.0, 3.0, 4.0);
        assert!(approx(m.determinant(), 24.0));
    }

    #[test]
    fn axis_angle4f_quat() {
        let aa = AxisAngle4f::from_angle_xyz(PI, 0.0, 1.0, 0.0);
        let q = aa.get_quat();
        let v = Vector3f::new(1.0, 0.0, 0.0);
        let out = q.transform_vec3(&v);
        assert!(approx(out.x, -1.0) && approx(out.y, 0.0), "got ({}, {}, {})", out.x, out.y, out.z);
    }
}
