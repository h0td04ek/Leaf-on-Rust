//! Перенос `net.minecraft.util.TickThrottler`.

use std::sync::atomic::{AtomicI32, Ordering};

/// `TickThrottler(incrementStep, threshold)` — троттлер для событий/тиков.
#[derive(Debug)]
pub struct TickThrottler {
    increment_step: i32,
    threshold: i32,
    count: AtomicI32,
}

impl TickThrottler {
    pub fn new(increment_step: i32, threshold: i32) -> Self {
        Self { increment_step, threshold, count: AtomicI32::new(0) }
    }

    /// `increment()`
    pub fn increment(&self) {
        self.count.fetch_add(self.increment_step, Ordering::SeqCst);
    }

    /// `tick()` — декремент, если > 0.
    pub fn tick(&self) {
        self.count.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| if v > 0 { Some(v - 1) } else { None }).ok();
    }

    pub fn is_under_threshold(&self) -> bool {
        self.count.load(Ordering::SeqCst) < self.threshold
    }

    pub fn is_increment_and_under_threshold(&self) -> bool {
        self.count.fetch_add(self.increment_step, Ordering::SeqCst) + self.increment_step < self.threshold
    }

    pub fn is_increment_and_under_threshold_with(&self, increment_step: i32, threshold: i32) -> bool {
        self.count.fetch_add(increment_step, Ordering::SeqCst) + increment_step < threshold
    }
}
