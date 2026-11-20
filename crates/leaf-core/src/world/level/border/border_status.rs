//! `net.minecraft.world.level.border.BorderStatus`.

/// Статус границы с цветом для клиентского хинта.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStatus {
    /// `GROWING(4259712)`.
    Growing,
    /// `SHRINKING(16724016)`.
    Shrinking,
    /// `STATIONARY(2138367)`.
    Stationary,
}

impl BorderStatus {
    /// `getColor()`.
    pub fn get_color(&self) -> i32 {
        match self {
            BorderStatus::Growing => 4259712,
            BorderStatus::Shrinking => 16724016,
            BorderStatus::Stationary => 2138367,
        }
    }
}
