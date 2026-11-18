//! Перенос `net.minecraft.world.entity.ai.control.FlyingMoveControl`.
//!
//! В Java — наследник `MoveControl` с переопределённым `tick`.
//! Здесь `MoveControl` получил флаг `flying: Option<FlyingParams>`
//! и диспетчеризацию в `tick` (семантика идентична).

use crate::entity::ai::attributes::attributes;
use crate::util::mth;
use crate::world::entity::mob::Mob;

/// Параметры `FlyingMoveControl` (`maxTurn`, `hoversInPlace`).
#[derive(Clone, Copy, Debug)]
pub struct FlyingMoveControl {
    /// `maxTurn`.
    pub max_turn: i32,
    /// `hoversInPlace`.
    pub hovers_in_place: bool,
}

impl FlyingMoveControl {
    /// `tick()` летающего режима (`MoveControl.tick` с
    /// переопределением — порт `FlyingMoveControl.tick`).
    pub fn tick(&self, mc: &mut super::move_control::MoveControl, mob: &mut Mob) {
        use super::move_control::MoveOperation;
        if mc.operation == MoveOperation::MoveTo {
            mc.operation = MoveOperation::Wait;
            mob.living.entity.no_gravity = true;
            let d = mc.wanted_x - mob.living.entity.get_x();
            let d1 = mc.wanted_y - mob.living.entity.get_y();
            let d2 = mc.wanted_z - mob.living.entity.get_z();
            let d3 = d * d + d1 * d1 + d2 * d2;
            if d3 < MoveControl_min_speed_sqr() {
                mob.living.set_yya(0.0);
                mob.living.set_zza(0.0);
                return;
            }
            let f = (mth::atan2(d2, d) as f32 * 180.0 / std::f32::consts::PI) - 90.0;
            let new_yaw = mc.rotlerp(mob.living.entity.get_y_rot(), f, 90.0);
            mob.living.entity.set_y_rot(new_yaw);
            let f1 = if mob.living.entity.on_ground {
                (mc.speed_modifier as f32)
                    * mob.living.get_attribute_value_by_id(&attributes::movement_speed()) as f32
            } else {
                (mc.speed_modifier as f32)
                    * mob.living.get_attribute_value_by_id(&attributes::flying_speed()) as f32
            };
            mob.living.set_speed(f1);
            let square_root = mth::sqrt_f64(d * d + d2 * d2);
            if d1.abs() > 1.0e-5 || square_root.abs() > 1.0e-5 {
                let f2 = -(mth::atan2(d1, square_root) as f32 * 180.0 / std::f32::consts::PI);
                let new_pitch = mc.rotlerp(mob.living.entity.get_x_rot(), f2, self.max_turn as f32);
                mob.living.entity.set_x_rot(new_pitch);
                mob.living.set_yya(if d1 > 0.0 { f1 } else { -f1 });
            }
        } else {
            if !self.hovers_in_place {
                mob.living.entity.no_gravity = false;
            }
            mob.living.set_yya(0.0);
            mob.living.set_zza(0.0);
        }
    }
}

fn MoveControl_min_speed_sqr() -> f64 {
    super::move_control::MoveControl::MIN_SPEED_SQR as f64
}
