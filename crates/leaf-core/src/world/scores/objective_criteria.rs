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
