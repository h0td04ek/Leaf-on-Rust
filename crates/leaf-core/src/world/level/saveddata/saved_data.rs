//! Перенос `net.minecraft.world.level.saveddata.SavedData`.
//!
//! В Java — абстрактный класс с dirty-флагом. В Rust — трейт
//! [`SavedData`] с [`SavedDataBase`] для хранения флага.
//! Объекты разделяются между `DimensionDataStorage` и потребителями
//! через `Arc<Mutex<Box<dyn SavedData>>>` — [`SharedSavedData`].

use std::any::Any;
use std::sync::{Arc, Mutex};

/// Общее хранилище `SavedData` — `Rc`/`Arc`-обёртка над trait-объектом.
pub type SharedSavedData = Arc<Mutex<Box<dyn SavedData>>>;

/// `SavedData` — базовый контракт мировых сохранённых данных.
///
/// `as_any`/`as_any_mut` — аналог Java-генериков для даункаста
/// `Box<dyn SavedData>` к конкретному типу.
pub trait SavedData: Send {
    /// `isDirty()`.
    fn is_dirty(&self) -> bool;
    /// `setDirty()`.
    fn set_dirty(&mut self) {
        self.set_dirty_flag(true);
    }
    /// `setDirty(boolean)`.
    fn set_dirty_flag(&mut self, dirty: bool);
    /// Даункаст к конкретному типу.
    fn as_any(&self) -> &dyn Any;
    /// Мутабельный даункаст.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Базовый dirty-флаг — встраивается в реализации `SavedData`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SavedDataBase {
    dirty: bool,
}

impl SavedDataBase {
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self) {
        self.set_dirty_flag(true);
    }

    pub fn set_dirty_flag(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
}
