//! `EntityDataAccessor<T>` / `EntityDataSerializer<T>` — типы entity data.

use std::sync::Arc;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `EntityDataSerializer<T>` — сериализатор значения entity data.
/// Type-erased через `Arc<dyn EntityDataSerializer>`.
pub trait EntityDataSerializer: Send + Sync {
    /// `encode(buf, value)` — сериализация `Box<dyn Any>` значения.
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &dyn std::any::Any);

    /// `decode(buf)` — десериализация в `Box<dyn Any>`.
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Box<dyn std::any::Any + Send + Sync>;

    /// `copy(value)` — копия значения.
    fn copy_value(&self, value: &dyn std::any::Any) -> Box<dyn std::any::Any + Send + Sync>;

    /// `serialized_id` — id в `EntityDataSerializers`.
    fn serialized_id(&self) -> i32;
}

/// `EntityDataSerializer.forValueType(codec)` — value-type serializer.
/// `T` должен быть `Clone + Send + Sync + 'static`.
pub struct ForValueTypeSerializer<T: Clone + Send + Sync + 'static> {
    codec: Box<dyn StreamCodec<FriendlyByteBuf, T> + Send + Sync>,
    serialized_id: i32,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + Send + Sync + 'static> ForValueTypeSerializer<T> {
    /// `new(codec, serialized_id)`.
    pub fn new(
        codec: impl StreamCodec<FriendlyByteBuf, T> + Send + Sync + 'static,
        serialized_id: i32,
    ) -> Arc<Self> {
        Arc::new(Self {
            codec: Box::new(codec),
            serialized_id,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<T: Clone + Send + Sync + 'static> EntityDataSerializer for ForValueTypeSerializer<T> {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &dyn std::any::Any) {
        if let Some(v) = value.downcast_ref::<T>() {
            self.codec.encode(buf, v);
        }
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Box<dyn std::any::Any + Send + Sync> {
        Box::new(self.codec.decode(buf))
    }
    fn copy_value(&self, value: &dyn std::any::Any) -> Box<dyn std::any::Any + Send + Sync> {
        if let Some(v) = value.downcast_ref::<T>() {
            Box::new(v.clone())
        } else {
            panic!("Type mismatch in copy_value")
        }
    }
    fn serialized_id(&self) -> i32 { self.serialized_id }
}

/// `EntityDataAccessor<T>` — `{ id, serializer }`.
#[derive(Clone)]
pub struct EntityDataAccessor {
    /// `id`.
    pub id: i32,
    /// `serializer`.
    pub serializer: Arc<dyn EntityDataSerializer>,
}

impl EntityDataAccessor {
    /// `new(id, serializer)`.
    pub fn new(id: i32, serializer: Arc<dyn EntityDataSerializer>) -> Self {
        Self { id, serializer }
    }

    /// `create_accessor(id)` на serializer.
    pub fn create(id: i32, serializer: &Arc<dyn EntityDataSerializer>) -> Self {
        Self { id, serializer: serializer.clone() }
    }
}

impl PartialEq for EntityDataAccessor {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for EntityDataAccessor {}
impl std::hash::Hash for EntityDataAccessor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
impl std::fmt::Debug for EntityDataAccessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<entity data: {}>", self.id)
    }
}
