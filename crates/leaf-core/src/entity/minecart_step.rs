//! Перенос `net.minecraft.world.entity.vehicle.NewMinecartBehavior.MinecartStep`
//! (network-часть record'а).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::util::mth;
use crate::world::phys::vec3::Vec3;

/// `MinecartStep` — шаг лерпа minecart'а: позиция, движение, повороты, вес.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinecartStep {
    /// `position`.
    pub position: Vec3,
    /// `movement`.
    pub movement: Vec3,
    /// `yRot`.
    pub y_rot: f32,
    /// `xRot`.
    pub x_rot: f32,
    /// `weight`.
    pub weight: f32,
}

impl MinecartStep {
    /// `new MinecartStep(position, movement, yRot, xRot, weight)`.
    pub fn new(position: Vec3, movement: Vec3, y_rot: f32, x_rot: f32, weight: f32) -> Self {
        Self { position, movement, y_rot, x_rot, weight }
    }

    /// `ZERO`.
    pub fn zero() -> Self {
        Self {
            position: Vec3::ZERO,
            movement: Vec3::ZERO,
            y_rot: 0.0,
            x_rot: 0.0,
            weight: 0.0,
        }
    }

    /// `STREAM_CODEC` — Vec3 + Vec3 + ROTATION_BYTE + ROTATION_BYTE + FLOAT.
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let position = buf.read_vec3();
        let movement = buf.read_vec3();
        let y_rot = mth::unpack_degrees(buf.read_byte());
        let x_rot = mth::unpack_degrees(buf.read_byte());
        let weight = buf.read_float();
        Self::new(position, movement, y_rot, x_rot, weight)
    }

    /// `write`.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_vec3(&self.position);
        buf.write_vec3(&self.movement);
        buf.write_byte(mth::pack_degrees(self.y_rot) as i32);
        buf.write_byte(mth::pack_degrees(self.x_rot) as i32);
        buf.write_float(self.weight);
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MinecartStep> {
        crate::network::codec::stream_codec::FnCodec::new(
            |buf: &mut FriendlyByteBuf, s: &MinecartStep| s.write(buf),
            |buf: &mut FriendlyByteBuf| MinecartStep::read(buf),
        )
    }
}
