//! Перенос `net.minecraft.world.level.SpawnData` — данные спавна
//! моб-спавнера (`entityToSpawn` NBT + `customSpawnRules` +
//! `equipment`).
//!
//! Codec-слой Java (`CODEC`/`LIST_CODEC` через Mojang serialization)
//! заменён NBT-сериализацией `save`/`load` — та же структура тегов,
//! что читает/пишет `ValueInput`/`ValueOutput` в Java.

use crate::core::block_pos::BlockPos;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::util::inclusive_range::InclusiveRange;
use crate::util::random::weighted_list::WeightedList;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::light_layer::LightLayer;

/// `SpawnData.CustomSpawnRules` — границы освещения для спавна.
#[derive(Clone, Debug)]
pub struct CustomSpawnRules {
    /// `blockLightLimit`.
    pub block_light_limit: InclusiveRange<i32>,
    /// `skyLightLimit`.
    pub sky_light_limit: InclusiveRange<i32>,
}

impl CustomSpawnRules {
    /// `LIGHT_RANGE` — [0, 15].
    pub fn light_range() -> InclusiveRange<i32> {
        InclusiveRange::new(0, 15).unwrap()
    }

    /// `new(blockLightLimit, skyLightLimit)`.
    pub fn new(
        block_light_limit: InclusiveRange<i32>,
        sky_light_limit: InclusiveRange<i32>,
    ) -> Self {
        Self { block_light_limit, sky_light_limit }
    }

    /// `isValidPosition(pos, level)` — обе яркости в пределах.
    pub fn is_valid_position(&self, pos: BlockPos, level: &dyn LevelReader) -> bool {
        self.block_light_limit
            .is_value_in_range(&level.get_brightness(LightLayer::Block, pos))
            && self.sky_light_limit
                .is_value_in_range(&level.get_brightness(LightLayer::Sky, pos))
    }

    /// NBT: `{"block_light_limit": {min,max}, "sky_light_limit": {...}}`
    /// (int-значение = min == max).
    pub fn save(&self) -> CompoundTag {
        let mut tag = CompoundTag::new();
        tag.put("block_light_limit", range_to_tag(&self.block_light_limit));
        tag.put("sky_light_limit", range_to_tag(&self.sky_light_limit));
        tag
    }

    pub fn load(tag: &CompoundTag) -> Option<Self> {
        let block = range_from_tag(tag.get("block_light_limit")?)?;
        let sky = range_from_tag(tag.get("sky_light_limit")?)?;
        Some(Self::new(block, sky))
    }
}

fn range_to_tag(range: &InclusiveRange<i32>) -> Tag {
    let mut t = CompoundTag::new();
    t.put("min", Tag::Int(range.min_inclusive));
    t.put("max", Tag::Int(range.max_inclusive));
    Tag::Compound(t)
}

fn range_from_tag(tag: &Tag) -> Option<InclusiveRange<i32>> {
    match tag {
        Tag::Int(v) => InclusiveRange::new(*v, *v).ok(),
        Tag::Compound(c) => {
            let min = c.get_int("min")?;
            let max = c.get_int("max")?;
            InclusiveRange::new(min, max).ok()
        }
        _ => None,
    }
}

/// `SpawnData` — record `(entityToSpawn, customSpawnRules, equipment)`.
#[derive(Clone, Debug)]
pub struct SpawnData {
    /// `entityToSpawn` — NBT сущности (с "id").
    pub entity_to_spawn: CompoundTag,
    /// `customSpawnRules`.
    pub custom_spawn_rules: Option<CustomSpawnRules>,
    /// `equipment` — `EquipmentTable` из `mob.equip`.
    pub equipment: Option<crate::entity::equipment_table::EquipmentTable>,
}

impl SpawnData {
    /// `ENTITY_TAG` — ключ "entity" в SpawnPotentials-записях.
    pub const ENTITY_TAG: &'static str = "entity";

    /// `new()` — пустой SpawnData.
    pub fn new() -> Self {
        Self::default()
    }

    /// `new(entityToSpawn, customSpawnRules, equipment)` —
    /// конструктор нормализует "id" (string → Identifier или remove).
    pub fn with_data(
        mut entity_to_spawn: CompoundTag,
        custom_spawn_rules: Option<CustomSpawnRules>,
        equipment: Option<crate::entity::equipment_table::EquipmentTable>,
    ) -> Self {
        // Java: `entityToSpawn.read("id", Identifier.CODEC)` →
        // `store("id", Identifier.CODEC, id)` либо `remove("id")`.
        match entity_to_spawn.get_string("id") {
            Some(s) => match crate::resources::identifier::Identifier::try_parse(s) {
                Some(id) => {
                    entity_to_spawn.put("id", Tag::String(id.to_string()));
                }
                None => {
                    entity_to_spawn.remove("id");
                }
            },
            None => {
                entity_to_spawn.remove("id");
            }
        }
        Self { entity_to_spawn, custom_spawn_rules, equipment }
    }

    /// `getEntityToSpawn()`.
    pub fn entity_to_spawn(&self) -> &CompoundTag {
        &self.entity_to_spawn
    }

    /// `getCustomSpawnRules()`.
    pub fn custom_spawn_rules(&self) -> Option<&CustomSpawnRules> {
        self.custom_spawn_rules.as_ref()
    }

    /// `getEquipment()`.
    pub fn equipment(&self) -> Option<&crate::entity::equipment_table::EquipmentTable> {
        self.equipment.as_ref()
    }

    /// `id`-поле тега (`entityToSpawn.id`), если валидный Identifier.
    pub fn entity_id(&self) -> Option<crate::resources::identifier::Identifier> {
        self.entity_to_spawn
            .get_string("id")
            .and_then(|s| crate::resources::identifier::Identifier::try_parse(s))
    }

    /// NBT-сериализация одной записи (`{"entity": ..., ...}`).
    pub fn save(&self) -> CompoundTag {
        let mut tag = CompoundTag::new();
        tag.put(SpawnData::ENTITY_TAG, Tag::Compound(self.entity_to_spawn.clone()));
        if let Some(rules) = &self.custom_spawn_rules {
            tag.put("custom_spawn_rules", Tag::Compound(rules.save()));
        }
        if let Some(eq) = &self.equipment {
            tag.put("equipment", Tag::Compound(eq.save()));
        }
        tag
    }

    /// Десериализация записи (`WeightedList` element).
    pub fn load(tag: &CompoundTag) -> Option<Self> {
        let entity = match tag.get(SpawnData::ENTITY_TAG) {
            Some(Tag::Compound(c)) => c.clone(),
            _ => CompoundTag::new(),
        };
        let custom_spawn_rules = tag
            .get("custom_spawn_rules")
            .and_then(|t| match t {
                Tag::Compound(c) => CustomSpawnRules::load(c),
                _ => None,
            });
        let equipment = tag
            .get("equipment")
            .and_then(|t| match t {
                Tag::Compound(c) => crate::entity::equipment_table::EquipmentTable::load(c),
                _ => None,
            });
        Some(Self::with_data(entity, custom_spawn_rules, equipment))
    }
}

impl Default for SpawnData {
    fn default() -> Self {
        Self::with_data(CompoundTag::new(), None, None)
    }
}

/// `SpawnData.LIST_CODEC` — `WeightedList<SpawnData>` в NBT
/// (`[{weight, entity, ...}, ...]`).
pub fn save_spawn_potentials(list: &WeightedList<SpawnData>) -> Vec<Tag> {
    list.unwrap()
        .iter()
        .map(|w| {
            let mut t = w.value.save();
            t.put("weight", Tag::Int(w.weight));
            Tag::Compound(t)
        })
        .collect()
}

/// `SpawnData.LIST_CODEC` — загрузка `SpawnPotentials`.
pub fn load_spawn_potentials(tags: &[Tag]) -> WeightedList<SpawnData> {
    let mut builder = WeightedList::builder();
    for t in tags {
        if let Tag::Compound(c) = t {
            if let Some(data) = SpawnData::load(c) {
                let weight = c.get_int("weight").unwrap_or(1);
                builder.add_with_weight(data, weight);
            }
        }
    }
    builder.build()
}
