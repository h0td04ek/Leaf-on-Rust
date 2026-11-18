//! Перенос `net.minecraft.world.entity.ExperienceOrb` — сфера опыта.

use crate::core::holder::Holder;
use crate::world::entity::entity::Entity;
use crate::world::entity::entity_type;
use crate::world::phys::vec3::Vec3;
use std::sync::Arc;

/// `ExperienceOrb` — сущность-сфера опыта.
#[derive(Clone)]
pub struct ExperienceOrb {
    /// базовая `Entity`.
    pub entity: Entity,
    /// `value` — количество опыта в сфере.
    pub value: i32,
    /// `age` — возраст.
    pub age: i32,
    /// `health` — "прочность".
    pub health: i32,
    /// `count` — счётчик для merge-логики.
    pub count: i32,
}

impl ExperienceOrb {
    /// `new(level, x, y, z, value)`.
    pub fn new(
        level: Option<Arc<dyn crate::world::level::level_reader::LevelReader>>,
        x: f64,
        y: f64,
        z: f64,
        value: i32,
    ) -> Self {
        let mut entity = Entity::new(
            Holder::Direct(entity_type::experience_orb_entity_type()),
            level,
        );
        entity.pos = Vec3::new(x, y, z);
        Self { entity, value, age: 0, health: 5, count: 0 }
    }

    /// `getValue()`.
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// `award(level, pos, amount)` — разбиение на сферы
    /// ванильными номиналами (2477/1237/617/307/149/73/37/17/7/3/1).
    pub fn award(
        level: Option<&Arc<dyn crate::world::level::level_reader::LevelReader>>,
        pos: Vec3,
        mut amount: i32,
    ) {
        const VALUES: [i32; 11] = [2477, 1237, 617, 307, 149, 73, 37, 17, 7, 3, 1];
        while amount > 0 {
            let mut i = VALUES.len() - 1;
            while i > 0 && VALUES[i] > amount {
                i -= 1;
            }
            let v = VALUES[i];
            amount -= v;
            let orb = ExperienceOrb::new(level.cloned(), pos.x as f64, pos.y as f64, pos.z as f64, v);
            if let Some(l) = level {
                l.add_fresh_experience_orb(orb);
            }
        }
    }
}
