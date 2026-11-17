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
}

/// `DebugSubscription.DOES_NOT_EXPIRE`.
pub const DOES_NOT_EXPIRE: i32 = 0;

impl DebugSubscription {
    /// `new DebugSubscription(codec, expireAfterTicks)`.
    pub fn new(
        value_stream_codec: Option<Arc<dyn StreamCodec<FriendlyByteBuf, ()> + Send + Sync>>,
        expire_after_ticks: i32,
    ) -> Self {
        Self { value_stream_codec, expire_after_ticks }
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

impl std::fmt::Debug for DebugSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugSubscription")
            .field("expire_after_ticks", &self.expire_after_ticks)
            .finish()
    }
}
