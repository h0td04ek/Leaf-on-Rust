//! Перенос `net.minecraft.util.debugchart.LocalSampleLogger`.

use crate::util::debugchart::abstract_sample_logger::AbstractSampleLogger;
use crate::util::debugchart::sample_logger::SampleLogger;
use crate::util::debugchart::sample_storage::SampleStorage;

/// `LocalSampleLogger.CAPACITY`.
pub const CAPACITY: usize = 240;

/// `LocalSampleLogger` — хранит до 240 сэмплов в памяти.
pub struct LocalSampleLogger {
    inner: AbstractSampleLogger,
    /// `samples[240][size]`.
    samples: Vec<Vec<i64>>,
    start: usize,
    size: usize,
}

impl LocalSampleLogger {
    /// `new LocalSampleLogger(size)` — defaults = нули.
    pub fn new(size: usize) -> Self {
        Self::with_defaults(size, vec![0; size])
    }

    /// `new LocalSampleLogger(size, defaults)`.
    pub fn with_defaults(size: usize, defaults: Vec<i64>) -> Self {
        Self {
            inner: AbstractSampleLogger::new(size, defaults),
            samples: vec![vec![0; size]; CAPACITY],
            start: 0,
            size: 0,
        }
    }

    fn wrap_index(&self, index: usize) -> usize {
        index % CAPACITY
    }
}

impl SampleLogger for LocalSampleLogger {
    fn log_full_sample(&mut self, sample: &[i64]) {
        let n = sample.len().min(self.inner.sample.len());
        self.inner.sample[..n].copy_from_slice(&sample[..n]);
        self.use_sample();
        self.inner.reset_sample();
    }

    fn log_sample(&mut self, value: i64) {
        self.inner.sample[0] = value;
        self.use_sample();
        self.inner.reset_sample();
    }

    fn log_partial_sample(&mut self, value: i64, index: usize) {
        assert!(
            index >= 1 && index < self.inner.sample.len(),
            "{} out of bounds for dimensions {}",
            index,
            self.inner.sample.len()
        );
        self.inner.sample[index] = value;
    }
}

impl LocalSampleLogger {
    /// `useSample()`.
    fn use_sample(&mut self) {
        let idx = self.wrap_index(self.start + self.size);
        let n = self.inner.sample.len();
        self.samples[idx][..n].copy_from_slice(&self.inner.sample[..n]);
        if self.size < CAPACITY {
            self.size += 1;
        } else {
            self.start = self.wrap_index(self.start + 1);
        }
    }
}

impl SampleStorage for LocalSampleLogger {
    fn capacity(&self) -> usize {
        self.samples.len()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn get(&self, index: usize) -> i64 {
        self.get_dim(index, 0)
    }

    fn get_dim(&self, index: usize, dimension: usize) -> i64 {
        assert!(
            index < self.size,
            "{} out of bounds for length {}",
            index,
            self.size
        );
        let row = &self.samples[self.wrap_index(self.start + index)];
        assert!(
            dimension < row.len(),
            "{} out of bounds for dimensions {}",
            dimension,
            row.len()
        );
        row[dimension]
    }

    fn reset(&mut self) {
        self.start = 0;
        self.size = 0;
    }
}
