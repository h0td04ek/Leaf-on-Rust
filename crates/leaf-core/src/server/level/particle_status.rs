//! Перенос `net.minecraft.server.level.ParticleStatus`.

use crate::network::chat::component::Component;

/// `ParticleStatus` — настройка частиц клиента.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticleStatus {
    All,
    Decreased,
    Minimal,
}

impl ParticleStatus {
    /// Порядок в `values()`.
    pub const VALUES: [ParticleStatus; 3] = [
        ParticleStatus::All,
        ParticleStatus::Decreased,
        ParticleStatus::Minimal,
    ];

    /// `id` (0,1,2).
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `BY_ID` (WRAP).
    pub fn by_id_wrapped(id: i32) -> ParticleStatus {
        let n = id.rem_euclid(3);
        ParticleStatus::VALUES[n as usize]
    }

    /// `caption()`.
    pub fn caption(&self) -> Component {
        let key = match self {
            ParticleStatus::All => "options.particles.all",
            ParticleStatus::Decreased => "options.particles.decreased",
            ParticleStatus::Minimal => "options.particles.minimal",
        };
        Component::translatable(key, Vec::new())
    }
}
