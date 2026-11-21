//! `net.minecraft.world.timeline.AttributeTrackSampler` —
//! сэмплер значения трека с кэшем аргумента по `cacheTickId`.

use crate::util::keyframe_track_sampler::KeyframeTrackSampler;
use crate::world::attribute::attribute_modifier::AttributeModifier;
use crate::world::attribute::attribute_value::AttributeValue;
use crate::world::attribute::lerp_function::LerpFunction;
use crate::util::lerp_function::LerpFunction as TrackLerp;
use std::sync::{Arc, Mutex};

use super::attribute_track::DayTimeGetter;

/// Мост `world.attribute.LerpFunction` → `util.LerpFunction<AttributeValue>`.
pub(crate) fn attribute_value_lerp(lerp: LerpFunction) -> TrackLerp<AttributeValue> {
    Arc::new(move |delta: f32, start: AttributeValue, end: AttributeValue| {
        lerp.apply(delta, &start, &end)
    })
}

/// `AttributeTrackSampler<Value, Argument>` — type-erased.
/// Кэширует `Argument` по `cacheTickId`, как в Java.
pub struct AttributeTrackSampler {
    modifier: AttributeModifier,
    argument_sampler: KeyframeTrackSampler<AttributeValue>,
    day_time_getter: DayTimeGetter,
    /// `(cachedTickId, cachedArgument)`.
    cached: Mutex<(i64, Option<AttributeValue>)>,
}

impl AttributeTrackSampler {
    /// Конструктор.
    pub fn new(
        modifier: AttributeModifier,
        argument_sampler: KeyframeTrackSampler<AttributeValue>,
        day_time_getter: DayTimeGetter,
    ) -> Self {
        Self {
            modifier,
            argument_sampler,
            day_time_getter,
            cached: Mutex::new((0, None)),
        }
    }

    /// `applyTimeBased(baseValue, cacheTickId)` — `EnvironmentAttributeLayer.TimeBased`.
    pub fn apply_time_based(&self, base_value: &AttributeValue, cache_tick_id: i64) -> AttributeValue {
        let mut cached = self.cached.lock().unwrap();
        if cached.1.is_none() || cache_tick_id != cached.0 {
            cached.0 = cache_tick_id;
            cached.1 = Some(self.argument_sampler.sample((self.day_time_getter)()));
        }
        self.modifier.apply(base_value, cached.1.as_ref().unwrap())
    }
}
