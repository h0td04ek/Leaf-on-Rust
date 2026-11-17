//! Типы значений `EnvironmentAttribute` — аналог Java-дженериков
//! `AttributeType<Value>` в типизированном виде.

use crate::core::particles::particle_options::ParticleOptions;
use crate::entity::schedule::activity::Activity;
use crate::util::tri_state::TriState;
use crate::world::attribute::bed_rule::BedRule;
use crate::world::level::moon_phase::MoonPhase;

/// `AttributeValue` — значение атрибута (type-erased аналог `Value`).
#[derive(Clone, Debug, PartialEq)]
pub enum AttributeValue {
    /// `BOOLEAN`.
    Boolean(bool),
    /// `TRI_STATE`.
    TriState(TriState),
    /// `FLOAT` / `ANGLE_DEGREES`.
    Float(f32),
    /// `RGB_COLOR` / `ARGB_COLOR` / целочисленные значения.
    Int(i32),
    /// `MOON_PHASE`.
    MoonPhase(MoonPhase),
    /// `ACTIVITY`.
    Activity(Activity),
    /// `BED_RULE`.
    BedRule(BedRule),
    /// `PARTICLE`.
    Particle(ParticleOptions),
    /// `AMBIENT_PARTICLES`.
    AmbientParticles(Vec<AmbientParticle>),
    /// `BACKGROUND_MUSIC`.
    BackgroundMusic(BackgroundMusic),
    /// `AMBIENT_SOUNDS`.
    AmbientSounds(AmbientSounds),
    /// Аргумент `ALPHA_BLEND` для float (`FloatWithAlpha`).
    FloatWithAlpha { value: f32, alpha: f32 },
    /// Аргумент `BLEND_TO_GRAY` для цветов.
    BlendToGray { brightness: f32, factor: f32 },
}

/// `AmbientParticle` — `record(ParticleOptions particle, float chance)`.
#[derive(Clone, Debug, PartialEq)]
pub struct AmbientParticle {
    /// Частица.
    pub particle: ParticleOptions,
    /// Шанс появления 0..1.
    pub chance: f32,
}

/// `BackgroundMusic` — фоновая музыка биома.
#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundMusic {
    /// `replaceCurrentMusic`.
    pub replace_current_music: bool,
    /// `sound` — Holder<SoundEvent>-эквивалент (id + опциональный прямой).
    pub sound: Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>,
    /// `minDelay` (ticks).
    pub min_delay: i32,
    /// `maxDelay` (ticks).
    pub max_delay: i32,
    /// `soundEvent`-variant (в Java — Either<SoundEvent, SoundEvent>>).
    pub sound_event: Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>,
}

impl BackgroundMusic {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self {
            replace_current_music: false,
            sound: None,
            min_delay: 0,
            max_delay: 0,
            sound_event: None,
        }
    }
}

/// `AmbientAdditionsSettings` — настройки добавочных эмбиент-звуков.
#[derive(Clone, Debug, PartialEq)]
pub struct AmbientAdditionsSettings {
    /// `sound`.
    pub sound: Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>,
    /// `tickChance`.
    pub tick_chance: f64,
}

/// `AmbientMoodSettings` — настройки эмбиент-настроения.
#[derive(Clone, Debug, PartialEq)]
pub struct AmbientMoodSettings {
    /// `sound`.
    pub sound: Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>,
    /// `tickDelay`.
    pub tick_delay: i32,
    /// `blockSearchExtent`.
    pub block_search_extent: i32,
    /// `offset`.
    pub offset: f64,
}

/// `AmbientLoopSettings` — настройки зацикленного эмбиент-звука.
#[derive(Clone, Debug, PartialEq)]
pub struct AmbientLoopSettings {
    /// `sound`.
    pub sound: Option<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>,
    /// `delay`.
    pub delay: f64,
    /// `volume`.
    pub volume: f32,
}

/// `AmbientSounds` — все эмбиент-звуки биома.
#[derive(Clone, Debug, PartialEq)]
pub struct AmbientSounds {
    /// `additions`.
    pub additions: AmbientAdditionsSettings,
    /// `mood`.
    pub mood: AmbientMoodSettings,
    /// `loop`.
    pub loop_sound: AmbientLoopSettings,
}

impl AmbientSounds {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self {
            additions: AmbientAdditionsSettings { sound: None, tick_chance: 0.0 },
            mood: AmbientMoodSettings { sound: None, tick_delay: 0, block_search_extent: 0, offset: 0.0 },
            loop_sound: AmbientLoopSettings { sound: None, delay: 0.0, volume: 1.0 },
        }
    }
}
