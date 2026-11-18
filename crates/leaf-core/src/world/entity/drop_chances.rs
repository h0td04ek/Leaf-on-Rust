//! Перенос `net.minecraft.world.entity.DropChances`.

use crate::entity::equipment_slot::EquipmentSlot;
use std::collections::HashMap;

/// `DropChances` — шансы дропа экипировки по слотам.
#[derive(Clone, Debug)]
pub struct DropChances {
    /// `byEquipment`.
    pub by_equipment: HashMap<EquipmentSlot, f32>,
}

impl DropChances {
    /// `DEFAULT_EQUIPMENT_DROP_CHANCE`.
    pub const DEFAULT_EQUIPMENT_DROP_CHANCE: f32 = 0.085;
    /// `PRESERVE_ITEM_DROP_CHANCE_THRESHOLD`.
    pub const PRESERVE_ITEM_DROP_CHANCE_THRESHOLD: f32 = 1.0;
    /// `PRESERVE_ITEM_DROP_CHANCE`.
    pub const PRESERVE_ITEM_DROP_CHANCE: f32 = 2.0;
    /// `DEFAULT`.
    pub fn default_chances() -> Self {
        let mut map = HashMap::new();
        for slot in EquipmentSlot::VALUES {
            map.insert(slot, Self::DEFAULT_EQUIPMENT_DROP_CHANCE);
        }
        Self { by_equipment: map }
    }

    /// `withGuaranteedDrop(slot)`.
    pub fn with_guaranteed_drop(&self, slot: EquipmentSlot) -> Self {
        self.with_equipment_chance(slot, Self::PRESERVE_ITEM_DROP_CHANCE)
    }

    /// `withEquipmentChance(slot, chance)`.
    pub fn with_equipment_chance(&self, slot: EquipmentSlot, chance: f32) -> Self {
        assert!(chance >= 0.0, "Tried to set invalid equipment chance {} for {:?}", chance, slot);
        if self.by_equipment(slot) == chance {
            return self.clone();
        }
        let mut map = HashMap::new();
        for s in EquipmentSlot::VALUES {
            map.insert(s, if s == slot { chance } else { self.by_equipment(s) });
        }
        Self { by_equipment: map }
    }

    /// `byEquipment(slot)`.
    pub fn by_equipment(&self, slot: EquipmentSlot) -> f32 {
        self.by_equipment
            .get(&slot)
            .copied()
            .unwrap_or(Self::DEFAULT_EQUIPMENT_DROP_CHANCE)
    }

    /// `isPreserved(slot)`.
    pub fn is_preserved(&self, slot: EquipmentSlot) -> bool {
        self.by_equipment(slot) > Self::PRESERVE_ITEM_DROP_CHANCE_THRESHOLD
    }
}

impl Default for DropChances {
    fn default() -> Self {
        Self::default_chances()
    }
}
