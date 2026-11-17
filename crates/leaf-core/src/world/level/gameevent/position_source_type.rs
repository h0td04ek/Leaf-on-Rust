//! Перенос `net.minecraft.world.level.gameevent.PositionSourceType` —
//! тип источника позиции (реестр `Registries.POSITION_SOURCE_TYPE`).

/// `PositionSourceType` — маркер типа `PositionSource`.
/// Провод: VarInt registry-id типа.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PositionSourceTypeKind {
    /// `block`.
    Block,
    /// `entity`.
    Entity,
}

impl PositionSourceTypeKind {
    /// Все типы в порядке регистрации.
    pub const VALUES: [PositionSourceTypeKind; 2] = [
        PositionSourceTypeKind::Block,
        PositionSourceTypeKind::Entity,
    ];

    /// registry-id типа.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// Тип по registry-id.
    pub fn by_id(id: i32) -> PositionSourceTypeKind {
        Self::VALUES.get(id as usize).copied().unwrap_or(PositionSourceTypeKind::Block)
    }
}
