//! Перенос `net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler`
//! (включая Leaf/Gale-патч `ignoreNullLegacyStructureData` и
//! Paper `MCDataConverter`).
//!
//! Обрабатывает legacy-данные структур (формат до 1.13):
//! - `markChunkDone` — снять индексы обработанных чанков;
//! - `applyFix` — для чанков dataVersion < 1493 с
//!   `hasLegacyStructureData` подставить structure starts/references
//!   из legacy `.dat` файлов.
//!
//! Отличия от Java:
//! - `DimensionDataStorage` → [`DimensionDataStorage`](crate::world::level::storage::dimension_data_storage::DimensionDataStorage)
//!   за `Arc` (может отсутствовать — тогда populateCaches пропускается).
//! - `DataFixer` → [`DataFixer`](crate::util::datafix::DataFixer).
//! - `MCDataConverter.convertTag(MCTypeRegistry.CHUNK, ...)` →
//!   [`LegacyChunkConverter`] (опциональный хук; без него конвертация
//!   пропускается, как `var8`-пустой путь в Java).
//! - `GaleGlobalConfiguration...ignoreNullLegacyStructureData` →
//!   поле `ignore_null_legacy_structure_data` конструктора.
//! - `indexMap` хранит `SharedSavedData` (shared с DimensionDataStorage
//!   через `computeIfAbsent`); свежесозданные индексы — те же
//!   `Arc<Mutex<Box<dyn SavedData>>>`, даункаст к
//!   [`StructureFeatureIndexSavedData`] через `as_any`.

use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::resources::resource_key::ResourceKey;
use crate::util::datafix::data_fix_types::DataFixTypes;
use crate::util::datafix::DataFixer;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::chunk::storage::legacy_tag_fixer::LegacyTagFixer;
use crate::world::level::dimension::{level_to_level_stem, END, NETHER, OVERWORLD};
use crate::world::level::levelgen::structure::structure_feature_index_saved_data::StructureFeatureIndexSavedData;
use crate::world::level::saveddata::saved_data::{SavedData, SharedSavedData};
use crate::world::level::storage::dimension_data_storage::DimensionDataStorage;
use crate::world::level::LevelKey;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// `LAST_MONOLYTH_STRUCTURE_DATA_VERSION`.
pub const LAST_MONOLITH_STRUCTURE_DATA_VERSION: i32 = 1493;

/// `CURRENT_TO_LEGACY_MAP` — текущее имя → legacy-ключ `.dat`.
fn current_to_legacy_map(key: &str) -> Option<&'static str> {
    Some(match key {
        "Village" => "Village",
        "Mineshaft" => "Mineshaft",
        "Mansion" => "Mansion",
        "Igloo" => "Temple",
        "Desert_Pyramid" => "Temple",
        "Jungle_Pyramid" => "Temple",
        "Swamp_Hut" => "Temple",
        "Stronghold" => "Stronghold",
        "Monument" => "Monument",
        "Fortress" => "Fortress",
        "EndCity" => "EndCity",
        _ => return None,
    })
}

/// `LEGACY_TO_CURRENT_MAP` — legacy `id` дочернего куска → текущее имя.
fn legacy_to_current_map(key: &str) -> Option<&'static str> {
    Some(match key {
        "Iglu" => "Igloo",
        "TeDP" => "Desert_Pyramid",
        "TeJP" => "Jungle_Pyramid",
        "TeSH" => "Swamp_Hut",
        _ => return None,
    })
}

/// `OLD_STRUCTURE_REGISTRY_KEYS`.
const OLD_STRUCTURE_REGISTRY_KEYS: &[&str] = &[
    "pillager_outpost",
    "mineshaft",
    "mansion",
    "jungle_pyramid",
    "desert_pyramid",
    "igloo",
    "ruined_portal",
    "shipwreck",
    "swamp_hut",
    "stronghold",
    "monument",
    "ocean_ruin",
    "fortress",
    "endcity",
    "buried_treasure",
    "village",
    "nether_fossil",
    "bastion_remnant",
];

/// `MCDataConverter` (Paper `ca.spottedleaf.dataconverter`) —
/// конвертация legacy-тегов чанков. Опциональный хук: без него
/// `applyFix` пропускает convertTag (в Java конвертер обязателен,
/// но тег при этом почти всегда уже новой версии).
pub trait LegacyChunkConverter: Send + Sync {
    /// `MCDataConverter.convertTag(MCTypeRegistry.CHUNK, tag, from, to)`.
    fn convert_chunk_tag(&self, tag: CompoundTag, from_version: i32, to_version: i32)
        -> CompoundTag;
}

/// Даункаст `SharedSavedData` → `StructureFeatureIndexSavedData`.
fn as_index(data: &SharedSavedData) -> std::sync::MutexGuard<'_, Box<dyn SavedData>> {
    data.lock().unwrap()
}

/// `LegacyStructureDataHandler`.
pub struct LegacyStructureDataHandler {
    has_legacy_data: bool,
    /// `dataMap` — structure name → packedChunkPos → feature NBT.
    data_map: HashMap<String, HashMap<i64, CompoundTag>>,
    /// `indexMap` — legacy key → shared index saved data.
    index_map: HashMap<String, SharedSavedData>,
    dimension_data_storage: Option<Arc<DimensionDataStorage>>,
    legacy_keys: Vec<String>,
    current_keys: Vec<String>,
    /// Хранится как в Java (поле не читается — datafix идёт через storage).
    #[allow(dead_code)]
    data_fixer: Arc<dyn DataFixer>,
    chunk_converter: Option<Arc<dyn LegacyChunkConverter>>,
    ignore_null_legacy_structure_data: bool,
    caches_initialized: bool,
}

impl LegacyStructureDataHandler {
    /// `new LegacyStructureDataHandler(storage, legacyKeys, currentKeys, dataFixer)`.
    /// `ignore_null_legacy_structure_data` — Gale-конфиг
    /// `misc.ignoreNullLegacyStructureData` (default false).
    pub fn new(
        dimension_data_storage: Option<Arc<DimensionDataStorage>>,
        legacy_keys: Vec<String>,
        current_keys: Vec<String>,
        data_fixer: Arc<dyn DataFixer>,
    ) -> Self {
        Self::with_options(
            dimension_data_storage,
            legacy_keys,
            current_keys,
            data_fixer,
            None,
            false,
        )
    }

    /// Полный конструктор с опциональным chunk-конвертером и
    /// Gale-флагом.
    pub fn with_options(
        dimension_data_storage: Option<Arc<DimensionDataStorage>>,
        legacy_keys: Vec<String>,
        current_keys: Vec<String>,
        data_fixer: Arc<dyn DataFixer>,
        chunk_converter: Option<Arc<dyn LegacyChunkConverter>>,
        ignore_null_legacy_structure_data: bool,
    ) -> Self {
        // В Java hasLegacyData вычисляется по dataMap до populateCaches —
        // она всегда пуста на этом этапе, поэтому всегда false.
        // Сохраняем то же поведение.
        let data_map: HashMap<String, HashMap<i64, CompoundTag>> = HashMap::new();
        let mut flag = false;
        for key in &current_keys {
            flag |= data_map.contains_key(key);
        }
        Self {
            has_legacy_data: flag,
            data_map,
            index_map: HashMap::new(),
            dimension_data_storage,
            legacy_keys,
            current_keys,
            data_fixer,
            chunk_converter,
            ignore_null_legacy_structure_data,
            caches_initialized: false,
        }
    }

    /// Вызов методов `StructureFeatureIndexSavedData` на shared-объекте.
    fn with_index<R>(
        data: &SharedSavedData,
        f: impl FnOnce(&mut StructureFeatureIndexSavedData) -> R,
    ) -> R {
        let mut guard = data.lock().unwrap();
        let index = guard
            .as_any_mut()
            .downcast_mut::<StructureFeatureIndexSavedData>()
            .expect("indexMap entry is not StructureFeatureIndexSavedData");
        f(index)
    }

    /// `markChunkDone(chunkPos)`.
    pub fn mark_chunk_done_index(&mut self, chunk_pos: ChunkPos) {
        let packed = chunk_pos.as_long();
        for key in self.legacy_keys.clone() {
            let unhandled = self
                .index_map
                .get(&key)
                .map(|data| Self::with_index(data, |i| i.has_unhandled_index(packed)))
                .unwrap_or(false);
            if unhandled {
                if let Some(data) = self.index_map.get(&key) {
                    Self::with_index(data, |i| i.remove_index(packed));
                }
            }
        }
    }

    /// `targetDataVersion()`.
    pub fn target_data_version_index(&self) -> i32 {
        LAST_MONOLITH_STRUCTURE_DATA_VERSION
    }

    /// `applyFix(tag)` — главная точка входа `LegacyTagFixer`.
    pub fn apply_fix_index(&mut self, mut tag: CompoundTag) -> CompoundTag {
        if !self.caches_initialized {
            if let Some(storage) = self.dimension_data_storage.clone() {
                self.populate_caches(&storage);
            }
        }

        let data_version = crate::nbt::utils::get_data_version(&tag);
        if data_version < LAST_MONOLITH_STRUCTURE_DATA_VERSION {
            if let Some(converter) = &self.chunk_converter {
                tag = converter.convert_chunk_tag(
                    tag,
                    data_version,
                    LAST_MONOLITH_STRUCTURE_DATA_VERSION,
                );
            }
            // `tag.getCompound("Level").flatMap(getBoolean("hasLegacyStructureData")).orElse(false)`
            let has_legacy = tag
                .get_compound("Level")
                .and_then(|level| level.get_bool("hasLegacyStructureData"))
                .unwrap_or(false);
            if has_legacy {
                tag = self.update_from_legacy(tag);
            }
        }
        tag
    }

    /// `updateFromLegacy(tag)`.
    fn update_from_legacy(&mut self, mut tag: CompoundTag) -> CompoundTag {
        let level = tag.get_compound_or_empty("Level");
        let chunk_pos = ChunkPos::new(
            level.get_int_or("xPos", 0),
            level.get_int_or("zPos", 0),
        );
        if self.is_unhandled_structure_start(chunk_pos.x, chunk_pos.z) {
            tag = self.update_structure_start(tag, chunk_pos);
        }

        // `compoundOrEmpty1` = Level.Structures, `compoundOrEmpty2` = .References
        // (Java читает их из живого `compoundOrEmpty` — у нас re-fetch
        // после возможного update_structure_start).
        let level = tag.get_compound_or_empty("Level");
        let mut structures = level
            .get_compound("Structures")
            .cloned()
            .unwrap_or_default();
        let mut references = structures
            .get_compound("References")
            .cloned()
            .unwrap_or_default();

        for key in self.current_keys.clone() {
            let is_old_key = OLD_STRUCTURE_REGISTRY_KEYS
                .iter()
                .any(|k| k.eq_ignore_ascii_case(&key));
            if references.get_long_array(&key).is_none() && is_old_key {
                let mut list = Vec::new();
                for cx in (chunk_pos.x - 8)..=(chunk_pos.x + 8) {
                    for cz in (chunk_pos.z - 8)..=(chunk_pos.z + 8) {
                        if self.has_legacy_start(cx, cz, &key) {
                            list.push(ChunkPos::as_long_static(cx, cz));
                        }
                    }
                }
                references.put_long_array(&key, &list);
            }
        }

        structures.put("References", Tag::Compound(references));
        let mut new_level = tag.get_compound_or_empty("Level");
        new_level.put("Structures", Tag::Compound(structures));
        tag.put("Level", Tag::Compound(new_level));
        tag
    }

    /// `hasLegacyStart(chunkX, chunkZ, key)`.
    fn has_legacy_start(&self, chunk_x: i32, chunk_z: i32, key: &str) -> bool {
        if !self.has_legacy_data {
            return false;
        }
        if self.data_map.get(key).is_none() {
            return false;
        }
        let Some(legacy_key) = current_to_legacy_map(key) else {
            return false;
        };
        self.index_map
            .get(legacy_key)
            .map(|data| Self::with_index(data, |i| i.has_start_index(ChunkPos::as_long_static(chunk_x, chunk_z))))
            .unwrap_or(false)
    }

    /// `isUnhandledStructureStart(chunkX, chunkZ)`.
    fn is_unhandled_structure_start(&self, chunk_x: i32, chunk_z: i32) -> bool {
        if !self.has_legacy_data {
            return false;
        }
        for key in &self.current_keys {
            if self.data_map.contains_key(key) {
                let Some(legacy_key) = current_to_legacy_map(key) else {
                    continue;
                };
                let unhandled = self
                    .index_map
                    .get(legacy_key)
                    .map(|data| {
                        Self::with_index(data, |i| {
                            i.has_unhandled_index(ChunkPos::as_long_static(chunk_x, chunk_z))
                        })
                    })
                    .unwrap_or(false);
                if unhandled {
                    return true;
                }
            }
        }
        false
    }

    /// `updateStructureStart(tag, chunkPos)`.
    fn update_structure_start(&mut self, mut tag: CompoundTag, chunk_pos: ChunkPos) -> CompoundTag {
        let level = tag.get_compound_or_empty("Level");
        let mut structures = level
            .get_compound("Structures")
            .cloned()
            .unwrap_or_default();
        let mut starts = structures
            .get_compound("Starts")
            .cloned()
            .unwrap_or_default();

        let packed = chunk_pos.as_long();
        for key in self.current_keys.clone() {
            if let Some(map) = self.data_map.get(&key) {
                let Some(legacy_key) = current_to_legacy_map(&key) else {
                    continue;
                };
                let unhandled = self
                    .index_map
                    .get(legacy_key)
                    .map(|data| {
                        Self::with_index(data, |i| i.has_unhandled_index(packed))
                    })
                    .unwrap_or(false);
                if unhandled {
                    if let Some(feature) = map.get(&packed) {
                        starts.put(&key, Tag::Compound(feature.clone()));
                    }
                }
            }
        }

        structures.put("Starts", Tag::Compound(starts));
        let mut new_level = tag.get_compound_or_empty("Level");
        new_level.put("Structures", Tag::Compound(structures));
        tag.put("Level", Tag::Compound(new_level));
        tag
    }

    /// `populateCaches(storage)` — `synchronized` в Java → `&mut self`.
    fn populate_caches(&mut self, storage: &Arc<DimensionDataStorage>) {
        if self.caches_initialized {
            return;
        }
        for key in self.legacy_keys.clone() {
            #[allow(unused_assignments)]
            let mut compound_tag = CompoundTag::new();
            // Gale `ignoreNullLegacyStructureData`: tag == null → continue.
            match storage.read_tag_from_disk(
                &key,
                DataFixTypes::SavedDataStructureFeatureIndices,
                LAST_MONOLITH_STRUCTURE_DATA_VERSION,
            ) {
                Ok(tag) => {
                    if self.ignore_null_legacy_structure_data && tag.is_empty() {
                        continue;
                    }
                    compound_tag = tag
                        .get_compound("data")
                        .map(|d| d.get_compound_or_empty("Features"))
                        .unwrap_or_default();
                    if compound_tag.is_empty() {
                        continue;
                    }
                }
                Err(_) => continue,
            }

            // `compoundTag.forEach` — наполнение dataMap.
            let entries: Vec<(String, CompoundTag)> = compound_tag
                .iter()
                .filter_map(|(k, v)| match v {
                    Tag::Compound(c) => Some((k.clone(), c.clone())),
                    _ => None,
                })
                .collect();
            for (_name, mut feature) in entries {
                let packed = ChunkPos::as_long_static(
                    feature.get_int_or("ChunkX", 0),
                    feature.get_int_or("ChunkZ", 0),
                );
                let children = feature.get_list_or_empty("Children").to_vec();
                if !children.is_empty() {
                    if let Some(Tag::Compound(first)) = children.first() {
                        if let Some(child_id) = first.get_string("id") {
                            if let Some(mapped) = legacy_to_current_map(child_id) {
                                feature.put_string("id", mapped);
                            }
                        }
                    }
                }
                if let Some(id) = feature.get_string("id") {
                    let id = id.to_string();
                    self.data_map
                        .entry(id)
                        .or_default()
                        .insert(packed, feature);
                }
            }

            // `<key>_index` saved data.
            let type_ = StructureFeatureIndexSavedData::saved_data_type(format!("{key}_index"));
            let saved = storage.compute_if_absent(&type_);
            let all_empty = Self::with_index(&saved, |i| i.get_all().is_empty());
            if all_empty {
                // Свежий индекс (не из storage) — заполняется из Features.
                let fresh: SharedSavedData = Arc::new(Mutex::new(Box::new(
                    StructureFeatureIndexSavedData::new(),
                ) as Box<dyn SavedData>));
                for (_name, feature) in compound_tag.iter() {
                    if let Tag::Compound(c) = feature {
                        let packed = ChunkPos::as_long_static(
                            c.get_int_or("ChunkX", 0),
                            c.get_int_or("ChunkZ", 0),
                        );
                        Self::with_index(&fresh, |i| i.add_index(packed));
                    }
                }
                self.index_map.insert(key.clone(), fresh);
            } else {
                self.index_map.insert(key.clone(), saved);
            }
        }
        self.caches_initialized = true;
    }
}

impl LegacyTagFixer for LegacyStructureDataHandler {
    fn apply_fix(&mut self, tag: CompoundTag) -> CompoundTag {
        self.apply_fix_index(tag)
    }

    fn mark_chunk_done(&mut self, chunk_pos: ChunkPos) {
        self.mark_chunk_done_index(chunk_pos);
    }

    fn target_data_version(&self) -> i32 {
        self.target_data_version_index()
    }
}

/// `getLegacyTagFixer(level, storage, dataFixer)` — фабрика
/// `Supplier<LegacyTagFixer>` → `Rc<dyn Fn() -> Rc<RefCell<dyn LegacyTagFixer>>>`.
/// Для неизвестных измерений — `EmptyLegacyTagFixer`.
///
/// `chunk_converter` — опциональный `MCDataConverter`-хук.
/// `ignore_null_legacy_structure_data` — Gale-флаг.
pub fn get_legacy_tag_fixer(
    level: &ResourceKey<LevelKey>,
    storage: Option<Arc<DimensionDataStorage>>,
    data_fixer: Arc<dyn DataFixer>,
    chunk_converter: Option<Arc<dyn LegacyChunkConverter>>,
    ignore_null_legacy_structure_data: bool,
) -> Rc<dyn Fn() -> Rc<std::cell::RefCell<dyn LegacyTagFixer>>> {
    let stem = level_to_level_stem(level);
    let (legacy, current): (Vec<&'static str>, Vec<&'static str>) =
        if stem == *OVERWORLD {
            (
                vec!["Monument", "Stronghold", "Village", "Mineshaft", "Temple", "Mansion"],
                vec![
                    "Village",
                    "Mineshaft",
                    "Mansion",
                    "Igloo",
                    "Desert_Pyramid",
                    "Jungle_Pyramid",
                    "Swamp_Hut",
                    "Stronghold",
                    "Monument",
                ],
            )
        } else if stem == *NETHER {
            (vec!["Fortress"], vec!["Fortress"])
        } else if stem == *END {
            (vec!["EndCity"], vec!["EndCity"])
        } else {
            return Rc::new(|| {
                Rc::new(std::cell::RefCell::new(
                    crate::world::level::chunk::storage::legacy_tag_fixer::EmptyLegacyTagFixer,
                )) as Rc<std::cell::RefCell<dyn LegacyTagFixer>>
            });
        };

    Rc::new(move || {
        let handler = LegacyStructureDataHandler::with_options(
            storage.clone(),
            legacy.iter().map(|s| s.to_string()).collect(),
            current.iter().map(|s| s.to_string()).collect(),
            data_fixer.clone(),
            chunk_converter.clone(),
            ignore_null_legacy_structure_data,
        );
        Rc::new(std::cell::RefCell::new(handler)) as Rc<std::cell::RefCell<dyn LegacyTagFixer>>
    })
}

