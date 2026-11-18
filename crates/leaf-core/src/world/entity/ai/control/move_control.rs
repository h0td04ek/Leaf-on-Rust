//! Перенос `net.minecraft.world.entity.ai.control.MoveControl`.

use crate::core::block_pos::BlockPos;
use crate::tags::block_tags::BlockTags;
use crate::util::mth;
use crate::entity::ai::attributes::attributes;
use crate::world::entity::mob::Mob;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::pathfinder::node_evaluator::NodeEvaluator;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::phys::shapes::VoxelShape;

/// `MoveControl.Operation`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MoveOperation {
    /// `WAIT`.
    #[default]
    Wait,
    /// `MOVE_TO`.
    MoveTo,
    /// `STRAFE`.
    Strafe,
    /// `JUMPING`.
    Jumping,
}

/// `MoveControl`.
#[derive(Default)]
pub struct MoveControl {
    /// `wantedX/Y/Z`.
    pub wanted_x: f64,
    pub wanted_y: f64,
    pub wanted_z: f64,
    /// `speedModifier`.
    pub speed_modifier: f64,
    /// `strafeForwards`, `strafeRight`.
    pub strafe_forwards: f32,
    pub strafe_right: f32,
    /// `operation`.
    pub operation: MoveOperation,
}

impl MoveControl {
    /// `MIN_SPEED` / `MIN_SPEED_SQR` / `MAX_TURN`.
    pub const MIN_SPEED: f32 = 5.0e-4;
    pub const MIN_SPEED_SQR: f32 = 2.5000003e-7;
    pub const MAX_TURN: f32 = 90.0;

    /// `hasWanted`.
    pub fn has_wanted(&self) -> bool {
        self.operation == MoveOperation::MoveTo
    }

    /// `getSpeedModifier`.
    pub fn speed_modifier(&self) -> f64 {
        self.speed_modifier
    }

    /// `setWantedPosition`.
    pub fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.wanted_x = x;
        self.wanted_y = y;
        self.wanted_z = z;
        self.speed_modifier = speed_modifier;
        if self.operation != MoveOperation::Jumping {
            self.operation = MoveOperation::MoveTo;
        }
    }

    /// `strafe`.
    pub fn strafe(&mut self, forward: f32, strafe: f32) {
        self.operation = MoveOperation::Strafe;
        self.strafe_forwards = forward;
        self.strafe_right = strafe;
        self.speed_modifier = 0.25;
    }

    /// `tick()`.
    pub fn tick(&mut self, mob: &mut Mob) {
        match self.operation {
            MoveOperation::Strafe => {
                let f = mob.living.get_attribute_value_by_id(&attributes::movement_speed()) as f32;
                let f1 = (self.speed_modifier as f32) * f;
                let mut f2 = self.strafe_forwards;
                let mut f3 = self.strafe_right;
                let mut square_root = mth::sqrt_f32(f2 * f2 + f3 * f3);
                if square_root < 1.0 {
                    square_root = 1.0;
                }
                square_root = f1 / square_root;
                f2 *= square_root;
                f3 *= square_root;
                let sin = mth::sin(mob.living.entity.get_y_rot() as f64 * (std::f64::consts::PI / 180.0));
                let cos = mth::cos(mob.living.entity.get_y_rot() as f64 * (std::f64::consts::PI / 180.0));
                let f4 = f2 * cos - f3 * sin;
                let f5 = f3 * cos + f2 * sin;
                if !self.is_walkable(mob, f4, f5) {
                    self.strafe_forwards = 1.0;
                    self.strafe_right = 0.0;
                }
                mob.living.set_speed(f1);
                mob.living.set_zza(self.strafe_forwards);
                mob.living.set_xxa(self.strafe_right);
                self.operation = MoveOperation::Wait;
            }
            MoveOperation::MoveTo => {
                self.operation = MoveOperation::Wait;
                let d = self.wanted_x - mob.living.entity.get_x();
                let d1 = self.wanted_z - mob.living.entity.get_z();
                let d2 = self.wanted_y - mob.living.entity.get_y();
                let d3 = d * d + d2 * d2 + d1 * d1;
                if d3 < Self::MIN_SPEED_SQR as f64 {
                    mob.living.set_zza(0.0);
                    return;
                }
                let f5 = (mth::atan2(d1, d) as f32 * 180.0 / std::f32::consts::PI) - 90.0;
                let new_yaw = self.rotlerp(mob.living.entity.get_y_rot(), f5, Self::MAX_TURN);
                mob.living.entity.set_y_rot(new_yaw);
                let speed = (self.speed_modifier as f32)
                    * mob.living.get_attribute_value_by_id(&attributes::movement_speed()) as f32;
                mob.living.set_speed(speed);
                let block_pos = mob.living.block_position();
                let Some(level) = mob.level() else { return };
                let block_state = level.get_block_state(block_pos).unwrap_or_else(BlockState::air);
                let collision_shape = block_state.get_collision_shape(level.as_ref(), block_pos);
                if d2 > mob.living.max_up_step() as f64
                    && d * d + d1 * d1 < (mob.living.entity.dimensions.width.max(1.0)) as f64
                    || (!collision_shape.is_empty()
                        && mob.living.entity.get_y()
                            < collision_shape.max(crate::core::direction::Axis::Y) + block_pos.y as f64
                        && !block_state.is_block_tag(&BlockTags::doors())
                        && !block_state.is_block_tag(&BlockTags::fences()))
                {
                    mob.jump_control.jump();
                    self.operation = MoveOperation::Jumping;
                }
            }
            MoveOperation::Jumping => {
                let speed = (self.speed_modifier as f32)
                    * mob.living.get_attribute_value_by_id(&attributes::movement_speed()) as f32;
                mob.living.set_speed(speed);
                if mob.living.entity.on_ground || (mob.living.is_in_water() && mob.living.is_affected_by_fluids()) {
                    self.operation = MoveOperation::Wait;
                }
            }
            MoveOperation::Wait => {
                mob.living.set_zza(0.0);
            }
        }
    }

    /// `isWalkable`.
    fn is_walkable(&self, mob: &Mob, relative_x: f32, relative_z: f32) -> bool {
        let Some(navigation) = mob.navigation_evaluator_mut() else {
            return true;
        };
        let pos = BlockPos::containing(
            mob.living.entity.get_x() + relative_x as f64,
            mob.living.entity.get_block_y() as f64,
            mob.living.entity.get_z() + relative_z as f64,
        );
        let pt = {
            let mut eval = navigation.borrow_mut();
            eval.get_path_type_at(mob, pos)
        };
        pt != PathType::Walkable
    }

    /// `rotlerp`.
    pub fn rotlerp(&self, source_angle: f32, target_angle: f32, maximum_change: f32) -> f32 {
        let f = mth::wrap_degrees_f32(target_angle - source_angle);
        let f = f.clamp(-maximum_change, maximum_change);
        let mut f1 = source_angle + f;
        if f1 < 0.0 {
            f1 += 360.0;
        } else if f1 > 360.0 {
            f1 -= 360.0;
        }
        f1
    }

    /// `getWantedX/Y/Z`.
    pub fn wanted_x(&self) -> f64 { self.wanted_x }
    pub fn wanted_y(&self) -> f64 { self.wanted_y }
    pub fn wanted_z(&self) -> f64 { self.wanted_z }

    /// `setWait`.
    pub fn set_wait(&mut self) {
        self.operation = MoveOperation::Wait;
    }
}
