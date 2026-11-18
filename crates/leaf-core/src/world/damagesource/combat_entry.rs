//! Перенос `net.minecraft.world.damagesource.CombatEntry` — запись боя.

use crate::world::damagesource::damage_source::DamageSource;
use crate::world::damagesource::fall_location::FallLocation;

/// `CombatEntry` — запись о полученном уроне (source, damage, fallLocation, fallDistance).
#[derive(Clone)]
pub struct CombatEntry {
    /// `source` — источник урона.
    pub source: DamageSource,
    /// `damage` — количество урона.
    pub damage: f32,
    /// `fallLocation` — место падения.
    pub fall_location: Option<FallLocation>,
    /// `fallDistance` — дистанция падения.
    pub fall_distance: f32,
}

impl CombatEntry {
    /// `new(source, damage, fallLocation, fallDistance)`.
    pub fn new(
        source: DamageSource,
        damage: f32,
        fall_location: Option<FallLocation>,
        fall_distance: f32,
    ) -> Self {
        Self {
            source,
            damage,
            fall_location,
            fall_distance,
        }
    }

    /// `source()`.
    pub fn get_source(&self) -> &DamageSource {
        &self.source
    }

    /// `fallLocation()`.
    pub fn get_fall_location(&self) -> Option<FallLocation> {
        self.fall_location
    }

    /// `fallDistance()`.
    pub fn get_fall_distance(&self) -> f32 {
        self.fall_distance
    }

    /// `damage()`.
    pub fn get_damage(&self) -> f32 {
        self.damage
    }
}
