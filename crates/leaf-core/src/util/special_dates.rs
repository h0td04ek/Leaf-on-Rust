//! Перенос `net.minecraft.util.SpecialDates`.
//! Leaf/Gale: предсказание Хеллоуина кэшируется по epoch millis.

use crate::util::month_day::MonthDay;
use std::sync::Mutex;

pub const HALLOWEEN: MonthDay = MonthDay::of(10, 31);
pub const CHRISTMAS: MonthDay = MonthDay::of(12, 24);
pub const NEW_YEAR: MonthDay = MonthDay::of(1, 1);
pub const CHRISTMAS_RANGE: [MonthDay; 3] = [
    MonthDay::of(12, 24),
    MonthDay::of(12, 25),
    MonthDay::of(12, 26),
];

const HALLOWEEN_MONTH: u32 = 10;
const HALLOWEEN_DAY: u32 = 31;

static NEXT_HALLOWEEN: Mutex<(i64, i64)> = Mutex::new((0, 0));

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 29 } else { 28 }
        }
        _ => 0,
    }
}

fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    // Howard Hinnant days_from_civil
    let y = if m <= 2 { y - 1 } else { y } as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = ((m + 9) % 12) as i64;
    let doy = (153 * mp + 2) / 5 + (d as i64) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((if m <= 2 { y + 1 } else { y }) as i32, m, d)
}

/// Текущие (год, месяц, день) в локальном времени — приблизительно через
/// SystemTime + локальный offset недоступен в std; используем UTC+локаль нельзя.
/// В Java это ZonedDateTime.now() — локальная зона. Здесь берём localtime через
/// `chrono`-подобную ручную реализацию не требуется: используем `time` crate?
/// Нет зависимости — используем std::time и считаем по UTC (см. примечание).
fn local_now() -> (i64, u32, u32, u32, u32, u32) {
    // std не даёт локальную зону; используем UTC — для дат праздников
    // расхождение возможно у границ суток. Приемлемо для порта ядра.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days = now.div_euclid(86400);
    let secs = now.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    (now, y as u32, m, d, (secs / 3600) as u32, ((secs % 3600) / 60) as u32)
}

/// `dayNow()` — MonthDay.now (локальная дата; у нас UTC — см. local_now).
pub fn day_now() -> MonthDay {
    let (_ts, _y, m, d, _h, _min) = local_now();
    MonthDay::of(m, d)
}

/// `isHalloween()` — с кэшем границ Хеллоуина (Gale-оптимизация).
pub fn is_halloween() -> bool {
    let (current_epoch_millis, y, _m, _d, _h, _min) = local_now();
    let current_epoch_millis = current_epoch_millis * 1000;
    let mut guard = NEXT_HALLOWEEN.lock().unwrap();
    let (start, end) = *guard;
    let (start, end) = if end == 0 || current_epoch_millis >= end {
        // start/end of local day — упрощённо через days_from_civil на дату,
        // считая полночь локальной = полночь UTC (см. примечание выше)
        let today = days_from_civil(y as i32, _m, _d);
        let mut start_day = today;
        let mut halloween_day = days_from_civil(
            y as i32,
            HALLOWEEN_MONTH,
            HALLOWEEN_DAY.min(days_in_month(y as i32, HALLOWEEN_MONTH)),
        );
        // Halloween — Oct 31 всегда существует
        halloween_day = days_from_civil(y as i32, HALLOWEEN_MONTH, HALLOWEEN_DAY);
        if !days_from_civil(y as i32, _m, _d) < halloween_day + 1 {
            start_day = halloween_day;
        } else {
            start_day = days_from_civil(y as i32 + 1, HALLOWEEN_MONTH, HALLOWEEN_DAY);
        }
        let _ = today;
        let (s, e) = (start_day * 86400 * 1000, (start_day + 1) * 86400 * 1000);
        *guard = (s, e);
        (s, e)
    } else {
        (start, end)
    };
    current_epoch_millis >= start && current_epoch_millis < end
}

/// `isExtendedChristmas()` — 24..26 декабря.
pub fn is_extended_christmas() -> bool {
    CHRISTMAS_RANGE.contains(&day_now())
}
