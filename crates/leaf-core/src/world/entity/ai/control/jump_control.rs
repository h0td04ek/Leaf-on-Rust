//! Перенос `net.minecraft.world.entity.ai.control.JumpControl`.

use crate::world::entity::mob::Mob;

/// `JumpControl`.
#[derive(Default)]
pub struct JumpControl {
    /// `jump`.
    pub jump: bool,
}

impl JumpControl {
    /// `jump()`.
    pub fn jump(&mut self) {
        self.jump = true;
    }

    /// `tick()`.
    pub fn tick(&mut self, mob: &mut Mob) {
        mob.living.jumping = self.jump;
        self.jump = false;
    }
}
