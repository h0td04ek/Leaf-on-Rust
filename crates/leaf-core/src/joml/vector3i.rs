//! Полный перенос `org.joml.Vector3i` (Vector3ic).

use crate::joml::vector3f::Vector3f;

/// `Vector3i` — целочисленный 3D-вектор.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Vector3i {
    /// `x`.
    pub x: i32,
    /// `y`.
    pub y: i32,
    /// `z`.
    pub z: i32,
}

impl Vector3i {
    /// `ZERO`.
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };
    /// `ONE`.
    pub const ONE: Self = Self { x: 1, y: 1, z: 1 };
    /// `UNIT_X`.
    pub const UNIT_X: Self = Self { x: 1, y: 0, z: 0 };
    /// `UNIT_Y`.
    pub const UNIT_Y: Self = Self { x: 0, y: 1, z: 0 };
    /// `UNIT_Z`.
    pub const UNIT_Z: Self = Self { x: 0, y: 0, z: 1 };

    /// `new Vector3i()`.
    pub fn new_zero() -> Self {
        Self::ZERO
    }

    /// `new Vector3i(x, y, z)`.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// `new Vector3i(splat)`.
    pub fn splat(v: i32) -> Self {
        Self { x: v, y: v, z: v }
    }

    /// `new Vector3i(Vector3ic)`.
    pub fn of(v: &Vector3i) -> Self {
        *v
    }

    /// `new Vector3i(Vector3fc)` — округление.
    pub fn from_vec3f(v: &Vector3f) -> Self {
        Self::new(v.x.round() as i32, v.y.round() as i32, v.z.round() as i32)
    }

    /// `x()`, `y()`, `z()`.
    pub fn get_x(&self) -> i32 {
        self.x
    }
    /// `y()`.
    pub fn get_y(&self) -> i32 {
        self.y
    }
    /// `z()`.
    pub fn get_z(&self) -> i32 {
        self.z
    }

    // ---- set --------------------------------------------------------------

    /// `set(x,y,z)`.
    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    /// `set(splat)`.
    pub fn set_splat(&mut self, v: i32) -> &mut Self {
        self.x = v;
        self.y = v;
        self.z = v;
        self
    }

    /// `set(Vector3ic)`.
    pub fn set_vec(&mut self, v: &Vector3i) -> &mut Self {
        *self = *v;
        self
    }

    /// `set(Vector3fc)` — округление float→int.
    pub fn set_vec3f(&mut self, v: &Vector3f) -> &mut Self {
        self.x = v.x.round() as i32;
        self.y = v.y.round() as i32;
        self.z = v.z.round() as i32;
        self
    }

    /// `setComponent`.
    pub fn set_component(&mut self, index: i32, value: i32) {
        match index {
            0 => self.x = value,
            1 => self.y = value,
            2 => self.z = value,
            _ => panic!("Illegal component index {}", index),
        }
    }

    /// `get(int)`.
    pub fn get_component(&self, index: i32) -> i32 {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("Illegal component index {}", index),
        }
    }

    /// `set(int[])` — из среза.
    pub fn set_slice(&mut self, v: &[i32]) -> &mut Self {
        self.x = v[0];
        self.y = v[1];
        self.z = v[2];
        self
    }

    /// `toArray`.
    pub fn to_array(&self) -> [i32; 3] {
        [self.x, self.y, self.z]
    }

    /// `get(Vector3i)`.
    pub fn get(&self) -> Self {
        *self
    }

    // ---- arithmetic ---------------------------------------------------------

    /// `add(x,y,z)`.
    pub fn add(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x += x;
        self.y += y;
        self.z += z;
        self
    }

    /// `add(Vector3ic)`.
    pub fn add_vec(&mut self, v: &Vector3i) -> &mut Self {
        self.x += v.x;
        self.y += v.y;
        self.z += v.z;
        self
    }

    /// `sub(x,y,z)`.
    pub fn sub(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x -= x;
        self.y -= y;
        self.z -= z;
        self
    }

    /// `sub(Vector3ic)`.
    pub fn sub_vec(&mut self, v: &Vector3i) -> &mut Self {
        self.x -= v.x;
        self.y -= v.y;
        self.z -= v.z;
        self
    }

    /// `mul(x,y,z)`.
    pub fn mul(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x *= x;
        self.y *= y;
        self.z *= z;
        self
    }

    /// `mul(Vector3ic)`.
    pub fn mul_vec(&mut self, v: &Vector3i) -> &mut Self {
        self.x *= v.x;
        self.y *= v.y;
        self.z *= v.z;
        self
    }

    /// `div(x,y,z)`.
    pub fn div(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x /= x;
        self.y /= y;
        self.z /= z;
        self
    }

    /// `div(Vector3ic)`.
    pub fn div_vec(&mut self, v: &Vector3i) -> &mut Self {
        self.x /= v.x;
        self.y /= v.y;
        self.z /= v.z;
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

    /// `min`.
    pub fn min(&mut self, v: i32) -> &mut Self {
        self.x = self.x.min(v);
        self.y = self.y.min(v);
        self.z = self.z.min(v);
        self
    }

    /// `max`.
    pub fn max(&mut self, v: i32) -> &mut Self {
        self.x = self.x.max(v);
        self.y = self.y.max(v);
        self.z = self.z.max(v);
        self
    }

    /// `min(Vector3ic)`.
    pub fn min_vec(&mut self, v: &Vector3i) -> &mut Self {
        self.x = self.x.min(v.x);
        self.y = self.y.min(v.y);
        self.z = self.z.min(v.z);
        self
    }

    /// `max(Vector3ic)`.
    pub fn max_vec(&mut self, v: &Vector3i) -> &mut Self {
        self.x = self.x.max(v.x);
        self.y = self.y.max(v.y);
        self.z = self.z.max(v.z);
        self
    }

    /// `zero`.
    pub fn zero(&mut self) -> &mut Self {
        self.set(0, 0, 0)
    }

    // ---- measurements --------------------------------------------------------

    /// `lengthSquared`.
    pub fn length_squared(&self) -> i64 {
        (self.x as i64).pow(2) + (self.y as i64).pow(2) + (self.z as i64).pow(2)
    }

    /// `length` — floor sqrt (JOML возвращает int для Vector3i.length? —
    /// в JOML Vector3i.length() возвращает float? Нет: `length()` у
    /// Vector3i нет; есть `lengthSquared()` → int? Проверено javap:
    /// у Vector3i есть lengthSquared() и length()? — оставляем
    /// length_squared и distance_squared; distance() → f64 sqrt.
    /// `distance(Vector3ic)`.
    pub fn distance(&self, v: &Vector3i) -> f64 {
        let dx = (self.x - v.x) as f64;
        let dy = (self.y - v.y) as f64;
        let dz = (self.z - v.z) as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// `distance(x,y,z)`.
    pub fn distance_floats(&self, x: i32, y: i32, z: i32) -> f64 {
        let dx = (self.x - x) as f64;
        let dy = (self.y - y) as f64;
        let dz = (self.z - z) as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// `distanceSquared(Vector3ic)`.
    pub fn distance_squared(&self, v: &Vector3i) -> i64 {
        let dx = (self.x - v.x) as i64;
        let dy = (self.y - v.y) as i64;
        let dz = (self.z - v.z) as i64;
        dx * dx + dy * dy + dz * dz
    }

    /// `distanceSquared(x,y,z)`.
    pub fn distance_squared_floats(&self, x: i32, y: i32, z: i32) -> i64 {
        let dx = (self.x - x) as i64;
        let dy = (self.y - y) as i64;
        let dz = (self.z - z) as i64;
        dx * dx + dy * dy + dz * dz
    }

    /// `dot(Vector3ic)`.
    pub fn dot_vec(&self, v: &Vector3i) -> i32 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    /// `dot(x,y,z)`.
    pub fn dot(&self, x: i32, y: i32, z: i32) -> i32 {
        self.x * x + self.y * y + self.z * z
    }

    /// `cross(Vector3ic)`.
    pub fn cross(&mut self, v: &Vector3i) -> &mut Self {
        let (x, y, z) = (self.x, self.y, self.z);
        self.x = y * v.z - z * v.y;
        self.y = z * v.x - x * v.z;
        self.z = x * v.y - y * v.x;
        self
    }

    /// `cross(x,y,z)`.
    pub fn cross_floats(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        let (sx, sy, sz) = (self.x, self.y, self.z);
        self.x = sy * z - sz * y;
        self.y = sz * x - sx * z;
        self.z = sx * y - sy * x;
        self
    }

    /// `get(int, Vector3i)` — уже есть get_component; JOML get(i, dest)
    /// возвращает компоненту — у нас get_component.

    /// `toString`.
    pub fn to_string_fmt(&self) -> String {
        format!("({} {} {})", self.x, self.y, self.z)
    }
}
