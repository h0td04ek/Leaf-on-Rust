//! Перенос `net.minecraft.util.KeyframeTrack<T>` (record).

use crate::util::easing_type::{EasingType, SimpleEasing};
use crate::util::keyframe::Keyframe;
use crate::util::keyframe_track_sampler::KeyframeTrackSampler;
use crate::util::lerp_function::LerpFunction;

/// `KeyframeTrack(keyframes, easingType)`.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyframeTrack<T> {
    pub keyframes: Vec<Keyframe<T>>,
    pub easing_type: EasingType,
}

impl<T: Clone> KeyframeTrack<T> {
    /// Конструктор с проверкой: пустой трек → panic.
    pub fn new(keyframes: Vec<Keyframe<T>>, easing_type: EasingType) -> Result<Self, String> {
        if keyframes.is_empty() {
            return Err("Track has no keyframes".to_string());
        }
        Self::validate_keyframes(&keyframes)?;
        Ok(Self { keyframes, easing_type })
    }

    /// `validateKeyframes` → DataResult-аналог: Result<(), String>.
    pub fn validate_keyframes(keyframes: &[Keyframe<T>]) -> Result<(), String> {
        if keyframes.is_empty() {
            return Err("Keyframes must not be empty".to_string());
        }
        // isInOrder по ticks
        for w in keyframes.windows(2) {
            if w[0].ticks > w[1].ticks {
                return Err("Keyframes must be ordered by ticks field".to_string());
            }
        }
        if keyframes.len() > 1 {
            let mut i = 0;
            let mut ticks = keyframes.last().unwrap().ticks;
            for keyframe in keyframes {
                if keyframe.ticks == ticks {
                    i += 1;
                    if i > 2 {
                        return Err(format!("More than 2 keyframes on same tick: {}", keyframe.ticks));
                    }
                } else {
                    i = 0;
                }
                ticks = keyframe.ticks;
            }
        }
        Ok(())
    }

    /// `validatePeriod(track, periodTicks)` — тики в [0; period].
    pub fn validate_period(track: &KeyframeTrack<T>, period_ticks: i32) -> Result<(), String> {
        for keyframe in &track.keyframes {
            let ticks = keyframe.ticks;
            if ticks < 0 || ticks > period_ticks {
                return Err(format!(
                    "Keyframe at tick {} must be in range [0; {}]",
                    keyframe.ticks, period_ticks
                ));
            }
        }
        Ok(())
    }

    /// `bakeSampler(periodTicks, lerp)`.
    pub fn bake_sampler(
        &self,
        period_ticks: Option<i32>,
        lerp: LerpFunction<T>,
    ) -> KeyframeTrackSampler<T> {
        KeyframeTrackSampler::new(self.clone(), period_ticks, lerp)
    }
}

/// `KeyframeTrack.Builder<T>`.
pub struct KeyframeTrackBuilder<T> {
    keyframes: Vec<Keyframe<T>>,
    easing: EasingType,
}

impl<T: Clone> KeyframeTrackBuilder<T> {
    pub fn new() -> Self {
        Self { keyframes: Vec::new(), easing: EasingType::Simple(SimpleEasing::Linear) }
    }

    /// `addKeyframe(ticks, value)`.
    pub fn add_keyframe(&mut self, ticks: i32, value: T) -> &mut Self {
        self.keyframes.push(Keyframe::new(ticks, value));
        self
    }

    /// `setEasing(easing)`.
    pub fn set_easing(&mut self, easing: EasingType) -> &mut Self {
        self.easing = easing;
        self
    }

    /// `build()` — с валидацией.
    pub fn build(self) -> Result<KeyframeTrack<T>, String> {
        KeyframeTrack::validate_keyframes(&self.keyframes)?;
        Ok(KeyframeTrack { keyframes: self.keyframes, easing_type: self.easing })
    }
}

impl<T: Clone> Default for KeyframeTrackBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
