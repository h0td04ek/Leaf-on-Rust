//! Перенос `net.minecraft.world.entity.PositionMoveRotation`.

use std::collections::BTreeSet;

use crate::entity::relative::Relative;
use crate::util::mth;
use crate::world::phys::vec3::Vec3;

/// `PositionMoveRotation` — позиция + дельта-движение + поворот.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionMoveRotation {
    /// `position`.
    pub position: Vec3,
    /// `deltaMovement`.
    pub delta_movement: Vec3,
    /// `yRot`.
    pub y_rot: f32,
    /// `xRot`.
    pub x_rot: f32,
}

impl PositionMoveRotation {
    pub fn new(position: Vec3, delta_movement: Vec3, y_rot: f32, x_rot: f32) -> Self {
        Self { position, delta_movement, y_rot, x_rot }
    }

    /// `withRotation(yRot, xRot)`.
    pub fn with_rotation(&self, y_rot: f32, x_rot: f32) -> Self {
        Self { y_rot, x_rot, ..*self }
    }

    /// `calculateAbsolute(current, after, relatives)`.
    pub fn calculate_absolute(
        current: &PositionMoveRotation,
        after: &PositionMoveRotation,
        relatives: &BTreeSet<Relative>,
    ) -> PositionMoveRotation {
        let d = if relatives.contains(&Relative::X) { current.position.x } else { 0.0 };
        let d1 = if relatives.contains(&Relative::Y) { current.position.y } else { 0.0 };
        let d2 = if relatives.contains(&Relative::Z) { current.position.z } else { 0.0 };
        let f = if relatives.contains(&Relative::YRot) { current.y_rot } else { 0.0 };
        let f1 = if relatives.contains(&Relative::XRot) { current.x_rot } else { 0.0 };
        let vec3 = Vec3::new(d + after.position.x, d1 + after.position.y, d2 + after.position.z);
        let f2 = f + after.y_rot;
        let f3 = mth::clamp_f32(f1 + after.x_rot, -90.0, 90.0);

        let mut vec31 = current.delta_movement;
        if relatives.contains(&Relative::RotateDelta) {
            let f4 = (current.y_rot - f2).to_radians();
            let f5 = (current.x_rot - f3).to_radians();
            vec31 = vec31.x_rot(f5 as f32);
            vec31 = vec31.y_rot(f4 as f32);
        }

        let vec32 = Vec3::new(
            Self::calculate_delta(vec31.x, after.delta_movement.x, relatives, Relative::DeltaX),
            Self::calculate_delta(vec31.y, after.delta_movement.y, relatives, Relative::DeltaY),
            Self::calculate_delta(vec31.z, after.delta_movement.z, relatives, Relative::DeltaZ),
        );
        PositionMoveRotation::new(vec3, vec32, f2, f3)
    }

    fn calculate_delta(
        position: f64,
        delta_movement: f64,
        relatives: &BTreeSet<Relative>,
        delta_relative: Relative,
    ) -> f64 {
        if relatives.contains(&delta_relative) {
            position + delta_movement
        } else {
            delta_movement
        }
    }

    /// `STREAM_CODEC` — Vec3×2 (3 double каждый) + FLOAT×2.
    pub fn read(buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf) -> Self {
        let position = buf.read_vec3();
        let delta_movement = buf.read_vec3();
        let y_rot = buf.read_float();
        let x_rot = buf.read_float();
        Self::new(position, delta_movement, y_rot, x_rot)
    }

    /// `write`.
    pub fn write(&self, buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf) {
        buf.write_vec3(&self.position);
        buf.write_vec3(&self.delta_movement);
        buf.write_float(self.y_rot);
        buf.write_float(self.x_rot);
    }
}
