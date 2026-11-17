//! Перенос `net.minecraft.sounds.SoundSource`.

/// `SoundSource` — источник звука (11 значений).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundSource {
    /// `MASTER`.
    Master = 0,
    /// `MUSIC`.
    Music = 1,
    /// `RECORDS`.
    Records = 2,
    /// `WEATHER`.
    Weather = 3,
    /// `BLOCKS`.
    Blocks = 4,
    /// `HOSTILE`.
    Hostile = 5,
    /// `NEUTRAL`.
    Neutral = 6,
    /// `PLAYERS`.
    Players = 7,
    /// `AMBIENT`.
    Ambient = 8,
    /// `VOICE`.
    Voice = 9,
    /// `UI`.
    Ui = 10,
}

/// Все значения (`values()`).
pub const SOUND_SOURCES: [SoundSource; 11] = [
    SoundSource::Master,
    SoundSource::Music,
    SoundSource::Records,
    SoundSource::Weather,
    SoundSource::Blocks,
    SoundSource::Hostile,
    SoundSource::Neutral,
    SoundSource::Players,
    SoundSource::Ambient,
    SoundSource::Voice,
    SoundSource::Ui,
];

impl SoundSource {
    /// `values()`.
    pub fn values() -> &'static [SoundSource] {
        &SOUND_SOURCES
    }

    /// `byId` — ordinal.
    pub fn by_id(id: i32) -> SoundSource {
        SOUND_SOURCES[id.rem_euclid(11) as usize]
    }

    /// `ordinal` (`writeEnum` = VarInt ordinal).
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }

    /// `getName`.
    pub fn get_name(&self) -> &'static str {
        match self {
            SoundSource::Master => "master",
            SoundSource::Music => "music",
            SoundSource::Records => "record",
            SoundSource::Weather => "weather",
            SoundSource::Blocks => "block",
            SoundSource::Hostile => "hostile",
            SoundSource::Neutral => "neutral",
            SoundSource::Players => "player",
            SoundSource::Ambient => "ambient",
            SoundSource::Voice => "voice",
            SoundSource::Ui => "ui",
        }
    }
}
