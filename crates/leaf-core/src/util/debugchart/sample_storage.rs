//! Перенос `net.minecraft.util.debugchart.SampleStorage`.

/// `SampleStorage` — доступ к записанным сэмплам.
pub trait SampleStorage {
    /// `capacity()` — максимум хранимых сэмплов.
    fn capacity(&self) -> usize;
    /// `size()` — текущее число сэмплов.
    fn size(&self) -> usize;
    /// `get(index)` — первое измерение.
    fn get(&self, index: usize) -> i64;
    /// `get(index, dimension)`.
    fn get_dim(&self, index: usize, dimension: usize) -> i64;
    /// `reset()`.
    fn reset(&mut self);
}
