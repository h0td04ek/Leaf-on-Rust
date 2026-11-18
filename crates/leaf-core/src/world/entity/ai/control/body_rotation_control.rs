//! Перенос `net.minecraft.world.entity.ai.control.BodyRotationControl`.

use crate::util::mth;
use crate::world::entity::mob::Mob;

/// `BodyRotationControl`.
#[derive(Default)]
pub struct BodyRotationControl {
    /// `headStableTime`.
    pub head_stable_time: i32,
    /// `lastStableYHeadRot`.
    pub last_stable_y_head_rot: f32,
}

impl BodyRotationControl {
    /// Константы vanilla.
    const HEAD_STABLE_ANGLE: f32 = 15.0;
    const DELAY_UNTIL_STARTING_TO_FACE_FORWARD: i32 = 10;
    const HOW_LONG_IT_TAKES_TO_FACE_FORWARD: i32 = 10;

    /// `clientTick()`.
    pub fn client_tick(&mut self, mob: &mut Mob) {
        if self.is_moving(mob) {
            mob.living.y_body_rot = mob.living.entity.get_y_rot();
            self.rotate_head_if_necessary(mob);
            self.last_stable_y_head_rot = mob.living.y_head_rot;
            self.head_stable_time = 0;
        } else if self.not_carrying_mob_passengers(mob) {
            if (mob.living.y_head_rot - self.last_stable_y_head_rot).abs()
                > Self::HEAD_STABLE_ANGLE
            {
                self.head_stable_time = 0;
                self.last_stable_y_head_rot = mob.living.y_head_rot;
                self.rotate_body_if_necessary(mob);
            } else {
                self.head_stable_time += 1;
                if self.head_stable_time > Self::DELAY_UNTIL_STARTING_TO_FACE_FORWARD {
                    self.rotate_head_towards_front(mob);
                }
            }
        }
    }

    /// `rotateBodyIfNecessary`.
    fn rotate_body_if_necessary(&self, mob: &mut Mob) {
        mob.living.y_body_rot = mth::rotate_if_necessary(
            mob.living.y_body_rot,
            mob.living.y_head_rot,
            mob.get_max_head_y_rot(),
        );
    }

    /// `rotateHeadIfNecessary`.
    fn rotate_head_if_necessary(&self, mob: &mut Mob) {
        mob.living.y_head_rot = mth::rotate_if_necessary(
            mob.living.y_head_rot,
            mob.living.y_body_rot,
            mob.get_max_head_y_rot(),
        );
    }

    /// `rotateHeadTowardsFront`.
    fn rotate_head_towards_front(&self, mob: &mut Mob) {
        let i = self.head_stable_time - Self::DELAY_UNTIL_STARTING_TO_FACE_FORWARD;
        let f = mth::clamp_f32(
            i as f32 / Self::HOW_LONG_IT_TAKES_TO_FACE_FORWARD as f32,
            0.0,
            1.0,
        );
        let f1 = mob.get_max_head_y_rot() * (1.0 - f);
        mob.living.y_body_rot = mth::rotate_if_necessary(
            mob.living.y_body_rot,
            mob.living.y_head_rot,
            f1,
        );
    }

    /// `notCarryingMobPassengers`.
    fn not_carrying_mob_passengers(&self, mob: &Mob) -> bool {
        !mob.first_passenger_is_mob()
    }

    /// `isMoving` — смещение по X/Z от `xo`/`zo`.
    fn is_moving(&self, mob: &Mob) -> bool {
        let d = mob.living.entity.get_x() - mob.living.entity.old_pos.x;
        let d1 = mob.living.entity.get_z() - mob.living.entity.old_pos.z;
        d * d + d1 * d1 > 2.5000003e-7
    }
}
