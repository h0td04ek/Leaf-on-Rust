//! Перенос `net.minecraft.util.debugchart.SampleLogger`.

/// `SampleLogger` — интерфейс записи сэмплов.
pub trait SampleLogger {
    /// `logFullSample(long[])` — записать все измерения.
    fn log_full_sample(&mut self, sample: &[i64]);
    /// `logSample(long)` — записать одно значение (index 0).
    fn log_sample(&mut self, value: i64);
    /// `logPartialSample(long, index)` — записать одно измерение.
    fn log_partial_sample(&mut self, value: i64, index: usize);
}
