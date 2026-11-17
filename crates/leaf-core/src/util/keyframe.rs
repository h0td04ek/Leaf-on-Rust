//! Перенос `net.minecraft.util.Keyframe<T>` (record).
//! DFU-кодек (`codec(valueCodec)`) — позже.

/// `Keyframe(ticks, value)` — ключевой кадр трека.
#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe<T> {
    pub ticks: i32,
    pub value: T,
}

impl<T> Keyframe<T> {
    pub fn new(ticks: i32, value: T) -> Self {
        Self { ticks, value }
    }
}
