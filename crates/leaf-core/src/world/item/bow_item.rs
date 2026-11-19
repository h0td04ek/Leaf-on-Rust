//! Перенос статических методов `net.minecraft.world.item.BowItem`.
//! Само поведение предмета (`use`/`releaseUsing`) живёт в
//! `Item`-диспетчере; здесь — чистые вычисления.

/// `BowItem.getPowerForTime(charge)` — сила выстрела по тикам
/// удержания: `f = charge/20; f = (f*f + 2f)/3; clamp(f, 0, 1)`.
pub fn get_power_for_time(charge: i32) -> f32 {
    let mut f = charge as f32 / 20.0;
    f = (f * f + f * 2.0) / 3.0;
    f.clamp(0.0, 1.0)
}
