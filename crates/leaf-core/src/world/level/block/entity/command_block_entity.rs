//! Перенос `net.minecraft.world.level.block.entity.CommandBlockEntity.Mode`
//! (полный BE — с портом level/block-entities).

/// `CommandBlockEntity.Mode`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommandBlockMode {
    /// `SEQUENCE`.
    SEQUENCE,
    /// `AUTO`.
    AUTO,
    /// `REDSTONE`.
    REDSTONE,
}

impl CommandBlockMode {
    /// `values()`.
    pub const VALUES: [CommandBlockMode; 3] = [
        CommandBlockMode::SEQUENCE,
        CommandBlockMode::AUTO,
        CommandBlockMode::REDSTONE,
    ];

    /// `byOrdinal` — `readEnum`/`writeEnum` (VarInt ordinal).
    pub fn by_ordinal(ordinal: i32) -> CommandBlockMode {
        Self::VALUES[ordinal.rem_euclid(3) as usize]
    }
}

/// Маркер `CommandBlockEntity` — `Mode` вложен в BE.
#[derive(Debug)]
pub struct CommandBlockEntity;
