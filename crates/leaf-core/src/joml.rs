//! Минимальные замены типов `org.joml` (Vector3f, Vector4f, Quaternionf),
//! которые используются портированными util/world-классами. API совпадает
//! с joml настолько, насколько это нужно ядру; при необходимости будет
//! расширяться по мере переноса.

/// `org.joml.Vector3f`
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3f {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const UNIT_X: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const UNIT_Y: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const UNIT_Z: Self = Self { x: 0.0, y: 0.0, z: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn set(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn dot(&self, x: f32, y: f32, z: f32) -> f32 {
        self.x * x + self.y * y + self.z * z
    }

    pub fn dot_vec(&self, other: &Self) -> f32 {
        self.dot(other.x, other.y, other.z)
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize(&mut self) -> &mut Self {
        let len = self.length();
        if len > 0.0 {
            self.x /= len;
            self.y /= len;
            self.z /= len;
        }
        self
    }

    pub fn normalized(&self) -> Self {
        let mut v = *self;
        v.normalize();
        v
    }

    pub fn add(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x += x;
        self.y += y;
        self.z += z;
        self
    }

    pub fn add_vec(&mut self, other: &Self) -> &mut Self {
        self.add(other.x, other.y, other.z)
    }

    pub fn sub(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x -= x;
        self.y -= y;
        self.z -= z;
        self
    }

    pub fn sub_vec(&mut self, other: &Self) -> &mut Self {
        self.sub(other.x, other.y, other.z)
    }

    pub fn scale(&mut self, factor: f32) -> &mut Self {
        self.x *= factor;
        self.y *= factor;
        self.z *= factor;
        self
    }

    pub fn mul(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x *= x;
        self.y *= y;
        self.z *= z;
        self
    }

    pub fn negate(&mut self) -> &mut Self {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }

    pub fn cross(&mut self, other: &Self) -> &mut Self {
        let (x, y, z) = (
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        );
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn distance_squared(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn distance(&self, other: &Self) -> f32 {
        self.distance_squared(other).sqrt()
    }
}

/// `org.joml.Vector4f`
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4f {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;
        self
    }
}

/// `org.joml.Quaternionf` — минимальная версия (x,y,z,w).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quaternionf {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternionf {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;
        self
    }

    pub fn set_quat(&mut self, other: &Self) -> &mut Self {
        *self = *other;
        self
    }

    pub fn normalize(&mut self) -> &mut Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if len > 0.0 {
            self.x /= len;
            self.y /= len;
            self.z /= len;
            self.w /= len;
        }
        self
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// `rotationX(angle)` — кватернион поворота вокруг X.
    pub fn rotation_x(&mut self, angle: f32) -> &mut Self {
        let h = angle * 0.5;
        self.x = h.sin();
        self.y = 0.0;
        self.z = 0.0;
        self.w = h.cos();
        self
    }

    /// `rotationY(angle)`.
    pub fn rotation_y(&mut self, angle: f32) -> &mut Self {
        let h = angle * 0.5;
        self.x = 0.0;
        self.y = h.sin();
        self.z = 0.0;
        self.w = h.cos();
        self
    }

    /// `rotationZ(angle)`.
    pub fn rotation_z(&mut self, angle: f32) -> &mut Self {
        let h = angle * 0.5;
        self.x = 0.0;
        self.y = 0.0;
        self.z = h.sin();
        self.w = h.cos();
        self
    }

    /// `rotationXYZ(angleX, angleY, angleZ)` — порядок вращения XYZ,
    /// формула 1:1 с joml (декомпилировано из Quaternionf.class).
    pub fn rotation_xyz(&mut self, angle_x: f32, angle_y: f32, angle_z: f32) -> &mut Self {
        let sx = (angle_x * 0.5).sin();
        let cx = (angle_x * 0.5).cos();
        let sy = (angle_y * 0.5).sin();
        let cy = (angle_y * 0.5).cos();
        let sz = (angle_z * 0.5).sin();
        let cz = (angle_z * 0.5).cos();
        let t10 = cy * cz;
        let t11 = sy * sz;
        let t12 = sy * cz;
        let t13 = cy * sz;
        self.w = cx * t10 - sx * t11;
        self.x = sx * t10 + cx * t11;
        self.y = cx * t12 - sx * t13;
        self.z = cx * t13 + sx * t12;
        self
    }
}
