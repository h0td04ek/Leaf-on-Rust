//! Перенос `net.minecraft.world.level.saveddata.SavedDataType`.
//!
//! Java — record `(id, constructor, codec, dataFixType)`. Codec в DFU
//! заменён парой NBT-функций [`SavedDataCodec`]: Java `Codec<T>`
//! кодирует в `Dynamic<NbtOps>` — наш эквивалент `CompoundTag`.
//! Равенство — по `id` (как в Java `equals`/`hashCode`).

use crate::nbt::compound::CompoundTag;
use crate::util::datafix::data_fix_types::DataFixTypes;
use crate::world::level::saveddata::saved_data::SavedData;
use std::fmt;
use std::hash::{Hash, Hasher};

/// `Codec<T>` из `SavedDataType` — NBT-кодек saved data.
pub struct SavedDataCodec<T> {
    /// `codec.encode` — `T` → содержимое тега `data`.
    pub encode: fn(&T) -> CompoundTag,
    /// `codec.decode` — содержимое тега `data` → `T`.
    pub decode: fn(&CompoundTag) -> Option<T>,
    #[doc(hidden)]
    pub _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T> SavedDataCodec<T> {
    /// `new SavedDataCodec<>(encode, decode)`.
    pub fn new(encode: fn(&T) -> CompoundTag, decode: fn(&CompoundTag) -> Option<T>) -> Self {
        Self {
            encode,
            decode,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Clone for SavedDataCodec<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SavedDataCodec<T> {}

/// `SavedDataType<T>` — тип сохранённых данных: id + конструктор +
/// кодек + тип datafix.
#[derive(Clone)]
pub struct SavedDataType<T: SavedData + 'static> {
    /// `id()`.
    pub id: String,
    /// `constructor()`.
    pub constructor: fn() -> T,
    /// `codec()`.
    pub codec: SavedDataCodec<T>,
    /// `dataFixType()`.
    pub data_fix_type: DataFixTypes,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T: SavedData + 'static> SavedDataType<T> {
    /// `new SavedDataType<>(id, constructor, codec, dataFixType)`.
    pub fn new(
        id: impl Into<String>,
        constructor: fn() -> T,
        codec: SavedDataCodec<T>,
        data_fix_type: DataFixTypes,
    ) -> Self {
        Self {
            id: id.into(),
            constructor,
            codec,
            data_fix_type,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: SavedData + 'static> PartialEq for SavedDataType<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T: SavedData + 'static> Eq for SavedDataType<T> {}

impl<T: SavedData + 'static> Hash for SavedDataType<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: SavedData + 'static> fmt::Display for SavedDataType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SavedDataType[{}]", self.id)
    }
}
