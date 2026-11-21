//! `SWMRNibbleArray` — Single Writer Multi Reader nibble-массив (4096 нибблов,
//! 2048 байт). Держит `updating`- и `visible`-снимки; читатели видят
//! `storage_visible`, писатель работает с `storage_updating` (copy-on-write).

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use crate::world::level::chunk::data_layer::DataLayer;

/// `SWMRNibbleArray[]` — разделяемый массив нибблов (в Java элементы
/// разделяются между чанком и кэшем движка по ссылке). `Mutex` вместо
/// `RefCell`, чтобы `ChunkAccess` оставался `Send + Sync`.
pub type NibbleArrayRef = Arc<Mutex<SWMRNibbleArray>>;

/// `boolean[]` emptiness map — разделяемый массив (chunk ↔ engine cache).
pub type EmptinessMapRef = Arc<Mutex<Vec<bool>>>;

pub const ARRAY_SIZE: usize = 16 * 16 * 16 / 2;

pub const INIT_STATE_NULL: u8 = 0;
pub const INIT_STATE_UNINIT: u8 = 1;
pub const INIT_STATE_INIT: u8 = 2;
pub const INIT_STATE_HIDDEN: u8 = 3;

type Storage = Box<[u8; ARRAY_SIZE]>;

fn allocate_bytes() -> Storage {
    Box::new([0u8; ARRAY_SIZE])
}

thread_local! {
    static SAVE_STATE_CACHE: RefCell<[Option<SaveState>; 4]> =
        RefCell::new([None, None, None, None]);
}

/// `SWMRNibbleArray.SaveState`.
#[derive(Clone)]
pub struct SaveState {
    pub data: Option<Box<[u8; ARRAY_SIZE]>>,
    pub state: u8,
}

pub struct SWMRNibbleArray {
    state_updating: u8,
    state_visible: u8,
    storage_updating: Option<Storage>,
    updating_dirty: bool,
    storage_visible: Option<Storage>,
    cached_is_all_zero: bool,
    cached_is_all_zero_valid: bool,
}

impl SWMRNibbleArray {
    /// `fromVanilla(DataLayer)`.
    pub fn from_vanilla(nibble: Option<&DataLayer>) -> Self {
        match nibble {
            None => Self::new_null(),
            Some(n) if n.is_empty() => Self::new(),
            Some(n) => {
                let mut bytes = Box::new([0u8; ARRAY_SIZE]);
                bytes.copy_from_slice(n.get_data());
                Self::with_bytes(Some(bytes))
            }
        }
    }

    /// `new()` — lazy init (uninitialised).
    pub fn new() -> Self {
        Self::with_bytes_and_state(None, INIT_STATE_UNINIT)
    }

    /// `new(null, isNullNibble=true)` — null nibble.
    pub fn new_null() -> Self {
        Self::with_bytes_and_state(None, INIT_STATE_NULL)
    }

    /// `new(bytes)`.
    pub fn with_bytes(bytes: Option<Box<[u8; ARRAY_SIZE]>>) -> Self {
        let state = if bytes.is_some() { INIT_STATE_INIT } else { INIT_STATE_UNINIT };
        Self::with_bytes_and_state(bytes, state)
    }

    /// `new(bytes, state)`.
    pub fn with_bytes_and_state(bytes: Option<Box<[u8; ARRAY_SIZE]>>, state: u8) -> Self {
        Self {
            state_updating: state,
            state_visible: state,
            storage_visible: bytes.clone(),
            storage_updating: bytes,
            updating_dirty: false,
            cached_is_all_zero: false,
            cached_is_all_zero_valid: false,
        }
    }

    /// `getSaveState()`.
    pub fn get_save_state(&self) -> Option<SaveState> {
        let state = self.state_visible;
        if state == INIT_STATE_NULL {
            return None;
        }
        if state == INIT_STATE_UNINIT {
            return SAVE_STATE_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                Some(
                    cache[INIT_STATE_UNINIT as usize]
                        .get_or_insert_with(|| SaveState {
                            data: None,
                            state: INIT_STATE_UNINIT,
                        })
                        .clone(),
                )
            });
        }

        let data = self.storage_visible.as_ref().expect("initialised nibble");
        let zero = if self.cached_is_all_zero_valid {
            self.cached_is_all_zero
        } else {
            is_all_zero(data)
        };
        if zero {
            if state == INIT_STATE_INIT {
                return SAVE_STATE_CACHE.with(|cache| {
                    let mut cache = cache.borrow_mut();
                    Some(
                        cache[INIT_STATE_UNINIT as usize]
                            .get_or_insert_with(|| SaveState {
                                data: None,
                                state: INIT_STATE_UNINIT,
                            })
                            .clone(),
                    )
                });
            }
            return None;
        }
        let mut cloned = Box::new([0u8; ARRAY_SIZE]);
        cloned.copy_from_slice(&data[..]);
        Some(SaveState { data: Some(cloned), state })
    }

    /// `extrudeLower(other)` — копирует данные `other` в каждую Y-секцию
    /// (16 строк по `end - start + 1` байт из начала массива).
    pub fn extrude_lower(&mut self, other: &SWMRNibbleArray) {
        assert_ne!(other.state_updating, INIT_STATE_NULL);

        if other.storage_updating.is_none() {
            self.set_uninitialised();
            return;
        }
        let src = other.storage_updating.as_ref().unwrap();

        if !self.updating_dirty {
            match self.storage_updating {
                Some(_) => self.storage_updating = Some(allocate_bytes()),
                None => {
                    self.storage_updating = Some(allocate_bytes());
                    self.state_updating = INIT_STATE_INIT;
                }
            }
            self.updating_dirty = true;
        }
        let into = self.storage_updating.as_mut().unwrap();

        let start = 0usize;
        let end = (15 | (15 << 4)) >> 1;
        for y in 0..=15usize {
            into[(y << (8 - 1))..(y << (8 - 1)) + (end - start + 1)]
                .copy_from_slice(&src[start..=end]);
        }
    }

    /// `setFull()`.
    pub fn set_full(&mut self) {
        if self.state_updating != INIT_STATE_HIDDEN {
            self.state_updating = INIT_STATE_INIT;
        }
        let storage = match &mut self.storage_updating {
            Some(s) if self.updating_dirty => s,
            _ => {
                self.storage_updating = Some(allocate_bytes());
                self.storage_updating.as_mut().unwrap()
            }
        };
        storage.fill(0xFF);
        self.updating_dirty = true;
    }

    /// `setZero()`.
    pub fn set_zero(&mut self) {
        if self.state_updating != INIT_STATE_HIDDEN {
            self.state_updating = INIT_STATE_INIT;
        }
        let storage = match &mut self.storage_updating {
            Some(s) if self.updating_dirty => s,
            _ => {
                self.storage_updating = Some(allocate_bytes());
                self.storage_updating.as_mut().unwrap()
            }
        };
        storage.fill(0);
        self.updating_dirty = true;
    }

    /// `setNonNull()`.
    pub fn set_non_null(&mut self) {
        if self.state_updating == INIT_STATE_HIDDEN {
            self.state_updating = INIT_STATE_INIT;
            return;
        }
        if self.state_updating != INIT_STATE_NULL {
            return;
        }
        self.state_updating = INIT_STATE_UNINIT;
    }

    /// `setNull()`.
    pub fn set_null(&mut self) {
        self.state_updating = INIT_STATE_NULL;
        self.storage_updating = None;
        self.updating_dirty = false;
    }

    /// `setUninitialised()`.
    pub fn set_uninitialised(&mut self) {
        self.state_updating = INIT_STATE_UNINIT;
        self.storage_updating = None;
        self.updating_dirty = false;
    }

    /// `setHidden()`.
    pub fn set_hidden(&mut self) {
        if self.state_updating == INIT_STATE_HIDDEN {
            return;
        }
        if self.state_updating != INIT_STATE_INIT {
            self.set_null();
        } else {
            self.state_updating = INIT_STATE_HIDDEN;
        }
    }

    /// `isDirty()` (updating).
    pub fn is_dirty(&self) -> bool {
        self.state_updating != self.state_visible || self.updating_dirty
    }

    pub fn is_null_nibble_updating(&self) -> bool {
        self.state_updating == INIT_STATE_NULL
    }
    pub fn is_null_nibble_visible(&self) -> bool {
        self.state_visible == INIT_STATE_NULL
    }
    pub fn is_uninitialised_updating(&self) -> bool {
        self.state_updating == INIT_STATE_UNINIT
    }
    pub fn is_uninitialised_visible(&self) -> bool {
        self.state_visible == INIT_STATE_UNINIT
    }
    pub fn is_initialised_updating(&self) -> bool {
        self.state_updating == INIT_STATE_INIT
    }
    pub fn is_initialised_visible(&self) -> bool {
        self.state_visible == INIT_STATE_INIT
    }
    pub fn is_hidden_updating(&self) -> bool {
        self.state_updating == INIT_STATE_HIDDEN
    }
    pub fn is_hidden_visible(&self) -> bool {
        self.state_visible == INIT_STATE_HIDDEN
    }

    fn swap_updating_and_mark_dirty(&mut self) {
        if self.updating_dirty {
            return;
        }
        self.storage_updating = Some(match self.storage_updating.take() {
            Some(s) => {
                let mut copy = allocate_bytes();
                copy.copy_from_slice(&s[..]);
                copy
            }
            None => {
                let mut s = allocate_bytes();
                s.fill(0);
                s
            }
        });
        if self.state_updating != INIT_STATE_HIDDEN {
            self.state_updating = INIT_STATE_INIT;
        }
        self.updating_dirty = true;
    }

    /// `updateVisible()` — переносит updating-снимок в visible. Возвращает true,
    /// если данные изменились (нужно уведомить слушателей).
    pub fn update_visible(&mut self) -> bool {
        if !self.is_dirty() {
            return false;
        }
        match self.state_updating {
            INIT_STATE_NULL | INIT_STATE_UNINIT => {
                self.storage_visible = None;
            }
            _ => {
                self.storage_visible = self.storage_updating.take();
            }
        }
        self.storage_updating = self.storage_visible.clone();
        self.updating_dirty = false;
        self.state_visible = self.state_updating;
        self.cached_is_all_zero_valid = false;
        true
    }

    /// `toVanillaNibble()`.
    pub fn to_vanilla_nibble(&self) -> Option<DataLayer> {
        match self.state_visible {
            INIT_STATE_HIDDEN | INIT_STATE_NULL => None,
            INIT_STATE_UNINIT => Some(DataLayer::new()),
            INIT_STATE_INIT => {
                let data = self.storage_visible.as_ref().expect("initialised nibble");
                let mut layer = DataLayer::new();
                layer.get_data_mut().copy_from_slice(&data[..]);
                Some(layer)
            }
            _ => unreachable!(),
        }
    }

    /// `getUpdating(index)` — index = `x | (z << 4) | (y << 8)`.
    pub fn get_updating(&self, index: usize) -> u8 {
        let bytes = match &self.storage_updating {
            Some(b) => b,
            None => return 0,
        };
        let value = bytes[index >> 1];
        (value >> ((index & 1) << 2)) & 0xF
    }

    /// `getUpdating(x, y, z)`.
    pub fn get_updating_xyz(&self, x: i32, y: i32, z: i32) -> u8 {
        self.get_updating((x & 15) as usize | (((z & 15) as usize) << 4) | (((y & 15) as usize) << 8))
    }

    /// `getVisible(index)`.
    pub fn get_visible(&self, index: usize) -> u8 {
        let bytes = match &self.storage_visible {
            Some(b) => b,
            None => return 0,
        };
        let value = bytes[index >> 1];
        (value >> ((index & 1) << 2)) & 0xF
    }

    /// `getVisible(x, y, z)`.
    pub fn get_visible_xyz(&self, x: i32, y: i32, z: i32) -> u8 {
        self.get_visible((x & 15) as usize | (((z & 15) as usize) << 4) | (((y & 15) as usize) << 8))
    }

    /// `set(index, value)` — updating.
    pub fn set(&mut self, index: usize, value: u8) {
        if !self.updating_dirty {
            self.swap_updating_and_mark_dirty();
        }
        let shift = ((index & 1) << 2) as u8;
        let i = index >> 1;
        let storage = self.storage_updating.as_mut().unwrap();
        let old_value = storage[i];
        let new_value = (old_value & (0xF0u8 >> shift)) | ((value & 0xF) << shift);
        if old_value != new_value {
            storage[i] = new_value;
            self.cached_is_all_zero_valid = false;
        }
    }

    /// `set(x, y, z, value)` — updating.
    pub fn set_xyz(&mut self, x: i32, y: i32, z: i32, value: u8) {
        self.set(
            (x & 15) as usize | (((z & 15) as usize) << 4) | (((y & 15) as usize) << 8),
            value,
        );
    }

    pub fn state_updating(&self) -> u8 {
        self.state_updating
    }
    pub fn state_visible(&self) -> u8 {
        self.state_visible
    }
}

impl Default for SWMRNibbleArray {
    fn default() -> Self {
        Self::new()
    }
}

fn is_all_zero(data: &[u8; ARRAY_SIZE]) -> bool {
    data.chunks_exact(8).all(|c| u64::from_ne_bytes(c.try_into().unwrap()) == 0)
}

/// `StarLightEngine.getFilledEmptyLight(totalLightSections)` — массив
/// null-nibble'ов (`new SWMRNibbleArray(null, true)`).
pub fn get_filled_empty_light_vec(total_light_sections: usize) -> Vec<Option<NibbleArrayRef>> {
    (0..total_light_sections)
        .map(|_| Some(Arc::new(Mutex::new(SWMRNibbleArray::new_null()))))
        .collect()
}
