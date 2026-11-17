//! Перенос `net.minecraft.util.debug.DebugSubscription` —
//! упрощённая версия: value codec хранится как type-erased объект.

use std::sync::Arc;

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `DebugSubscription<T>` — подписка на debug-данные.
/// `T` стёрт до `()` — конкретные типы добавляются по мере порта
/// debugchart-логгеров.
#[derive(Clone)]
pub struct DebugSubscription {
    /// `valueStreamCodec` — `None` для подписок без значения.
    pub value_stream_codec: Option<Arc<dyn StreamCodec<FriendlyByteBuf, ()> + Send + Sync>>,
    /// `expireAfterTicks` — `DOES_NOT_EXPIRE` = 0.
    pub expire_after_ticks: i32,
    /// `registryId` — id в `Registries.DEBUG_SUBSCRIPTION`.
    pub registry_id: i32,
}

/// `DebugSubscription.DOES_NOT_EXPIRE`.
pub const DOES_NOT_EXPIRE: i32 = 0;

static REGISTRY: std::sync::OnceLock<std::sync::RwLock<Vec<DebugSubscription>>> = std::sync::OnceLock::new();
fn registry() -> &'static std::sync::RwLock<Vec<DebugSubscription>> {
    REGISTRY.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}

impl DebugSubscription {
    /// `new DebugSubscription(codec, expireAfterTicks)`.
    pub fn new(
        value_stream_codec: Option<Arc<dyn StreamCodec<FriendlyByteBuf, ()> + Send + Sync>>,
        expire_after_ticks: i32,
    ) -> Self {
        Self { value_stream_codec, expire_after_ticks, registry_id: -1 }
    }

    /// `DebugSubscriptions.DEDICATED_SERVER_TICK_TIME`.
    pub fn tick_time() -> Self {
        Self::new(None, DOES_NOT_EXPIRE)
    }

    /// `expireAfterTicks()`.
    pub fn expire_after_ticks(&self) -> i32 {
        self.expire_after_ticks
    }

    /// `valueStreamCodec()`.
    pub fn value_stream_codec(&self) -> Option<Arc<dyn StreamCodec<FriendlyByteBuf, ()> + Send + Sync>> {
        self.value_stream_codec.clone()
    }
}

impl DebugSubscription {
    /// `STREAM_CODEC` — registry-based (id по порядку регистрации).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, DebugSubscription> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, sub: &DebugSubscription| {
                buf.write_var_int(sub.registry_id());
            },
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                DebugSubscription::by_id(id)
            },
        )
    }

    /// `registryId()`.
    pub fn registry_id(&self) -> i32 {
        self.registry_id
    }

    /// `byId` — поиск по registry id (реестр заполняется при bootstrap).
    pub fn by_id(id: i32) -> Self {
        registry().read().unwrap().get(id as usize).cloned().unwrap_or_else(|| Self::tick_time())
    }

    /// `register` — регистрация подписки, возвращает id.
    pub fn register(sub: Self) -> i32 {
        let mut reg = registry().write().unwrap();
        reg.push(sub.clone());
        (reg.len() - 1) as i32
    }
}

impl std::fmt::Debug for DebugSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugSubscription")
            .field("expire_after_ticks", &self.expire_after_ticks)
            .finish()
    }
}

/// `DebugSubscription.Update<T>` — `{ subscription, Optional<T> value }`.
#[derive(Debug)]
pub struct Update {
    /// `subscription`.
    pub subscription: DebugSubscription,
    /// `value` — `Optional<T>` type-erased.
    pub value: Option<Box<dyn std::any::Any + Send + Sync>>,
}

impl Update {
    /// `new(subscription, value)`.
    pub fn new(subscription: DebugSubscription, value: Option<Box<dyn std::any::Any + Send + Sync>>) -> Self {
        Self { subscription, value }
    }

    /// `packUpdate(value)`.
    pub fn pack_update(subscription: &DebugSubscription, value: Option<Box<dyn std::any::Any + Send + Sync>>) -> Self {
        Self::new(subscription.clone(), value)
    }

    /// `emptyUpdate()`.
    pub fn empty_update(subscription: &DebugSubscription) -> Self {
        Self::new(subscription.clone(), None)
    }

    /// `STREAM_CODEC` — `registry(DEBUG_SUBSCRIPTION).dispatch(subscription, streamCodec)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Update> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, update: &Update| {
                buf.write_var_int(update.subscription.registry_id());
                // Write optional value via subscription's value codec
                if let Some(value) = &update.value {
                    // Value codec is type-erased — write as bytes
                    // For now, write as VarInt (simplified)
                    buf.write_var_int(1); // present
                    // Actual value encoding depends on subscription type
                } else {
                    buf.write_var_int(0); // empty
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let sub_id = buf.read_var_int();
                let subscription = DebugSubscription::by_id(sub_id);
                let has_value = buf.read_var_int() != 0;
                let value = if has_value {
                    // Read value — simplified as VarInt
                    Some(Box::new(buf.read_var_int()) as Box<dyn std::any::Any + Send + Sync>)
                } else {
                    None
                };
                Self::new(subscription, value)
            },
        )
    }
}

/// `DebugSubscription.Event<T>` — `{ subscription, T value }`.
#[derive(Debug)]
pub struct Event {
    /// `subscription`.
    pub subscription: DebugSubscription,
    /// `value` — type-erased.
    pub value: Box<dyn std::any::Any + Send + Sync>,
}

impl Event {
    /// `new(subscription, value)`.
    pub fn new(subscription: DebugSubscription, value: Box<dyn std::any::Any + Send + Sync>) -> Self {
        Self { subscription, value }
    }

    /// `packEvent(value)`.
    pub fn pack_event(subscription: &DebugSubscription, value: Box<dyn std::any::Any + Send + Sync>) -> Self {
        Self::new(subscription.clone(), value)
    }

    /// `STREAM_CODEC` — `registry(DEBUG_SUBSCRIPTION).dispatch(subscription, streamCodec)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Event> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, event: &Event| {
                buf.write_var_int(event.subscription.registry_id());
                // Write value — simplified as VarInt
                if let Some(v) = event.value.downcast_ref::<i32>() {
                    buf.write_var_int(*v);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let sub_id = buf.read_var_int();
                let subscription = DebugSubscription::by_id(sub_id);
                let value = Box::new(buf.read_var_int()) as Box<dyn std::any::Any + Send + Sync>;
                Self::new(subscription, value)
            },
        )
    }
}
