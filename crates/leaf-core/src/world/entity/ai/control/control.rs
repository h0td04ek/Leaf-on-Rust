//! Перенос `net.minecraft.world.entity.ai.control.Control`.

use crate::util::mth;

/// `Control` — маркер-интерфейс контролов + `rotateTowards`.
pub trait Control {}

/// `Control.rotateTowards(current, wanted, maxSpeed)`.
pub fn rotate_towards(current: f32, wanted: f32, max_speed: f32) -> f32 {
    let f = mth::degrees_difference(current, wanted);
    let f1 = mth::clamp_f32(f, -max_speed, max_speed);
    current + f1
}
