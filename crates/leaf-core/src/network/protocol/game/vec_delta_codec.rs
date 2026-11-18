//! `VecDeltaCodec` — кодек для дельта-сжатия Vec3 (truncation 4096).

use crate::world::phys::vec3::Vec3;

/// `VecDeltaCodec`.
#[derive(Clone, Debug)]
pub struct VecDeltaCodec {
    /// `base`.
    pub base: Vec3,
}

impl VecDeltaCodec {
    /// `new`.
    pub fn new() -> Self {
        Self { base: Vec3::ZERO }
    }

    /// `new(base)`.
    pub fn with_base(base: Vec3) -> Self {
        Self { base }
    }

    /// `encode` — `Math.round(value * 4096.0)`.
    pub fn encode(value: f64) -> i64 {
        (value * 4096.0).round() as i64
    }

    /// `decode` — `value / 4096.0`.
    pub fn decode(value: i64) -> f64 {
        value as f64 / 4096.0
    }

    /// `decode(x, y, z)` — декодирование с base.
    pub fn decode_delta(&self, x: i64, y: i64, z: i64) -> Vec3 {
        if x == 0 && y == 0 && z == 0 {
            return self.base;
        }
        let d = if x == 0 { self.base.x } else { Self::decode(Self::encode(self.base.x) + x) };
        let d1 = if y == 0 { self.base.y } else { Self::decode(Self::encode(self.base.y) + y) };
        let d2 = if z == 0 { self.base.z } else { Self::decode(Self::encode(self.base.z) + z) };
        Vec3::new(d, d1, d2)
    }

    /// `encodeX` — дельта X.
    pub fn encode_x(&self, value: &Vec3) -> i64 {
        Self::encode(value.x) - Self::encode(self.base.x)
    }

    /// `encodeY` — дельта Y.
    pub fn encode_y(&self, value: &Vec3) -> i64 {
        Self::encode(value.y) - Self::encode(self.base.y)
    }

    /// `encodeZ` — дельта Z.
    pub fn encode_z(&self, value: &Vec3) -> i64 {
        Self::encode(value.z) - Self::encode(self.base.z)
    }

    /// `delta` — `value - base`.
    pub fn delta(&self, value: &Vec3) -> Vec3 {
        value.subtract_vec(&self.base)
    }

    /// `setBase`.
    pub fn set_base(&mut self, base: Vec3) {
        self.base = base;
    }

    /// `getBase`.
    pub fn get_base(&self) -> Vec3 {
        self.base
    }
}

impl Default for VecDeltaCodec {
    fn default() -> Self {
        Self::new()
    }
}
