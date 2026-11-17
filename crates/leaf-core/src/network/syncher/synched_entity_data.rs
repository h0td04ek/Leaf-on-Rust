//! `SynchedEntityData` — синхронизация данных entity между сервером и клиентом.

use std::sync::Arc;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::syncher::entity_data_accessor::{EntityDataAccessor, EntityDataSerializer};

/// `SynchedEntityData` — массив `DataItem` по id.
pub struct SynchedEntityData {
    /// `itemsById` — `DataItem` по id (0..=254).
    items_by_id: Vec<DataItem>,
    /// `isDirty`.
    is_dirty: bool,
}

/// `DataItem<T>` — одно синхронизируемое поле.
pub struct DataItem {
    /// `accessor`.
    pub accessor: EntityDataAccessor,
    /// `value` — type-erased.
    value: Box<dyn std::any::Any + Send + Sync>,
    /// `initialValue`.
    initial_value: Box<dyn std::any::Any + Send + Sync>,
    /// `dirty`.
    dirty: bool,
}

impl DataItem {
    /// `new(accessor, value)`.
    pub fn new(accessor: EntityDataAccessor, value: Box<dyn std::any::Any + Send + Sync>) -> Self {
        let initial = accessor.serializer.copy_value(value.as_ref());
        Self { accessor, value, initial_value: initial, dirty: false }
    }

    /// `getAccessor`.
    pub fn get_accessor(&self) -> &EntityDataAccessor {
        &self.accessor
    }

    /// `setValue`.
    pub fn set_value(&mut self, value: Box<dyn std::any::Any + Send + Sync>) {
        self.value = value;
    }

    /// `getValue`.
    pub fn get_value(&self) -> &dyn std::any::Any {
        self.value.as_ref()
    }

    /// `isDirty`.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// `setDirty`.
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    /// `isSetToDefault` — сравнение через сериализацию.
    pub fn is_set_to_default(&self) -> bool {
        let mut buf1 = FriendlyByteBuf::new();
        let mut buf2 = FriendlyByteBuf::new();
        self.accessor.serializer.encode(&mut buf1, self.value.as_ref());
        self.accessor.serializer.encode(&mut buf2, self.initial_value.as_ref());
        buf1.copy_bytes() == buf2.copy_bytes()
    }

    /// `value()` — `DataValue`.
    pub fn value(&self) -> DataValue {
        DataValue {
            id: self.accessor.id,
            serializer: self.accessor.serializer.serialized_id(),
            value: self.accessor.serializer.copy_value(self.value.as_ref()),
        }
    }
}

/// `DataValue<T>` — `{ id, serializer, value }` для wire.
pub struct DataValue {
    /// `id`.
    pub id: i32,
    /// `serializer` — serialized id.
    pub serializer: i32,
    /// `value` — type-erased.
    pub value: Box<dyn std::any::Any + Send + Sync>,
}

impl DataValue {
    /// `create(accessor, value)`.
    pub fn create(accessor: &EntityDataAccessor, value: Box<dyn std::any::Any + Send + Sync>) -> Self {
        Self {
            id: accessor.id,
            serializer: accessor.serializer.serialized_id(),
            value: accessor.serializer.copy_value(value.as_ref()),
        }
    }

    /// `write` — `byte id + VarInt serializer_id + value`.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_byte(self.id);
        buf.write_var_int(self.serializer);
        let serializer = crate::network::syncher::entity_data_serializers::get_serializer(self.serializer)
            .expect("Unknown serializer type");
        serializer.encode(buf, self.value.as_ref());
    }

    /// `read` — `byte id + VarInt serializer_id + value`.
    pub fn read(buf: &mut FriendlyByteBuf, id: i32) -> Self {
        let serializer_id = buf.read_var_int();
        let serializer = crate::network::syncher::entity_data_serializers::get_serializer(serializer_id)
            .expect("Unknown serializer type");
        let value = serializer.decode(buf);
        Self { id, serializer: serializer_id, value }
    }
}

impl SynchedEntityData {
    /// `new(entity, itemsById)` — внутренняя.
    pub fn new(items_by_id: Vec<DataItem>) -> Self {
        Self { items_by_id, is_dirty: false }
    }

    /// `defineId(clazz, serializer)` — `EntityDataAccessor`.
    /// `ClassTreeIdRegistry` — упрощённо: id = позиция в массиве.
    pub fn define_id(serializer: &Arc<dyn EntityDataSerializer>) -> EntityDataAccessor {
        EntityDataAccessor::create(0, serializer)
    }

    /// `getItem(key)`.
    pub fn get_item(&self, key: &EntityDataAccessor) -> Option<&DataItem> {
        self.items_by_id.get(key.id as usize)
    }

    /// `getItem` mutable.
    pub fn get_item_mut(&mut self, key: &EntityDataAccessor) -> Option<&mut DataItem> {
        self.items_by_id.get_mut(key.id as usize)
    }

    /// `get(key)`.
    pub fn get(&self, key: &EntityDataAccessor) -> Option<&dyn std::any::Any> {
        self.get_item(key).map(|i| i.get_value())
    }

    /// `set(key, value, force)`.
    pub fn set(&mut self, key: &EntityDataAccessor, value: Box<dyn std::any::Any + Send + Sync>, force: bool) {
        if let Some(item) = self.get_item_mut(key) {
            let mut buf1 = FriendlyByteBuf::new();
            let mut buf2 = FriendlyByteBuf::new();
            item.accessor.serializer.encode(&mut buf1, value.as_ref());
            item.accessor.serializer.encode(&mut buf2, item.value.as_ref());
            if force || buf1.copy_bytes() != buf2.copy_bytes() {
                item.set_value(value);
                item.set_dirty(true);
                self.is_dirty = true;
            }
        }
    }

    /// `isDirty`.
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// `packDirty` — список изменённых `DataValue`.
    pub fn pack_dirty(&mut self) -> Option<Vec<DataValue>> {
        if !self.is_dirty {
            return None;
        }
        self.is_dirty = false;
        let mut list = Vec::new();
        for item in &mut self.items_by_id {
            if item.is_dirty() {
                item.set_dirty(false);
                list.push(item.value());
            }
        }
        Some(list)
    }

    /// `getNonDefaultValues`.
    pub fn get_non_default_values(&self) -> Vec<DataValue> {
        self.items_by_id.iter()
            .filter(|i| !i.is_set_to_default())
            .map(|i| i.value())
            .collect()
    }

    /// `packAll` — Paper: все значения.
    pub fn pack_all(&self) -> Vec<DataValue> {
        self.items_by_id.iter().map(|i| i.value()).collect()
    }

}

/// `SynchedEntityData.Builder`.
pub struct Builder {
    items_by_id: Vec<Option<DataItem>>,
}

impl Builder {
        /// `new(entity)` — размер по `ID_REGISTRY.getCount`.
        pub fn new(size: usize) -> Self {
            Self { items_by_id: (0..size).map(|_| None).collect() }
        }

        /// `define(accessor, value)`.
        pub fn define(&mut self, accessor: EntityDataAccessor, value: Box<dyn std::any::Any + Send + Sync>) -> &mut Self {
            let id = accessor.id as usize;
            if id >= self.items_by_id.len() {
                panic!("Data value id is too big with {}! (Max is {})", id, self.items_by_id.len());
            }
            if self.items_by_id[id].is_some() {
                panic!("Duplicate id value for {}!", id);
            }
            if accessor.serializer.serialized_id() < 0 {
                panic!("Unregistered serializer for {}!", id);
            }
            self.items_by_id[id] = Some(DataItem::new(accessor, value));
            self
        }

        /// `build`.
        pub fn build(self) -> SynchedEntityData {
            for (i, item) in self.items_by_id.iter().enumerate() {
                if item.is_none() {
                    panic!("Entity has not defined synched data value {}", i);
                }
            }
            SynchedEntityData::new(self.items_by_id.into_iter().map(|i| i.unwrap()).collect())
        }
}

