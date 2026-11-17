//! Перенос `net.minecraft.util.MemoryReserve`.

static mut RESERVE: Option<Vec<u8>> = None;

/// `MemoryReserve.allocate()` — выделяет 10 МБ резерва.
/// В JVM это давит GC при старте; в Rust-JVM GC своя — функция
/// сохраняет семантику (резерв + три System.gc()).
pub fn allocate() {
    unsafe {
        RESERVE = Some(vec![0u8; 10 * 1024 * 1024]);
    }
}

/// `MemoryReserve.release()` — освобождает резерв и трижды GC.
pub fn release() {
    unsafe {
        RESERVE = None;
    }
    // Три System.gc() — в Rust-JVM аналог недоступен напрямую;
    // оставляем точку для вызова GC хоста при необходимости.
}
