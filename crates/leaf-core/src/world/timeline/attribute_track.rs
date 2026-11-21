//! `net.minecraft.world.timeline.AttributeTrack` —
//! трек модификатора + keyframe-трек аргумента (type-erased:
//! `Argument` → `AttributeValue`).

use crate::util::keyframe_track::KeyframeTrack;
use crate::world::attribute::attribute_modifier::AttributeModifier;
use crate::world::attribute::attribute_value::AttributeValue;
use crate::world::attribute::environment_attribute::EnvironmentAttribute;
use crate::world::attribute::lerp_function::LerpFunction;
use std::sync::Arc;

use super::attribute_track_sampler::{attribute_value_lerp, AttributeTrackSampler};

/// `record AttributeTrack(modifier, argumentTrack)`.
#[derive(Clone, Debug)]
pub struct AttributeTrack {
    /// `modifier`.
    pub modifier: AttributeModifier,
    /// `argumentTrack` — `KeyframeTrack<Argument>`.
    pub argument_track: KeyframeTrack<AttributeValue>,
}

/// `DayTimeGetter` — `LongSupplier` для `dayTime`.
pub type DayTimeGetter = Arc<dyn Fn() -> i64 + Send + Sync>;

impl AttributeTrack {
    /// `AttributeTrack(modifier, argumentTrack)`.
    pub fn new(modifier: AttributeModifier, argument_track: KeyframeTrack<AttributeValue>) -> Self {
        Self { modifier, argument_track }
    }

    /// `bakeSampler(attribute, periodTicks, dayTimeGetter)`.
    pub fn bake_sampler(
        &self,
        _attribute: &EnvironmentAttribute,
        period_ticks: Option<i32>,
        day_time_getter: DayTimeGetter,
    ) -> AttributeTrackSampler {
        let lerp = self.modifier.argument_keyframe_lerp();
        let argument_sampler = self
            .argument_track
            .bake_sampler(period_ticks, attribute_value_lerp(lerp));
        AttributeTrackSampler::new(self.modifier, argument_sampler, day_time_getter)
    }

    /// `validatePeriod(track, periodTicks)` — DataResult-аналог.
    pub fn validate_period(track: &AttributeTrack, period_ticks: i32) -> Result<(), String> {
        KeyframeTrack::validate_period(&track.argument_track, period_ticks).map(|_| ())
    }
}
