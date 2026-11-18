//! Перенос `net.minecraft.advancements.AdvancementType`.

/// `AdvancementType` — тип advancement (TASK/CHALLENGE/GOAL).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdvancementType {
    /// `TASK`.
    Task,
    /// `CHALLENGE`.
    Challenge,
    /// `GOAL`.
    Goal,
}

impl AdvancementType {
    /// `values()`.
    pub const VALUES: [AdvancementType; 3] = [
        AdvancementType::Task,
        AdvancementType::Challenge,
        AdvancementType::Goal,
    ];

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            AdvancementType::Task => "task",
            AdvancementType::Challenge => "challenge",
            AdvancementType::Goal => "goal",
        }
    }

    /// `fromSerializedName`.
    pub fn from_serialized_name(name: &str) -> Option<Self> {
        match name {
            "task" => Some(AdvancementType::Task),
            "challenge" => Some(AdvancementType::Challenge),
            "goal" => Some(AdvancementType::Goal),
            _ => None,
        }
    }
}
