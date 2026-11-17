//! Перенос `net.minecraft.util.SegmentedAnglePrecision`.

use crate::core::direction::Direction;

/// `SegmentedAnglePrecision(precision)` — дискретизация углов в биты.
pub struct SegmentedAnglePrecision {
    mask: i32,
    precision: i32,
    degree_to_angle: f32,
    angle_to_degree: f32,
}

impl SegmentedAnglePrecision {
    pub fn new(precision: i32) -> Self {
        if precision < 2 {
            panic!("Precision cannot be less than 2 bits");
        }
        if precision > 30 {
            panic!("Precision cannot be greater than 30 bits");
        }
        let i = 1i32 << precision;
        Self {
            mask: i - 1,
            precision,
            degree_to_angle: i as f32 / 360.0,
            angle_to_degree: 360.0 / i as f32,
        }
    }

    /// `isSameAxis` — одинаковая ли ось у двух направлений.
    pub fn is_same_axis(&self, first: i32, second: i32) -> bool {
        let i = self.mask >> 1;
        (first & i) == (second & i)
    }

    /// `fromDirection(Direction)`.
    pub fn from_direction(&self, direction: &Direction) -> i32 {
        if direction.get_axis().is_vertical() {
            0
        } else {
            direction.get_2d_data_value() << (self.precision - 2)
        }
    }

    pub fn from_degrees_with_turns(&self, degrees_with_turns: f32) -> i32 {
        (degrees_with_turns * self.degree_to_angle).round() as i32
    }

    pub fn from_degrees(&self, degrees: f32) -> i32 {
        self.normalize(self.from_degrees_with_turns(degrees))
    }

    pub fn to_degrees_with_turns(&self, degrees: i32) -> f32 {
        degrees as f32 * self.angle_to_degree
    }

    pub fn to_degrees(&self, degrees_with_turns: i32) -> f32 {
        let f = self.to_degrees_with_turns(self.normalize(degrees_with_turns));
        if f >= 180.0 { f - 360.0 } else { f }
    }

    pub fn normalize(&self, degrees: i32) -> i32 {
        degrees & self.mask
    }

    pub fn get_mask(&self) -> i32 {
        self.mask
    }
}
