//! `net.minecraft.world.entity.MobCategory` — категории спавна мобов.

use crate::util::string_representable::StringRepresentable;

/// `MobCategory`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MobCategory {
    /// `MONSTER`.
    Monster,
    /// `CREATURE`.
    Creature,
    /// `AMBIENT`.
    Ambient,
    /// `AXOLOTLS`.
    Axolotls,
    /// `UNDERGROUND_WATER_CREATURE`.
    UndergroundWaterCreature,
    /// `WATER_CREATURE`.
    WaterCreature,
    /// `WATER_AMBIENT`.
    WaterAmbient,
    /// `MISC`.
    Misc,
}

impl MobCategory {
    /// Все значения в порядке объявления.
    pub const VALUES: [MobCategory; 8] = [
        MobCategory::Monster,
        MobCategory::Creature,
        MobCategory::Ambient,
        MobCategory::Axolotls,
        MobCategory::UndergroundWaterCreature,
        MobCategory::WaterCreature,
        MobCategory::WaterAmbient,
        MobCategory::Misc,
    ];

    fn meta(&self) -> (&'static str, i32, bool, bool, i32) {
        match self {
            MobCategory::Monster => ("monster", 70, false, false, 128),
            MobCategory::Creature => ("creature", 10, true, true, 128),
            MobCategory::Ambient => ("ambient", 15, true, false, 128),
            MobCategory::Axolotls => ("axolotls", 5, true, false, 128),
            MobCategory::UndergroundWaterCreature => ("underground_water_creature", 5, true, false, 128),
            MobCategory::WaterCreature => ("water_creature", 5, true, false, 128),
            MobCategory::WaterAmbient => ("water_ambient", 20, true, false, 64),
            MobCategory::Misc => ("misc", -1, true, true, 128),
        }
    }

    /// `getName`.
    pub fn get_name(&self) -> &'static str {
        self.meta().0
    }

    /// `getMaxInstancesPerChunk`.
    pub fn get_max_instances_per_chunk(&self) -> i32 {
        self.meta().1
    }

    /// `isFriendly`.
    pub fn is_friendly(&self) -> bool {
        self.meta().2
    }

    /// `isPersistent`.
    pub fn is_persistent(&self) -> bool {
        self.meta().3
    }

    /// `getDespawnDistance`.
    pub fn get_despawn_distance(&self) -> i32 {
        self.meta().4
    }

    /// `getNoDespawnDistance` — всегда 32.
    pub fn get_no_despawn_distance(&self) -> i32 {
        32
    }
}

impl StringRepresentable for MobCategory {
    fn get_serialized_name(&self) -> String {
        self.get_name().to_string()
    }
}
