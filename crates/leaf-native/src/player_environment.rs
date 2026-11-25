use leaf_core::random::{LegacyRandomSource, RandomSource};

pub const MAX_AIR: i32 = 300;
pub const DROWNING_THRESHOLD: i32 = -20;
pub const LAVA_FIRE_TICKS: u16 = 15 * 20;

#[derive(Clone, Copy)]
pub struct Rules {
    pub drowning_damage: bool,
    pub fire_damage: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    pub air: i32,
    pub fire_ticks: u16,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            air: MAX_AIR,
            fire_ticks: 0,
        }
    }
}

impl Environment {
    pub fn burning(self) -> bool {
        self.fire_ticks > 0
    }
}

pub struct Hazards {
    environment: Environment,
    random: LegacyRandomSource,
}

pub enum Hazard {
    Drowning,
    Lava,
    Burning,
}

impl Hazard {
    pub fn damage(&self) -> f32 {
        match self {
            Self::Drowning => 2.0,
            Self::Lava => 4.0,
            Self::Burning => 1.0,
        }
    }

    pub fn death(&self) -> &'static str {
        match self {
            Self::Drowning => "drowned",
            Self::Lava => "tried to swim in lava",
            Self::Burning => "burned to death",
        }
    }
}

impl Hazards {
    pub fn new(seed: i64, environment: Environment) -> Self {
        Self {
            environment,
            random: LegacyRandomSource::new(seed),
        }
    }

    pub fn environment(&self) -> Environment {
        self.environment
    }

    pub fn clear(&mut self) {
        self.environment = Environment::default();
    }

    pub fn tick(
        &mut self,
        underwater: bool,
        in_water: bool,
        in_lava: bool,
        respiration: i32,
        rules: Rules,
    ) -> Option<Hazard> {
        let drowning = if underwater {
            if respiration <= 0 || self.random.next_double() < 1.0 / (f64::from(respiration) + 1.0)
            {
                self.environment.air -= 1;
            }
            if self.environment.air <= DROWNING_THRESHOLD {
                self.environment.air = 0;
                rules.drowning_damage
            } else {
                false
            }
        } else {
            self.environment.air = (self.environment.air + 4).min(MAX_AIR);
            false
        };

        if in_lava {
            self.environment.fire_ticks = LAVA_FIRE_TICKS;
            return rules.fire_damage.then_some(Hazard::Lava);
        }
        if in_water {
            self.environment.fire_ticks = 0;
        }
        let burning = self.environment.fire_ticks > 0 && self.environment.fire_ticks % 20 == 0;
        self.environment.fire_ticks = self.environment.fire_ticks.saturating_sub(1);
        if drowning {
            Some(Hazard::Drowning)
        } else if burning && rules.fire_damage {
            Some(Hazard::Burning)
        } else {
            None
        }
    }
}
