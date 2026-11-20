//! Перенос `net.minecraft.world.level.storage.DimensionDataStorage`.
//!
//! Хранит `SavedData` измерения в `<id>.dat` файлах: ленивая загрузка
//! через [`DataFixTypes`](crate::util::datafix::DataFixTypes)-фикс,
//! отложенное сохранение dirty-записей на отдельном потоке
//! (Java `CompletableFuture` + IO-пул → `std::thread`).
//!
//! Отличия от Java:
//! - `HolderLookup.Provider`/`RegistryOps` не нужны — кодеки работают
//!   напрямую с `CompoundTag`.
//! - `cache: Map<SavedDataType<?>, Optional<SavedData>>` →
//!   `HashMap<id, CacheEntry>`; equality `SavedDataType` — по id.
//! - Type-erased encode хранится в записи кэша (замыкание с
//!   downcast-шимом), т.к. `SavedDataCodec<T>` мономорфен.
//! - `Util.DIMENSION_DATA_IO_POOL` → один рабочий поток на storage;
//!   `CompletableFuture` → [`SaveFuture`].

use crate::nbt::accounter::NbtAccounter;
use crate::nbt::compound::CompoundTag;
use crate::nbt::io;
use crate::nbt::tag::Tag;
use crate::nbt::utils::{add_current_data_version, get_data_version_or};
use crate::util::datafix::data_fix_types::DataFixTypes;
use crate::util::datafix::DataFixer;
use crate::world::level::saveddata::saved_data::{SavedData, SharedSavedData};
use crate::world::level::saveddata::saved_data_type::SavedDataType;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Type-erased `codec.encode`: `&dyn SavedData` → `CompoundTag`.
type ErasedEncode = Box<dyn Fn(&dyn SavedData) -> CompoundTag + Send + Sync>;

/// Запись кэша: shared-объект + type-erased encode.
struct CacheEntry {
    data: Option<SharedSavedData>,
    encode: ErasedEncode,
}

/// `CompletableFuture<?>` из `scheduleSave` — join по завершении
/// цепочки отложенных записей.
#[derive(Clone)]
pub struct SaveFuture {
    pending: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl SaveFuture {
    fn completed() -> Self {
        Self {
            pending: Arc::new(Mutex::new(None)),
        }
    }

    /// `join()` — дождаться завершения записи.
    pub fn join(self) {
        if let Some(handle) = self.pending.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}

/// `DimensionDataStorage`.
pub struct DimensionDataStorage {
    /// `cache` — по `SavedDataType.id` (Java-equality по id).
    cache: Mutex<HashMap<String, CacheEntry>>,
    fixer: Arc<dyn DataFixer>,
    data_folder: PathBuf,
    /// `pendingWriteFuture` — цепочка записей.
    pending_write: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl DimensionDataStorage {
    /// `new DimensionDataStorage(dataFolder, fixerUpper, registries)`.
    /// `registries` не нужен — кодеки NBT-нативные.
    pub fn new(data_folder: impl Into<PathBuf>, fixer: Arc<dyn DataFixer>) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            fixer,
            data_folder: data_folder.into(),
            pending_write: Arc::new(Mutex::new(None)),
        }
    }

    /// `getDataFile(filename)` — `<id>.dat`.
    fn data_file(&self, filename: &str) -> PathBuf {
        self.data_folder.join(format!("{filename}.dat"))
    }

    /// `computeIfAbsent(type)` — загрузить или создать + `set`.
    pub fn compute_if_absent<T: SavedData + 'static>(
        &self,
        type_: &SavedDataType<T>,
    ) -> SharedSavedData {
        if let Some(data) = self.get(type_) {
            return data;
        }
        let value = (type_.constructor)();
        let shared: SharedSavedData =
            Arc::new(Mutex::new(Box::new(value) as Box<dyn SavedData>));
        self.set_shared(type_, shared.clone());
        shared
    }

    /// `get(type)` — lazy `readSavedData` + кэш.
    pub fn get<T: SavedData + 'static>(
        &self,
        type_: &SavedDataType<T>,
    ) -> Option<SharedSavedData> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(entry) = cache.get(type_.id.as_str()) {
                return entry.data.clone();
            }
        }
        let loaded = self.read_saved_data(type_);
        self.cache.lock().unwrap().insert(
            type_.id.clone(),
            CacheEntry {
                data: loaded.clone(),
                encode: erased_encode::<T>(type_.codec.encode),
            },
        );
        loaded
    }

    /// `readSavedData(type)` — чтение `.dat` + codec.parse по `data`.
    fn read_saved_data<T: SavedData + 'static>(
        &self,
        type_: &SavedDataType<T>,
    ) -> Option<SharedSavedData> {
        let data_file = self.data_file(&type_.id);
        if !data_file.exists() {
            return None;
        }
        let tag = self
            .read_tag_from_disk(
                &type_.id,
                type_.data_fix_type,
                DataFixTypes::current_version(),
            )
            .ok()?;
        let data_tag = tag.get("data").and_then(Tag::as_compound)?;
        let value = (type_.codec.decode)(data_tag)?;
        Some(Arc::new(Mutex::new(Box::new(value) as Box<dyn SavedData>)))
    }

    /// `set(type, value)` — положить в кэш + dirty.
    pub fn set<T: SavedData + 'static>(&self, type_: &SavedDataType<T>, value: T) {
        let shared: SharedSavedData =
            Arc::new(Mutex::new(Box::new(value) as Box<dyn SavedData>));
        self.set_shared(type_, shared);
    }

    /// Внутренний `set` для уже за-боксованного shared-объекта.
    fn set_shared<T: SavedData + 'static>(
        &self,
        type_: &SavedDataType<T>,
        value: SharedSavedData,
    ) {
        value.lock().unwrap().set_dirty();
        self.cache.lock().unwrap().insert(
            type_.id.clone(),
            CacheEntry {
                data: Some(value),
                encode: erased_encode::<T>(type_.codec.encode),
            },
        );
    }

    /// `readTagFromDisk(filename, dataFixType, version)` — чтение
    /// `.dat` (gzip или raw NBT) + datafix до `version`.
    pub fn read_tag_from_disk(
        &self,
        filename: &str,
        data_fix_type: DataFixTypes,
        version: i32,
    ) -> std::io::Result<CompoundTag> {
        let bytes = std::fs::read(self.data_file(filename))?;
        let compressed = is_gzip(&bytes);
        let mut tag = if compressed {
            io::read_compressed(&bytes[..], &mut NbtAccounter::unlimited_heap())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?
        } else {
            io::read_bytes(&bytes, &mut NbtAccounter::unlimited_heap())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?
        };
        let data_version = get_data_version_or(&tag, 1343);
        tag = data_fix_type.update(self.fixer.as_ref(), tag, data_version, version);
        Ok(tag)
    }

    /// `scheduleSave()` — собрать dirty-теги и записать асинхронно,
    /// цепляясь к `pendingWriteFuture`.
    pub fn schedule_save(&self) -> SaveFuture {
        let entries = self.collect_dirty_tags_to_save();
        if entries.is_empty() {
            return SaveFuture::completed();
        }
        let data_folder = self.data_folder.clone();
        let prev = self.pending_write.lock().unwrap().take();
        let handle = std::thread::spawn(move || {
            if let Some(prev) = prev {
                let _ = prev.join();
            }
            for (id, tag) in entries {
                try_write(&data_folder, &id, &tag);
            }
        });
        *self.pending_write.lock().unwrap() = Some(handle);
        SaveFuture {
            pending: Arc::clone(&self.pending_write),
        }
    }

    /// `collectDirtyTagsToSave()` — `{id: {data: ..., DataVersion: ...}}`
    /// для всех dirty записей; dirty-флаги сбрасываются.
    fn collect_dirty_tags_to_save(&self) -> Vec<(String, CompoundTag)> {
        let mut result = Vec::new();
        let cache = self.cache.lock().unwrap();
        for (id, entry) in cache.iter() {
            let Some(data) = &entry.data else { continue };
            let mut guard = data.lock().unwrap();
            if guard.is_dirty() {
                let encoded = encode_entry(&entry.encode, &**guard);
                guard.set_dirty_flag(false);
                result.push((id.clone(), encoded));
            }
        }
        result
    }

    /// `saveAndJoin()` — `scheduleSave().join()`.
    pub fn save_and_join(&self) {
        self.schedule_save().join();
    }

    /// `close()` — `saveAndJoin()`.
    pub fn close(&self) {
        self.save_and_join();
    }
}

/// `isGzip` — проверка магических байт `1f 8b`.
fn is_gzip(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b
}

/// Type-erased `codec.encode`: downcast `dyn SavedData` → `T` → encode.
fn erased_encode<T: SavedData + 'static>(
    codec_encode: fn(&T) -> CompoundTag,
) -> ErasedEncode {
    Box::new(move |data: &dyn SavedData| {
        let typed = data
            .as_any()
            .downcast_ref::<T>()
            .expect("SavedData type mismatch in DimensionDataStorage cache");
        codec_encode(typed)
    })
}

/// `encodeUnchecked` — `{data: codec.encode(data), DataVersion: current}`.
fn encode_entry(encode: &ErasedEncode, data: &dyn SavedData) -> CompoundTag {
    let mut out = CompoundTag::new();
    out.put("data", Tag::Compound(encode(data)));
    add_current_data_version(&mut out);
    out
}

/// `tryWrite(type, tag)` — `NbtIo.writeCompressed` в `<id>.dat`.
fn try_write(data_folder: &Path, id: &str, tag: &CompoundTag) {
    let path = data_folder.join(format!("{id}.dat"));
    if let Ok(mut file) = std::fs::File::create(&path) {
        use std::io::Write;
        let _ = io::write_compressed(tag, &mut file);
    }
}
