//! `net.minecraft.world.level.gamerules.GameRuleType`.

/// Тип значения game rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameRuleType {
    /// `INT("integer")`.
    Int,
    /// `BOOL("boolean")`.
    Bool,
}

impl GameRuleType {
    /// `getSerializedName()`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            GameRuleType::Int => "integer",
            GameRuleType::Bool => "boolean",
        }
    }
}
