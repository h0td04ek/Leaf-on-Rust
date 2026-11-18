//! Перенос `net.minecraft.world.entity.ai.sensing.Sensing`.

use crate::world::entity::entity::Entity;
use crate::world::entity::mob::MobPathing;
use std::collections::HashSet;

/// `Sensing` — senses видимости сущностей (LOS-кэш на тик).
#[derive(Default)]
pub struct Sensing {
    /// `seen` — id сущностей с видимой линией зрения.
    seen: HashSet<i32>,
    /// `unseen` — id сущностей без LOS.
    unseen: HashSet<i32>,
}

impl Sensing {
    /// `new`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `tick` — сброс кэшей каждый тик.
    pub fn tick(&mut self) {
        self.seen.clear();
        self.unseen.clear();
    }

    /// `hasLineOfSight(entity)` — с кэшем seen/unseen.
    pub fn has_line_of_sight(&mut self, mob: &dyn MobPathing, entity: &Entity) -> bool {
        let id = entity.get_id();
        if self.seen.contains(&id) {
            return true;
        }
        if self.unseen.contains(&id) {
            return false;
        }
        let has_los = mob.has_line_of_sight(entity);
        if has_los {
            self.seen.insert(id);
        } else {
            self.unseen.insert(id);
        }
        has_los
    }
}
