//! Перенос `net.minecraft.util.debugchart.RemoteDebugSampleType`.

/// `RemoteDebugSampleType` — тип сэмпла debug-метрик.
/// Java: `TICK_TIME(DebugSubscriptions.DEDICATED_SERVER_TICK_TIME)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RemoteDebugSampleType {
    /// `TICK_TIME`.
    TickTime = 0,
}

/// Все значения (`values()`).
pub const REMOTE_DEBUG_SAMPLE_TYPES: [RemoteDebugSampleType; 1] = [RemoteDebugSampleType::TickTime];

impl RemoteDebugSampleType {
    /// `values()`.
    pub fn values() -> &'static [RemoteDebugSampleType] {
        &REMOTE_DEBUG_SAMPLE_TYPES
    }

    /// `byId` — ordinal.
    pub fn by_id(id: i32) -> RemoteDebugSampleType {
        REMOTE_DEBUG_SAMPLE_TYPES[id.rem_euclid(1) as usize]
    }

    /// `ordinal` — `readEnum`/`writeEnum` пишут VarInt ordinal.
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }

    /// `subscription()` — `DebugSubscription` (портируется в `util::debug`).
    /// Для кодека пакета не нужен — возвращаем маркер.
    pub fn subscription(&self) -> crate::util::debug::debug_subscription::DebugSubscription {
        crate::util::debug::debug_subscription::DebugSubscription::tick_time()
    }
}
