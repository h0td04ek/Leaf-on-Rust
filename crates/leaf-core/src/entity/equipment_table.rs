//! Перенос `net.minecraft.world.entity.EquipmentTable` —
//! таблица экипировки спавнера (`loot_table` + `slot_drop_chances`).
//!
//! Адаптация: `ResourceKey<LootTable>` хранится как `Identifier`
//! (LootTable-тип ещё не портирован; `LevelReader::generate_loot`
//! принимает `&Identifier`).

use std::collections::HashMap;

use crate::entity::equipment_slot::EquipmentSlot;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;

/// `EquipmentTable` — record `(lootTable, slotDropChances)`.
#[derive(Clone, Debug)]
pub struct EquipmentTable {
    /// `lootTable` — ResourceKey<LootTable> (как Identifier).
    pub loot_table: Identifier,
    /// `slotDropChances` — шанс дропа по слотам.
    pub slot_drop_chances: HashMap<EquipmentSlot, f32>,
}

impl EquipmentTable {
    /// `new(lootTable, slotDropChances)`.
    pub fn new(
        loot_table: Identifier,
        slot_drop_chances: HashMap<EquipmentSlot, f32>,
    ) -> Self {
        Self { loot_table, slot_drop_chances }
    }

    /// `new(lootTable, dropChance)` — одинаковый шанс для всех слотов.
    pub fn with_uniform_chance(loot_table: Identifier, drop_chance: f32) -> Self {
        Self {
            loot_table,
            slot_drop_chances: Self::create_for_all_slots(drop_chance),
        }
    }

    /// `createForAllSlots(dropChance)` — все `EquipmentSlot.VALUES`.
    pub fn create_for_all_slots(drop_chance: f32) -> HashMap<EquipmentSlot, f32> {
        EquipmentSlot::VALUES
            .iter()
            .map(|&s| (s, drop_chance))
            .collect()
    }

    /// `createForAllSlots(slots, dropChance)`.
    pub fn create_for_slots(slots: &[EquipmentSlot], drop_chance: f32) -> HashMap<EquipmentSlot, f32> {
        slots.iter().map(|&s| (s, drop_chance)).collect()
    }

    /// Сериализация в NBT (`loot_table` + `slot_drop_chances`).
    pub fn save(&self) -> crate::nbt::compound::CompoundTag {
        let mut tag = crate::nbt::compound::CompoundTag::new();
        tag.put(
            "loot_table",
            crate::nbt::tag::Tag::String(self.loot_table.to_string()),
        );
        let mut chances = crate::nbt::compound::CompoundTag::new();
        for (slot, chance) in &self.slot_drop_chances {
            chances.put(
                equipment_slot_name(*slot),
                crate::nbt::tag::Tag::Float(*chance),
            );
        }
        tag.put("slot_drop_chances", crate::nbt::tag::Tag::Compound(chances));
        tag
    }

    /// Десериализация из NBT.
    pub fn load(tag: &crate::nbt::compound::CompoundTag) -> Option<Self> {
        let loot_table = tag
            .get_string("loot_table")
            .and_then(|s| Identifier::try_parse(s))?;
        let mut slot_drop_chances = HashMap::new();
        if let Some(Tag::Compound(chances)) = tag.get("slot_drop_chances") {
            for (name, t) in chances.iter() {
                if let Some(f) = t.as_float() {
                    if let Some(slot) = equipment_slot_from_name(name) {
                        slot_drop_chances.insert(slot, f);
                    }
                }
            }
        }
        Some(Self { loot_table, slot_drop_chances })
    }
}

/// Имя слота в NBT (snake_case, как `EquipmentSlot.CODEC`).
fn equipment_slot_name(slot: EquipmentSlot) -> &'static str {
    match slot {
        EquipmentSlot::Mainhand => "mainhand",
        EquipmentSlot::Offhand => "offhand",
        EquipmentSlot::Feet => "feet",
        EquipmentSlot::Legs => "legs",
        EquipmentSlot::Chest => "chest",
        EquipmentSlot::Head => "head",
        EquipmentSlot::Body => "body",
        EquipmentSlot::Saddle => "saddle",
    }
}

fn equipment_slot_from_name(name: &str) -> Option<EquipmentSlot> {
    Some(match name {
        "mainhand" => EquipmentSlot::Mainhand,
        "offhand" => EquipmentSlot::Offhand,
        "feet" => EquipmentSlot::Feet,
        "legs" => EquipmentSlot::Legs,
        "chest" => EquipmentSlot::Chest,
        "head" => EquipmentSlot::Head,
        "body" => EquipmentSlot::Body,
        "saddle" => EquipmentSlot::Saddle,
        _ => return None,
    })
}
