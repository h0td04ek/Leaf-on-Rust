use std::time::{Duration, Instant};

use leaf_core::world::tick_rate_manager::TickRateManager;

pub const MAX_RATE: f32 = 10_000.0;
pub const ADMIN_PERMISSION: u8 = 3;
const CONTROL_POLL: Duration = Duration::from_millis(5);

#[derive(Clone, Copy)]
pub enum Action {
    Query,
    Rate(f32),
    Freeze(bool),
    Step(i32),
    StopStep,
    Sprint(i32),
    StopSprint,
}

#[derive(Clone, Copy, PartialEq)]
pub struct TickState {
    pub rate: f32,
    pub frozen: bool,
    pub steps: i32,
}

pub struct Clock {
    manager: TickRateManager,
    next_tick: Instant,
    game_time: i64,
    sprint: Option<Sprint>,
    average_ms: f64,
}

struct Sprint {
    remaining: i32,
    requested: i32,
    was_frozen: bool,
    started: Instant,
}

impl Clock {
    pub fn new(now: Instant) -> Self {
        Self {
            manager: TickRateManager::new(),
            next_tick: now + Duration::from_millis(50),
            game_time: 0,
            sprint: None,
            average_ms: 0.0,
        }
    }

    pub fn time(&self) -> i64 {
        self.game_time
    }

    pub fn state(&self) -> TickState {
        TickState {
            rate: self.manager.tickrate(),
            frozen: self.manager.is_frozen(),
            steps: self.manager.frozen_ticks_to_run(),
        }
    }

    fn period(&self) -> Duration {
        Duration::from_nanos(self.manager.nanoseconds_per_tick() as u64)
    }

    pub fn poll(&mut self, now: Instant) -> Result<(i64, bool), Duration> {
        if let Some(sprint) = &mut self.sprint {
            sprint.remaining -= 1;
        } else {
            if self.manager.is_frozen() && !self.manager.is_stepping_forward() {
                return Err(CONTROL_POLL);
            }
            if now < self.next_tick {
                return Err((self.next_tick - now).min(CONTROL_POLL));
            }
            self.next_tick = (self.next_tick + self.period()).max(now);
        }
        let finished_step = self.manager.is_frozen() && self.manager.frozen_ticks_to_run() == 1;
        self.manager.tick();
        self.game_time += 1;
        Ok((self.game_time, finished_step))
    }

    pub fn complete(&mut self, elapsed: Duration, now: Instant) -> Option<String> {
        let milliseconds = elapsed.as_secs_f64() * 1000.0;
        self.average_ms += (milliseconds - self.average_ms) * 0.05;
        if self
            .sprint
            .as_ref()
            .is_some_and(|sprint| sprint.remaining == 0)
        {
            self.stop_sprint(now)
        } else {
            None
        }
    }

    pub fn execute(&mut self, action: Action, now: Instant) -> (String, i32) {
        match action {
            Action::Query => {
                let state = match &self.sprint {
                    Some(sprint) => format!("sprinting ({} ticks left)", sprint.remaining),
                    None if self.manager.is_stepping_forward() => format!(
                        "stepping ({} ticks left)",
                        self.manager.frozen_ticks_to_run()
                    ),
                    None if self.manager.is_frozen() => "frozen".into(),
                    None if self.average_ms > self.period().as_secs_f64() * 1000.0 => {
                        "lagging".into()
                    }
                    None => "running".into(),
                };
                let rate = self.manager.tickrate();
                (
                    format!(
                        "Tick rate: {rate:.1} TPS, {state}, {:.2} ms/tick",
                        self.average_ms
                    ),
                    rate as i32,
                )
            }
            Action::Rate(rate) => {
                self.manager.set_tick_rate(rate);
                self.next_tick = now + self.period();
                (format!("Set tick rate to {rate:.1} TPS"), rate as i32)
            }
            Action::Freeze(frozen) => {
                self.stop_sprint(now);
                self.manager.set_frozen_ticks_to_run(0);
                self.manager.set_frozen(frozen);
                self.next_tick = now + self.period();
                (
                    if frozen {
                        "The world is frozen"
                    } else {
                        "The world is running"
                    }
                    .into(),
                    i32::from(frozen),
                )
            }
            Action::Step(ticks) if self.manager.is_frozen() => {
                self.manager.set_frozen_ticks_to_run(ticks);
                self.next_tick = now + self.period();
                (format!("Stepping the world by {ticks} tick(s)"), 1)
            }
            Action::Step(_) => ("Freeze the world before stepping".into(), 0),
            Action::StopStep if self.manager.is_stepping_forward() => {
                self.manager.set_frozen_ticks_to_run(0);
                ("Stopped stepping the world".into(), 1)
            }
            Action::StopStep => ("The world is not stepping".into(), 0),
            Action::Sprint(ticks) => {
                self.stop_sprint(now);
                self.manager.set_frozen_ticks_to_run(0);
                self.sprint = Some(Sprint {
                    remaining: ticks,
                    requested: ticks,
                    was_frozen: self.manager.is_frozen(),
                    started: now,
                });
                self.manager.set_frozen(false);
                (format!("Sprinting for {ticks} tick(s)"), 1)
            }
            Action::StopSprint => self.stop_sprint(now).map_or_else(
                || ("The world is not sprinting".into(), 0),
                |summary| (summary, 1),
            ),
        }
    }

    fn stop_sprint(&mut self, now: Instant) -> Option<String> {
        let sprint = self.sprint.take()?;
        self.manager.set_frozen(sprint.was_frozen);
        self.next_tick = now + self.period();
        let ticks = sprint.requested - sprint.remaining;
        let elapsed = now.saturating_duration_since(sprint.started).as_secs_f64();
        Some(format!("World sprint ran {ticks} tick(s) in {elapsed:.3}s"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_honors_rate_freeze_steps_and_sprint() {
        let start = Instant::now();
        let ms = |value| start + Duration::from_millis(value);
        let mut clock = Clock::new(start);
        assert!(clock.poll(ms(49)).is_err());
        assert_eq!(clock.poll(ms(50)), Ok((1, false)));

        clock.execute(Action::Rate(40.0), ms(50));
        assert!(clock.poll(ms(74)).is_err());
        assert_eq!(clock.poll(ms(75)), Ok((2, false)));
        clock.execute(Action::Freeze(true), ms(75));
        assert!(clock.poll(ms(200)).is_err());

        clock.execute(Action::Step(2), ms(200));
        assert_eq!(clock.state().steps, 2);
        assert_eq!(clock.poll(ms(225)), Ok((3, false)));
        assert_eq!(clock.poll(ms(250)), Ok((4, true)));
        assert!(clock.poll(ms(300)).is_err());

        clock.execute(Action::Step(5), ms(300));
        assert_eq!(clock.execute(Action::StopStep, ms(301)).1, 1);
        assert!(clock.poll(ms(500)).is_err());

        clock.execute(Action::Sprint(3), ms(500));
        assert!(!clock.state().frozen);
        for (index, time) in [5, 6, 7].into_iter().enumerate() {
            assert_eq!(clock.poll(ms(500)), Ok((time, false)));
            assert_eq!(
                clock.complete(Duration::ZERO, ms(500)).is_some(),
                index == 2
            );
        }
        assert!(clock.state().frozen);
        assert_eq!(clock.time(), 7);

        clock.execute(Action::Sprint(4), ms(501));
        clock.poll(ms(501)).unwrap();
        assert_eq!(clock.execute(Action::StopSprint, ms(501)).1, 1);
        assert!(clock.state().frozen);
        clock.execute(Action::Freeze(false), ms(502));
        assert_eq!(clock.poll(ms(527)), Ok((9, false)));
    }
}
