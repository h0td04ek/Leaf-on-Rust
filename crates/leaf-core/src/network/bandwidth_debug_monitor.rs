//! Перенос `net.minecraft.network.BandwidthDebugMonitor`.

use crate::util::debugchart::{SampleLogger, SampleStorage};
use std::sync::atomic::{AtomicI64, Ordering};

/// `BandwidthDebugMonitor` — считает принятые байты в секунду.
pub struct BandwidthDebugMonitor<L: SampleLogger + SampleStorage> {
    bytes_received: AtomicI64,
    bandwidth_logger: L,
}

impl<L: SampleLogger + SampleStorage> BandwidthDebugMonitor<L> {
    /// `new BandwidthDebugMonitor(bandwidthLogger)`.
    pub fn new(bandwidth_logger: L) -> Self {
        Self { bytes_received: AtomicI64::new(0), bandwidth_logger }
    }

    /// `onReceive(amount)`.
    pub fn on_receive(&self, amount: i64) {
        self.bytes_received.fetch_add(amount, Ordering::Relaxed);
    }

    /// `tick()` — сбросить счётчик в логгер.
    pub fn tick(&mut self) {
        let value = self.bytes_received.swap(0, Ordering::Relaxed);
        self.bandwidth_logger.log_sample(value);
    }

    /// Доступ к логгеру (для debug-вывода).
    pub fn logger(&self) -> &L {
        &self.bandwidth_logger
    }
}
