//! Перенос `net.minecraft.world.phys.Vec3` — 3D-вектор double.
//! Codec/StreamCodec (network) — TODO отдельной итерацией.

use crate::util::mth;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const X_AXIS: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y_AXIS: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z_AXIS: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn add(&self, x: f64, y: f64, z: f64) -> Self {
        Self::new(self.x + x, self.y + y, self.z + z)
    }

    pub fn add_vec(&self, other: &Self) -> Self {
        self.add(other.x, other.y, other.z)
    }

    pub fn add_floored(&self, x: i32, y: i32, z: i32) -> Self {
        self.add(x as f64, y as f64, z as f64)
    }

    pub fn subtract(&self, x: f64, y: f64, z: f64) -> Self {
        Self::new(self.x - x, self.y - y, self.z - z)
    }

    pub fn subtract_vec(&self, other: &Self) -> Self {
        self.subtract(other.x, other.y, other.z)
    }

    pub fn scale(&self, factor: f64) -> Self {
        self.multiply(factor, factor, factor)
    }

    pub fn multiply(&self, x: f64, y: f64, z: f64) -> Self {
        Self::new(self.x * x, self.y * y, self.z * z)
    }

    pub fn reverse_subtract(&self, other: &Self) -> Self {
        Self::new(other.x - self.x, other.y - self.y, other.z - self.z)
    }

    pub fn dot(&self, x: f64, y: f64, z: f64) -> f64 {
        self.x * x + self.y * y + self.z * z
    }

    pub fn dot_vec(&self, other: &Self) -> f64 {
        self.dot(other.x, other.y, other.z)
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length_squared(&self) -> f64 {
        self.dot(self.x, self.y, self.z)
    }

    /// `lengthSqr()`.
    pub fn length_sqr(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f64 {
        mth::sqrt_f64(self.length_squared())
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        self.distance_to_sqr(other).sqrt()
    }

    pub fn distance_to_sqr(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn distance_to_center_sqr(&self, x: i32, y: i32, z: i32) -> f64 {
        let dx = self.x - (x as f64 + 0.5);
        let dy = self.y - (y as f64 + 0.5);
        let dz = self.z - (z as f64 + 0.5);
        dx * dx + dy * dy + dz * dz
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len < 1.0e-4 {
            Self::ZERO
        } else {
            self.scale(1.0 / len)
        }
    }

    pub fn negate(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }

    pub fn offset_x_rot(&self, angle: f32) -> Self {
        let cos = mth::cos(angle as f64) as f64;
        let sin = mth::sin(angle as f64) as f64;
        Self::new(self.x * cos - self.z * sin, self.y, self.z * cos + self.x * sin)
    }

    pub fn offset_y_rot(&self, angle: f32) -> Self {
        let cos = mth::cos(angle as f64) as f64;
        let sin = mth::sin(angle as f64) as f64;
        Self::new(self.x * cos + self.z * sin, self.y, self.z * cos - self.x * sin)
    }

    pub fn offset_z_rot(&self, angle: f32) -> Self {
        let cos = mth::cos(angle as f64) as f64;
        let sin = mth::sin(angle as f64) as f64;
        Self::new(self.x * cos - self.y * sin, self.y * cos + self.x * sin, self.z)
    }

    pub fn y_rot(&self, angle: f32) -> Self {
        let cos = mth::cos(angle as f64) as f64;
        let sin = mth::sin(angle as f64) as f64;
        Self::new(self.x * cos + self.z * sin, self.y, self.z * cos - self.x * sin)
    }

    pub fn x_rot(&self, angle: f32) -> Self {
        let cos = mth::cos(angle as f64) as f64;
        let sin = mth::sin(angle as f64) as f64;
        Self::new(self.x, self.y * cos - self.z * sin, self.y * sin + self.z * cos)
    }

    pub fn z_rot(&self, angle: f32) -> Self {
        let cos = mth::cos(angle as f64) as f64;
        let sin = mth::sin(angle as f64) as f64;
        Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos, self.z)
    }

    pub fn lower(&self) -> Self {
        Self::new(self.x.floor(), self.y.floor(), self.z.floor())
    }

    pub fn upper(&self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil(), self.z.ceil())
    }

    pub fn relative(&self, x: f64, y: f64, z: f64) -> Self {
        self.add(x, y, z)
    }

    pub fn relative_vec(&self, other: &Self) -> Self {
        self.add_vec(other)
    }

    pub fn closer_than(&self, other: &Self, distance: f64) -> bool {
        self.distance_to_sqr(other) < distance * distance
    }

    pub fn lerp(&self, target: &Self, delta: f64) -> Self {
        mth::lerp_vec(delta, self, target)
    }

    pub fn lerp_to_direction(&self, target: &Self, delta: f64) -> Self {
        let scaled = target.scale(delta);
        let remaining = 1.0 - delta;
        self.scale(remaining).add_vec(&scaled).normalize()
    }

    pub fn vector_to(&self, target: &Self) -> Self {
        target.subtract_vec(self).normalize()
    }

    pub fn to_vector3f(&self) -> crate::joml::Vector3f {
        crate::joml::Vector3f::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

// ---------------------------------------------------------------- статические фабрики Vec3i

use crate::core::vec3i::Vec3i;

pub fn at_lower_corner_of(to_copy: &Vec3i) -> Vec3 {
    Vec3::new(to_copy.x as f64, to_copy.y as f64, to_copy.z as f64)
}

pub fn at_lower_corner_with_offset(to_copy: &Vec3i, offset_x: f64, offset_y: f64, offset_z: f64) -> Vec3 {
    Vec3::new(
        to_copy.x as f64 + offset_x,
        to_copy.y as f64 + offset_y,
        to_copy.z as f64 + offset_z,
    )
}

pub fn at_center_of(to_copy: &Vec3i) -> Vec3 {
    at_lower_corner_with_offset(to_copy, 0.5, 0.5, 0.5)
}

pub fn at_bottom_center_of(to_copy: &Vec3i) -> Vec3 {
    at_lower_corner_with_offset(to_copy, 0.5, 0.0, 0.5)
}

pub fn up_from_bottom_center_of(to_copy: &Vec3i, vertical_offset: f64) -> Vec3 {
    at_lower_corner_with_offset(to_copy, 0.5, vertical_offset, 0.5)
}

impl Vec3 {
    /// `Vec3.STREAM_CODEC` — 3 double (`readVec3`/`writeVec3`).
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<
        crate::network::friendly_byte_buf::FriendlyByteBuf,
        Vec3,
    > {
        crate::network::codec::stream_codec::FnCodec::new(
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, v: &Vec3| {
                buf.write_vec3(v);
            },
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| buf.read_vec3(),
        )
    }
}
