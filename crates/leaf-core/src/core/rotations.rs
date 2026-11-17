//! `Rotations` — `{ x, y, z }` float, нормализация `% 360`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `Rotations`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotations {
    /// `x`.
    pub x: f32,
    /// `y`.
    pub y: f32,
    /// `z`.
    pub z: f32,
}

impl Rotations {
    /// `new(x, y, z)` — нормализация `% 360`, NaN/Inf → 0.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let norm = |v: f32| if !v.is_infinite() && !v.is_nan() { v % 360.0 } else { 0.0 };
        Self { x: norm(x), y: norm(y), z: norm(z) }
    }

    /// `ZERO`.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    /// `STREAM_CODEC` — 3 floats.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Rotations> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, r: &Rotations| {
                buf.write_float(r.x);
                buf.write_float(r.y);
                buf.write_float(r.z);
            },
            |buf: &mut FriendlyByteBuf| {
                Rotations::new(buf.read_float(), buf.read_float(), buf.read_float())
            },
        )
    }
}
