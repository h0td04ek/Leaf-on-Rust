//! Перенос `java.time.MonthDay` (только нужная часть для SpecialDates).

/// `MonthDay` — месяц (1..12) + день месяца (1..31).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MonthDay {
    pub month: u32,
    pub day: u32,
}

impl MonthDay {
    pub const fn of(month: u32, day: u32) -> Self {
        Self { month, day }
    }

    /// `MonthDay.from(LocalDate)`.
    pub fn from_ymd(year_month_day: (i32, u32, u32)) -> Self {
        let _ = year_month_day.0;
        Self { month: year_month_day.1, day: year_month_day.2 }
    }
}
