//! `FireworkRocketEntity` — фейерверк-ракета (`EntityType.FIREWORK_ROCKET`).
//!
//! Ванильная семантика: сущность привязана к выстрелившему
//! (`attachedToEntity`), после нулевого тика жизни получает
//! импульс по направлению взгляда владельца, живёт
//! `10 + random(6) + flightDuration*10 + 1` тиков и детонирует
//! (`explode` → звук + частицы + `level.explode` с радиусом 0).

use std::sync::Arc;

use crate::core::holder::Holder;
use crate::resources::identifier::Identifier;
use crate::world::entity::entity::Entity;
use crate::world::entity::entity_type;
use crate::world::item::component::fireworks::Fireworks;
use crate::world::item::item_stack::ItemStack;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;

/// `DATA_ID_FIREWORKS_ITEM`-эквивалент — `ItemStack` с
/// `fireworks`-компонентом.
const FIREWORKS_COMPONENT: &str = "fireworks";

/// `FireworkRocketEntity`.
#[derive(Clone)]
pub struct FireworkRocketEntity {
    /// базовая `Entity`.
    pub entity: Entity,
    /// `item` — стопка с `FIREWORKS`-компонентом.
    pub item: ItemStack,
    /// `attachedToEntity` — сущность-владелец (Option, т.к.
    /// Entity не имеет lifetime-связи с уровнем).
    pub attached_to: Option<Entity>,
    /// `life` — оставшиеся тики полёта (-1 = ещё не запущена).
    pub life: i32,
    /// `shotAtAngle` — выстрел под углом (crossbow).
    pub shot_at_angle: bool,
}

impl FireworkRocketEntity {
    /// `new(level, shooter, x, y, z, stack)`.
    pub fn new(
        level: Option<Arc<dyn LevelReader>>,
        shooter: Option<&Entity>,
        x: f64,
        y: f64,
        z: f64,
        stack: ItemStack,
    ) -> Self {
        let mut entity = Entity::new(
            Holder::Direct(entity_type::firework_rocket_entity_type()),
            level,
        );
        entity.pos = Vec3::new(x, y, z);
        Self {
            entity,
            item: stack,
            attached_to: shooter.cloned(),
            life: -1,
            shot_at_angle: false,
        }
    }

    /// `setShotAtAngle`.
    pub fn set_shot_at_angle(&mut self, v: bool) {
        self.shot_at_angle = v;
    }

    /// `getItem` — `DATA_ID_FIREWORKS_ITEM`.
    pub fn get_item(&self) -> &ItemStack {
        &self.item
    }

    /// `getFireworks` — `item.get(FIREWORKS)` или default.
    pub fn get_fireworks(&self) -> Fireworks {
        self.item
            .get_component::<Fireworks>(&Identifier::minecraft(FIREWORKS_COMPONENT))
            .unwrap_or_default()
    }

    /// `isAttachedToEntity`.
    pub fn is_attached_to_entity(&self) -> bool {
        self.attached_to.is_some()
    }

    /// `tick` — полёт и детонация. Вызывается level'ом для
    /// сущностей, добавленных через `add_fresh_firework_rocket`.
    pub fn tick(&mut self, level: &Arc<dyn LevelReader>) {
        // Привязка к владельцу: позиция = владелец + (0, eyeHeight, 0).
        if let Some(owner) = self.attached_to.clone() {
            let eye = owner.get_eye_height() as f64;
            self.entity.pos = Vec3::new(owner.pos.x, owner.pos.y + eye, owner.pos.z);
        }

        if self.life == 0 && self.attached_to.is_none() {
            // Запуск: импульс по направлению взгляда владельца.
            if let Some(owner) = &self.attached_to {
                let _ = owner;
            }
            let look = self.entity.get_look_angle();
            let (dx, dy, dz, life_bonus) = {
                let mut r = self.entity.random.lock().unwrap();
                let spread = 0.05;
                let dx = look.x + (r.next_double() - 0.5) * spread;
                let dy = look.y + (r.next_double() - 0.5) * spread;
                let dz = look.z + (r.next_double() - 0.5) * spread;
                (dx, dy, dz, r.next_int_bound(6))
            };
            let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-4);
            let power = 0.1;
            self.entity
                .set_delta_movement(dx / len * power, dy / len * power, dz / len * power);
            let flight = self.get_fireworks().flight_duration.max(0) as i32;
            self.life = 10 + life_bonus + flight * 10 + 1;
        }

        self.life -= 1;
        if self.life <= 0 {
            // Серверная сторона — level.rs не имеет client-режима.
            self.explode(level);
            self.entity.remove(crate::world::entity::entity::RemovalReason::Discarded);
        }
    }

    /// `explode` — детонация: для каждого `FireworkExplosion`
    /// звук (blast/twinkle по `hasTrail`) + `level.explode`
    /// с радиусом 0 (визуальный взрыв без урона).
    pub fn explode(&self, level: &Arc<dyn LevelReader>) {
        let fireworks = self.get_fireworks();
        if fireworks.explosions.is_empty() {
            return;
        }
        let pos = self.entity.position();
        let sources = crate::world::damagesource::damage_sources::DamageSources::vanilla();
        let damage_source = sources.fireworks(
            &self.entity,
            self.attached_to.as_ref(),
        );
        level.explode(
            Some(self.entity.clone()),
            Some(damage_source),
            pos,
            0.0,
            false,
            crate::world::item::enchantment::effects::enchantment_entity_effect::ExplosionInteraction::None,
            None,
        );
        // Звук детонации: blast (twinkle если hasTrail у первого).
        let first_trail = fireworks
            .explosions
            .first()
            .map(|e| e.has_trail)
            .unwrap_or(false);
        let sound_id = if first_trail {
            "entity.firework_rocket.twinkle"
        } else {
            "entity.firework_rocket.blast"
        };
        level.play_sound(
            pos,
            &Identifier::minecraft(sound_id),
            1.0,
            1.0,
        );
    }
}
