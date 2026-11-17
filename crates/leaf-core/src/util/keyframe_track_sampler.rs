//! Перенос `net.minecraft.util.KeyframeTrackSampler<T>`.

use crate::util::easing_type::EasingType;
use crate::util::keyframe::Keyframe;
use crate::util::keyframe_track::KeyframeTrack;
use crate::util::lerp_function::LerpFunction;

/// `KeyframeTrackSampler<T>` — запечённый сэмплер трека.
pub struct KeyframeTrackSampler<T> {
    period_ticks: Option<i32>,
    lerp: LerpFunction<T>,
    segments: Vec<Segment<T>>,
}

#[derive(Clone)]
struct Segment<T> {
    easing: EasingType,
    from_value: T,
    from_ticks: i32,
    to_value: T,
    to_ticks: i32,
}

impl<T: Clone> Segment<T> {
    fn new(track: &KeyframeTrack<T>, from: &Keyframe<T>, from_ticks: i32, to: &Keyframe<T>, to_ticks: i32) -> Self {
        Self {
            easing: track.easing_type.clone(),
            from_value: from.value.clone(),
            from_ticks,
            to_value: to.value.clone(),
            to_ticks,
        }
    }
}

impl<T: Clone> KeyframeTrackSampler<T> {
    pub(crate) fn new(track: KeyframeTrack<T>, period_ticks: Option<i32>, lerp: LerpFunction<T>) -> Self {
        let segments = Self::bake_segments(&track, period_ticks);
        Self { period_ticks, lerp, segments }
    }

    fn bake_segments(track: &KeyframeTrack<T>, period_ticks: Option<i32>) -> Vec<Segment<T>> {
        let list = &track.keyframes;
        if list.len() == 1 {
            let object = list[0].value.clone();
            return vec![Segment {
                easing: EasingType::Simple(crate::util::easing_type::SimpleEasing::Constant),
                from_value: object.clone(),
                from_ticks: 0,
                to_value: object,
                to_ticks: 0,
            }];
        }
        let mut list1 = Vec::new();
        if let Some(period) = period_ticks {
            let keyframe = list.first().unwrap();
            let keyframe1 = list.last().unwrap();
            list1.push(Segment::new(track, keyframe1, keyframe1.ticks - period, keyframe, keyframe.ticks));
            Self::add_segments_from_keyframes(track, list, &mut list1);
            list1.push(Segment::new(track, keyframe1, keyframe1.ticks, keyframe, keyframe.ticks + period));
        } else {
            Self::add_segments_from_keyframes(track, list, &mut list1);
        }
        list1
    }

    fn add_segments_from_keyframes(track: &KeyframeTrack<T>, keyframes: &[Keyframe<T>], output: &mut Vec<Segment<T>>) {
        for i in 0..keyframes.len() - 1 {
            let keyframe = &keyframes[i];
            let keyframe1 = &keyframes[i + 1];
            output.push(Segment::new(track, keyframe, keyframe.ticks, keyframe1, keyframe1.ticks));
        }
    }

    /// `sample(ticks)`.
    pub fn sample(&self, ticks: i64) -> T {
        let l = self.loop_ticks(ticks);
        let segment_at = self.get_segment_at(l);
        if l <= segment_at.from_ticks as i64 {
            segment_at.from_value.clone()
        } else if l >= segment_at.to_ticks as i64 {
            segment_at.to_value.clone()
        } else {
            let f = (l - segment_at.from_ticks as i64) as f32 / (segment_at.to_ticks - segment_at.from_ticks) as f32;
            let f1 = segment_at.easing.apply(f);
            (self.lerp)(f1, segment_at.from_value.clone(), segment_at.to_value.clone())
        }
    }

    fn get_segment_at(&self, current_ticks: i64) -> &Segment<T> {
        for segment in &self.segments {
            if current_ticks < segment.to_ticks as i64 {
                return segment;
            }
        }
        self.segments.last().unwrap()
    }

    /// `loopTicks` — Math.floorMod для period.
    fn loop_ticks(&self, ticks: i64) -> i64 {
        match self.period_ticks {
            Some(period) => ticks.rem_euclid(period as i64),
            None => ticks,
        }
    }
}
