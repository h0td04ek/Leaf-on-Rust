//! Перенос `net.minecraft.world.scores.criteria.ObjectiveCriteria` —
//! критерии целей. Здесь — вложенный `RenderType`; сам реестр
//! критериев (`ObjectiveCriteria` + стандартные записи) портируется
//! с системой счёта.

/// `ObjectiveCriteria.RenderType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderType {
    /// `INTEGER`.
    Integer,
    /// `HEARTS`.
    Hearts,
}

/// Все значения (`values()`).
pub const RENDER_TYPES: [RenderType; 2] = [RenderType::Integer, RenderType::Hearts];

impl RenderType {
    /// `values()`.
    pub fn values() -> &'static [RenderType] {
        &RENDER_TYPES
    }

    /// `getId`.
    pub fn get_id(&self) -> &'static str {
        match self {
            RenderType::Integer => "integer",
            RenderType::Hearts => "hearts",
        }
    }

    /// `getSerializedName` (`StringRepresentable`).
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_id()
    }

    /// `byId` — по сериализованному имени.
    pub fn by_serialized_name(name: &str) -> Option<RenderType> {
        RENDER_TYPES.iter().copied().find(|t| t.get_serialized_name() == name)
    }

    /// ordinal для `writeEnum`.
    pub fn ordinal(&self) -> i32 {
        match self {
            RenderType::Integer => 0,
            RenderType::Hearts => 1,
        }
    }

    /// `byOrdinal` — `readEnum`.
    pub fn by_ordinal(id: i32) -> Option<RenderType> {
        RENDER_TYPES.get(id as usize).copied()
    }
}

/// `ObjectiveCriteria` — критерий цели (id + render type).
///
/// В Java — registry-backed класс со статическими экземплярами
/// (DUMMY, TRIGGER, KILL_COUNT и т.д.); здесь — значение +
/// стандартные константы.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectiveCriteria {
    /// `id` — identifier критерия.
    pub id: crate::resources::identifier::Identifier,
    /// `renderType`.
    pub render_type: RenderType,
}

impl ObjectiveCriteria {
    /// `new(id, renderType)`.
    pub fn new(id: impl Into<crate::resources::identifier::Identifier>, render_type: RenderType) -> Self {
        Self { id: id.into(), render_type }
    }

    /// Стандартные критерии vanilla.
    pub fn dummy() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("dummy"), RenderType::Integer)
    }
    pub fn trigger() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("trigger"), RenderType::Integer)
    }
    pub fn death_count() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("death_count"), RenderType::Integer)
    }
    pub fn player_kill_count() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("player_kill_count"), RenderType::Integer)
    }
    pub fn total_kill_count() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("total_kill_count"), RenderType::Integer)
    }
    pub fn health() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("health"), RenderType::Hearts)
    }
    pub fn food() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("food"), RenderType::Integer)
    }
    pub fn air() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("air"), RenderType::Integer)
    }
    pub fn armor() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("armor"), RenderType::Integer)
    }
    pub fn xp() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("xp"), RenderType::Integer)
    }
    pub fn level() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("level"), RenderType::Integer)
    }
    pub fn team_kill() -> Self {
        Self::new(crate::resources::identifier::Identifier::minecraft("teamkill"), RenderType::Integer)
    }
}
