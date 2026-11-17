//! Перенос `net.minecraft.core.particles.ParticleType` /
//! `SimpleParticleType`.

use crate::resources::identifier::Identifier;
use std::sync::Arc;

/// Вид payload-опций частицы (аналог `ParticleType<T>` дженерика).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleOptionsKind {
    /// `SimpleParticleType` — без payload.
    Simple,
    /// `BlockParticleOption` — `BlockState` (id в BLOCK_STATE_REGISTRY).
    Block,
    /// `ColorParticleOption` — INT цвет.
    Color,
    /// `SpellParticleOption` — INT color + FLOAT power.
    Spell,
    /// `DustParticleOptions` — INT color + FLOAT scale.
    Dust,
    /// `DustColorTransitionOptions` — INT from + INT to + FLOAT scale.
    DustColorTransition,
    /// `SculkChargeParticleOptions` — FLOAT roll.
    SculkCharge,
    /// `ItemParticleOption` — `ItemStack.STREAM_CODEC`.
    Item,
    /// `VibrationParticleOption` — `PositionSource` + VAR_INT ticks.
    Vibration,
    /// `TrailParticleOption` — `Vec3` + INT color + VAR_INT duration.
    Trail,
    /// `PowerParticleOption` — FLOAT power.
    Power,
    /// `ShriekParticleOption` — VAR_INT delay.
    Shriek,
}

/// `ParticleType` — запись реестра `Registries.PARTICLE_TYPE`.
#[derive(Clone)]
pub struct ParticleType {
    /// Registry-ключ (`minecraft:flame`).
    pub id: Identifier,
    /// `overrideLimiter`.
    pub override_limiter: bool,
    /// Вид опций (None = SimpleParticleType).
    pub options_kind: Option<ParticleOptionsKind>,
}

impl ParticleType {
    /// `new`.
    pub fn new(id: Identifier, override_limiter: bool, options_kind: Option<ParticleOptionsKind>) -> Self {
        Self { id, override_limiter, options_kind }
    }

    /// `getOverrideLimiter`.
    pub fn get_override_limiter(&self) -> bool {
        self.override_limiter
    }
}

impl PartialEq for ParticleType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for ParticleType {}
impl std::hash::Hash for ParticleType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl std::fmt::Debug for ParticleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ParticleType").field(&self.id).finish()
    }
}

/// `Arc<ParticleType>` — разделяемый тип частицы.
pub type SharedParticleType = Arc<ParticleType>;
