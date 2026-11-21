//! Перенос `net.minecraft.world.TickRateManager` — управление
//! частотой тиков (freeze/step), используется `Level` и
//! `ServerLevel`. Состояние за `Mutex` — Java-поля мутабельны
//! при разделяемом доступе к уровню.

use crate::world::entity::entity::Entity;
use crate::util::time_util::{NANOSECONDS_PER_MILLISECOND, NANOSECONDS_PER_SECOND};
use std::sync::Mutex;

/// `TickRateManager` — `tickrate`, `nanosecondsPerTick`,
/// `frozenTicksToRun`, `runGameElements`, `isFrozen`.
#[derive(Debug)]
pub struct TickRateManager {
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    /// `tickrate` (default 20.0).
    tickrate: f32,
    /// `nanosecondsPerTick`.
    nanoseconds_per_tick: i64,
    /// `frozenTicksToRun`.
    frozen_ticks_to_run: i32,
    /// `runGameElements`.
    run_game_elements: bool,
    /// `isFrozen`.
    is_frozen: bool,
}

impl Default for TickRateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TickRateManager {
    /// `MIN_TICKRATE`.
    pub const MIN_TICKRATE: f32 = 1.0;

    /// `TickRateManager()` — конструктор по умолчанию.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                tickrate: 20.0,
                nanoseconds_per_tick: NANOSECONDS_PER_SECOND / 20,
                frozen_ticks_to_run: 0,
                run_game_elements: true,
                is_frozen: false,
            }),
        }
    }

    /// `setTickRate`.
    pub fn set_tick_rate(&self, tick_rate: f32) {
        let mut g = self.inner.lock().unwrap();
        g.tickrate = tick_rate.max(Self::MIN_TICKRATE);
        g.nanoseconds_per_tick = (NANOSECONDS_PER_SECOND as f64 / g.tickrate as f64) as i64;
    }

    /// `tickrate()`.
    pub fn tickrate(&self) -> f32 {
        self.inner.lock().unwrap().tickrate
    }

    /// `millisecondsPerTick()`.
    pub fn milliseconds_per_tick(&self) -> f32 {
        let g = self.inner.lock().unwrap();
        g.nanoseconds_per_tick as f32 / NANOSECONDS_PER_MILLISECOND as f32
    }

    /// `nanosecondsPerTick()`.
    pub fn nanoseconds_per_tick(&self) -> i64 {
        self.inner.lock().unwrap().nanoseconds_per_tick
    }

    /// `runsNormally()`.
    pub fn runs_normally(&self) -> bool {
        self.inner.lock().unwrap().run_game_elements
    }

    /// `isSteppingForward()`.
    pub fn is_stepping_forward(&self) -> bool {
        self.inner.lock().unwrap().frozen_ticks_to_run > 0
    }

    /// `setFrozenTicksToRun`.
    pub fn set_frozen_ticks_to_run(&self, frozen_ticks_to_run: i32) {
        self.inner.lock().unwrap().frozen_ticks_to_run = frozen_ticks_to_run;
    }

    /// `frozenTicksToRun()`.
    pub fn frozen_ticks_to_run(&self) -> i32 {
        self.inner.lock().unwrap().frozen_ticks_to_run
    }

    /// `setFrozen`.
    pub fn set_frozen(&self, frozen: bool) {
        self.inner.lock().unwrap().is_frozen = frozen;
    }

    /// `isFrozen()`.
    pub fn is_frozen(&self) -> bool {
        self.inner.lock().unwrap().is_frozen
    }

    /// `tick()` — вызывается раз в игровой тик.
    pub fn tick(&self) {
        let mut g = self.inner.lock().unwrap();
        g.run_game_elements = !g.is_frozen || g.frozen_ticks_to_run > 0;
        if g.frozen_ticks_to_run > 0 {
            g.frozen_ticks_to_run -= 1;
        }
    }

    /// `isEntityFrozen` — сущность заморожена, если игра на
    /// паузе (`!runsNormally`), она не `Player` и у неё нет
    /// игроков-пассажиров.
    pub fn is_entity_frozen(&self, entity: &Entity) -> bool {
        !self.runs_normally()
            && !entity.is_player()
            && entity.count_player_passengers() <= 0
    }
}
