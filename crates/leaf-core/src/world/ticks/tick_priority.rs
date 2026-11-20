//! `net.minecraft.world.ticks.TickPriority` — приоритет запланированного тика.

/// `TickPriority` — значения от -3 (extremely high) до 3 (extremely low).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TickPriority {
    /// `EXTREMELY_HIGH(-3)`.
    ExtremelyHigh = -3,
    /// `VERY_HIGH(-2)`.
    VeryHigh = -2,
    /// `HIGH(-1)`.
    High = -1,
    /// `NORMAL(0)` — по умолчанию.
    Normal = 0,
    /// `LOW(1)`.
    Low = 1,
    /// `VERY_LOW(2)`.
    VeryLow = 2,
    /// `EXTREMELY_LOW(3)`.
    ExtremelyLow = 3,
}

impl TickPriority {
    /// `values()`.
    pub const VALUES: [TickPriority; 7] = [
        TickPriority::ExtremelyHigh,
        TickPriority::VeryHigh,
        TickPriority::High,
        TickPriority::Normal,
        TickPriority::Low,
        TickPriority::VeryLow,
        TickPriority::ExtremelyLow,
    ];

    /// `byValue(priority)` — кламп к ближайшему допустимому.
    pub fn by_value(priority: i32) -> Self {
        for p in Self::VALUES {
            if p.value() == priority {
                return p;
            }
        }
        if priority < TickPriority::ExtremelyHigh.value() {
            TickPriority::ExtremelyHigh
        } else {
            TickPriority::ExtremelyLow
        }
    }

    /// `getValue()`.
    pub fn value(self) -> i32 {
        self as i32
    }

    /// `CODEC` — int-маппинг (`Codec.INT.xmap`).
    /// Чтение из NBT int.
    pub fn read_int(value: i32) -> Self {
        Self::by_value(value)
    }

    /// Запись в NBT int.
    pub fn write_int(self) -> i32 {
        self.value()
    }
}
