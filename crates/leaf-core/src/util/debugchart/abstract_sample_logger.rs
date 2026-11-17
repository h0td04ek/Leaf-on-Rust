//! Перенос `net.minecraft.util.debugchart.AbstractSampleLogger`.

use crate::util::debugchart::sample_logger::SampleLogger;

/// `AbstractSampleLogger` — кольцевая запись с дефолтами.
pub struct AbstractSampleLogger {
    /// `defaults` — значения после resetSample.
    pub defaults: Vec<i64>,
    /// `sample` — текущий накапливаемый сэмпл.
    pub sample: Vec<i64>,
}

impl AbstractSampleLogger {
    /// `new AbstractSampleLogger(size, defaults)`.
    pub fn new(size: usize, defaults: Vec<i64>) -> Self {
        assert_eq!(
            defaults.len(),
            size,
            "defaults have incorrect length of {}",
            defaults.len()
        );
        Self { defaults, sample: vec![0; size] }
    }

    /// `useSample()` — зафиксировать текущий sample (в подклассе).
    fn use_sample(&mut self) {
        let _ = &self.sample;
    }

    /// `resetSample()` — вернуть defaults в sample.
    pub fn reset_sample(&mut self) {
        self.sample.copy_from_slice(&self.defaults);
    }
}

impl SampleLogger for AbstractSampleLogger {
    fn log_full_sample(&mut self, sample: &[i64]) {
        let n = sample.len().min(self.sample.len());
        self.sample[..n].copy_from_slice(&sample[..n]);
        self.use_sample();
        self.reset_sample();
    }

    fn log_sample(&mut self, value: i64) {
        self.sample[0] = value;
        self.use_sample();
        self.reset_sample();
    }

    fn log_partial_sample(&mut self, value: i64, index: usize) {
        assert!(
            index >= 1 && index < self.sample.len(),
            "{} out of bounds for dimensions {}",
            index,
            self.sample.len()
        );
        self.sample[index] = value;
    }
}
