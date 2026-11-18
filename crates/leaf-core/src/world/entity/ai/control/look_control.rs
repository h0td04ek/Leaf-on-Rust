//! Перенос `net.minecraft.world.entity.ai.control.LookControl`.

use crate::util::mth;
use crate::world::entity::ai::control::control::rotate_towards;
use crate::world::entity::entity::Entity;
use crate::world::entity::mob::Mob;
use crate::world::phys::vec3::Vec3;

/// `LookControl`.
#[derive(Default)]
pub struct LookControl {
    /// `yMaxRotSpeed`.
    pub y_max_rot_speed: f32,
    /// `xMaxRotAngle`.
    pub x_max_rot_angle: f32,
    /// `lookAtCooldown`.
    pub look_at_cooldown: i32,
    /// `wantedX/Y/Z`.
    pub wanted_x: f64,
    pub wanted_y: f64,
    pub wanted_z: f64,
}

impl LookControl {
    /// `setLookAt(Vec3)` — delta из параметров моба.
    pub fn set_look_at_vec(&mut self, mob: &Mob, v: Vec3) {
        self.set_look_at(mob, v.x, v.y, v.z);
    }

    /// `setLookAt(Entity)` — взгляд на глаза сущности.
    pub fn set_look_at_entity(&mut self, mob: &Mob, entity: &Entity) {
        self.set_look_at(mob, entity.get_x(), entity.get_eye_y(), entity.get_z());
    }

    /// `setLookAt(Entity, deltaYaw, deltaPitch)`.
    pub fn set_look_at_entity_deltas(&mut self, entity: &Entity, dyaw: f32, dpitch: f32) {
        self.set_look_at_deltas(entity.get_x(), entity.get_eye_y(), entity.get_z(), dyaw, dpitch);
    }

    /// `setLookAt(x, y, z)` — delta = `mob.getHeadRotSpeed()`/
    /// `mob.getMaxHeadXRot()` (как в Java через `this.mob`).
    pub fn set_look_at(&mut self, mob: &Mob, x: f64, y: f64, z: f64) {
        let dyaw = mob.get_head_rot_speed();
        let dpitch = mob.get_max_head_x_rot();
        self.set_look_at_deltas(x, y, z, dyaw, dpitch);
    }

    /// `setLookAt(x, y, z, deltaYaw, deltaPitch)`.
    pub fn set_look_at_deltas(&mut self, x: f64, y: f64, z: f64, dyaw: f32, dpitch: f32) {
        self.wanted_x = x;
        self.wanted_y = y;
        self.wanted_z = z;
        self.y_max_rot_speed = dyaw;
        self.x_max_rot_angle = dpitch;
        self.look_at_cooldown = 2;
    }

    /// `tick()`.
    pub fn tick(&mut self, mob: &mut Mob) {
        if self.reset_x_rot_on_tick() {
            mob.living.entity.set_x_rot(0.0);
        }
        if self.look_at_cooldown > 0 {
            self.look_at_cooldown -= 1;
            if let Some(rotation_wanted) = self.get_y_rot_d(mob) {
                mob.living.y_head_rot = rotate_towards(
                    mob.living.y_head_rot,
                    rotation_wanted,
                    self.y_max_rot_speed,
                );
            }
            if let Some(p) = self.get_x_rot_d(mob) {
                let new_pitch = rotate_towards(mob.living.entity.get_x_rot(), p, self.x_max_rot_angle);
                mob.living.entity.set_x_rot(new_pitch);
            }
        } else {
            mob.living.y_head_rot = rotate_towards(
                mob.living.y_head_rot,
                mob.living.y_body_rot,
                10.0,
            );
        }
        self.clamp_head_rotation_to_body(mob);
    }

    /// `clampHeadRotationToBody`.
    fn clamp_head_rotation_to_body(&self, mob: &mut Mob) {
        if !mob.navigation_is_done() {
            mob.living.y_head_rot = mth::rotate_if_necessary(
                mob.living.y_head_rot,
                mob.living.y_body_rot,
                mob.get_max_head_y_rot(),
            );
        }
    }

    /// `resetXRotOnTick`.
    fn reset_x_rot_on_tick(&self) -> bool {
        true
    }

    /// `isLookingAtTarget`.
    pub fn is_looking_at_target(&self) -> bool {
        self.look_at_cooldown > 0
    }

    /// `getWantedX/Y/Z`.
    pub fn wanted_x(&self) -> f64 { self.wanted_x }
    pub fn wanted_y(&self) -> f64 { self.wanted_y }
    pub fn wanted_z(&self) -> f64 { self.wanted_z }

    /// `getXRotD`.
    fn get_x_rot_d(&self, mob: &Mob) -> Option<f32> {
        let d = self.wanted_x - mob.living.entity.get_x();
        let d1 = self.wanted_y - mob.living.entity.get_eye_y();
        let d2 = self.wanted_z - mob.living.entity.get_z();
        let square_root = mth::sqrt_f64(d * d + d2 * d2);
        if d1.abs() <= 1.0e-5 && square_root.abs() <= 1.0e-5 {
            None
        } else {
            Some(-(mth::atan2(d1, square_root) as f32 * 180.0 / std::f32::consts::PI))
        }
    }

    /// `getYRotD`.
    fn get_y_rot_d(&self, mob: &Mob) -> Option<f32> {
        let d = self.wanted_x - mob.living.entity.get_x();
        let d1 = self.wanted_z - mob.living.entity.get_z();
        if d1.abs() <= 1.0e-5 && d.abs() <= 1.0e-5 {
            None
        } else {
            Some((mth::atan2(d1, d) as f32 * 180.0 / std::f32::consts::PI) - 90.0)
        }
    }
}
