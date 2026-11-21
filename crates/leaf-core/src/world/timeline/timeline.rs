//! Перенос `net.minecraft.world.timeline.Timeline` —
//! таймлайн окружения (период + треки атрибутов).
//! Кодеки (`CODEC`/`DIRECT_CODEC`/`NETWORK_CODEC`) — как и в
//! остальных модулях, подключаются вместе с портом Mojang Codec.

use crate::world::attribute::attribute_modifier::AttributeModifier;
use crate::world::attribute::attribute_value::AttributeValue;
use crate::world::attribute::environment_attribute::EnvironmentAttribute;
use crate::util::keyframe_track::{KeyframeTrackBuilder};
use crate::world::level::Level;
use std::sync::Arc;

use super::attribute_track::{AttributeTrack, DayTimeGetter};
use super::attribute_track_sampler::AttributeTrackSampler;

/// `Timeline` — `periodTicks` + `tracks`.
/// Java `Map<EnvironmentAttribute<?>, AttributeTrack<?, ?>>`
/// → `Vec<(Arc<EnvironmentAttribute>, AttributeTrack)>`
/// (порядок вставки сохраняется, ключи уникальны по `id`).
#[derive(Clone, Debug)]
pub struct Timeline {
    /// `periodTicks` — `Optional<Integer>`.
    pub period_ticks: Option<i32>,
    /// `tracks`.
    pub tracks: Vec<(Arc<EnvironmentAttribute>, AttributeTrack)>,
}

impl Timeline {
    /// `Timeline(periodTicks, tracks)` — приватный конструктор Java;
    /// сборка через `Timeline::builder()`.
    pub(crate) fn new(
        period_ticks: Option<i32>,
        tracks: Vec<(Arc<EnvironmentAttribute>, AttributeTrack)>,
    ) -> Self {
        Self { period_ticks, tracks }
    }

    /// `getCurrentTicks(level)`.
    pub fn get_current_ticks(&self, level: &dyn Level) -> i64 {
        let total = self.get_total_ticks(level);
        match self.period_ticks {
            None => total,
            Some(p) => total % p as i64,
        }
    }

    /// `getTotalTicks(level)` — `level.getDayTime()`.
    pub fn get_total_ticks(&self, level: &dyn Level) -> i64 {
        Level::day_time(level)
    }

    /// `periodTicks()`.
    pub fn period_ticks(&self) -> Option<i32> {
        self.period_ticks
    }

    /// `attributes()` — множество атрибутов треков.
    pub fn attributes(&self) -> impl Iterator<Item = &Arc<EnvironmentAttribute>> {
        self.tracks.iter().map(|(a, _)| a)
    }

    /// `createTrackSampler(attribute, dayTimeGetter)`.
    /// Бросает `IllegalStateException`, если трека нет —
    /// здесь `Err(String)` (DataResult-стиль модуля).
    pub fn create_track_sampler(
        &self,
        attribute: &EnvironmentAttribute,
        day_time_getter: DayTimeGetter,
    ) -> Result<AttributeTrackSampler, String> {
        let track = self
            .tracks
            .iter()
            .find(|(a, _)| a.id == attribute.id)
            .map(|(_, t)| t)
            .ok_or_else(|| format!("Timeline has no track for {}", attribute.id.to_short_string()))?;
        Ok(track.bake_sampler(attribute, self.period_ticks, day_time_getter))
    }

    /// `builder()`.
    pub fn builder() -> TimelineBuilder {
        TimelineBuilder::new()
    }

    /// `filterSyncableTracks` — для `NETWORK_CODEC`.
    pub fn filter_syncable_tracks(&self, syncable: &[Arc<EnvironmentAttribute>]) -> Self {
        let tracks = self
            .tracks
            .iter()
            .filter(|(attr, _)| syncable.iter().any(|s| s.id == attr.id && s.is_syncable()))
            .cloned()
            .collect();
        Self { period_ticks: self.period_ticks, tracks }
    }
}

/// `Timeline.Builder`.
#[derive(Default)]
pub struct TimelineBuilder {
    period_ticks: Option<i32>,
    tracks: Vec<(Arc<EnvironmentAttribute>, AttributeTrack)>,
}

impl TimelineBuilder {
    /// `Builder()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `setPeriodTicks`.
    pub fn set_period_ticks(&mut self, period_ticks: i32) -> &mut Self {
        self.period_ticks = Some(period_ticks);
        self
    }

    /// `addModifierTrack(attribute, modifier, builderConsumer)`.
    /// `checkAllowedModifier` — panic при нарушении (как Java throw).
    pub fn add_modifier_track(
        &mut self,
        attribute: Arc<EnvironmentAttribute>,
        modifier: AttributeModifier,
        build: impl FnOnce(&mut KeyframeTrackBuilder<AttributeValue>),
    ) -> &mut Self {
        attribute
            .check_allowed_modifier(&modifier)
            .unwrap_or_else(|e| panic!("{}", e));
        let mut kb = KeyframeTrackBuilder::new();
        build(&mut kb);
        let argument_track = kb
            .build_ref()
            .unwrap_or_else(|e| panic!("invalid keyframe track: {}", e));
        self.tracks.push((attribute, AttributeTrack::new(modifier, argument_track)));
        self
    }

    /// `addTrack(attribute, builderConsumer)` — `AttributeModifier.override()`.
    pub fn add_track(
        &mut self,
        attribute: Arc<EnvironmentAttribute>,
        build: impl FnOnce(&mut KeyframeTrackBuilder<AttributeValue>),
    ) -> &mut Self {
        self.add_modifier_track(attribute, AttributeModifier::Override, build)
    }

    /// `build()` — `ImmutableMap.Builder.build()`: дубликаты ключей
    /// → ошибка (Java `IllegalArgumentException`).
    pub fn build(self) -> Result<Timeline, String> {
        for (i, (a, _)) in self.tracks.iter().enumerate() {
            if self.tracks[..i].iter().any(|(b, _)| b.id == a.id) {
                return Err(format!("Duplicate timeline track for {}", a.id.to_short_string()));
            }
        }
        Ok(Timeline::new(self.period_ticks, self.tracks))
    }
}
