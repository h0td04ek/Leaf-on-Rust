//! Перенос `net.minecraft.world.attribute.LerpFunction<T>`.
//! Размещён в util-логике, т.к. используется KeyframeTrackSampler;
//! пакетная структура `world/attribute` появится при порте атрибутов.

use crate::util::argb;
use crate::util::mth;
use std::sync::Arc;

/// `LerpFunction<T>` — интерполяция между start и end по delta.
pub type LerpFunction<T> = Arc<dyn Fn(f32, T, T) -> T + Send + Sync>;

/// `LerpFunction.ofFloat()` — обычный float-lerp.
pub fn of_float() -> LerpFunction<f32> {
    Arc::new(|delta, start, end| mth::lerp_f32(delta, start, end))
}

/// `LerpFunction.ofDegrees(maxDelta)` — углы с ограничением скачка.
pub fn of_degrees(max_delta: f32) -> LerpFunction<f32> {
    Arc::new(move |delta, start, end| {
        let f = mth::wrap_degrees_f32(end - start);
        if f.abs() >= max_delta { end } else { start + delta * f }
    })
}

/// `LerpFunction.ofConstant()` — всегда start.
pub fn of_constant<T: Clone + Send + Sync + 'static>() -> LerpFunction<T> {
    Arc::new(|_delta, start, _end| start)
}

/// `LerpFunction.ofStep(threshold)` — скачок при delta >= threshold.
pub fn of_step<T: Clone + Send + Sync + 'static>(threshold: f32) -> LerpFunction<T> {
    Arc::new(move |delta, start, end| {
        if delta >= threshold { end } else { start }
    })
}

/// `LerpFunction.ofColor()` — lerp цветов через ARGB.srgbLerp.
pub fn of_color() -> LerpFunction<i32> {
    Arc::new(|delta, start, end| argb::srgb_lerp(delta, start, end))
}
