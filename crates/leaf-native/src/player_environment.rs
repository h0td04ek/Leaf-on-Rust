use leaf_core::random::{LegacyRandomSource, RandomSource};

const MAX_AIR: i32 = 300;
const DROWNING_THRESHOLD: i32 = -20;
const LAVA_FIRE_TICKS: u16 = 15 * 20;

#[derive(Clone, Copy)]
pub struct Rules {
    pub drowning_damage: bool,
    pub fire_damage: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    pub air: i32,
    pub burning: bool,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            air: MAX_AIR,
            burning: false,
        }
    }
}

pub struct Hazards {
    air: i32,
    fire_ticks: u16,
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
    pub fn new(seed: i64) -> Self {
        Self {
            air: MAX_AIR,
            fire_ticks: 0,
            random: LegacyRandomSource::new(seed),
        }
    }

    pub fn environment(&self) -> Environment {
        Environment {
            air: self.air,
            burning: self.fire_ticks > 0,
        }
    }

    pub fn clear(&mut self) {
        self.air = MAX_AIR;
        self.fire_ticks = 0;
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
                self.air -= 1;
            }
            if self.air <= DROWNING_THRESHOLD {
                self.air = 0;
                rules.drowning_damage
            } else {
                false
            }
        } else {
            self.air = (self.air + 4).min(MAX_AIR);
            false
        };

        if in_lava {
            self.fire_ticks = LAVA_FIRE_TICKS;
            return rules.fire_damage.then_some(Hazard::Lava);
        }
        if in_water {
            self.fire_ticks = 0;
        }
        let burning = self.fire_ticks > 0 && self.fire_ticks % 20 == 0;
        self.fire_ticks = self.fire_ticks.saturating_sub(1);
        if drowning {
            Some(Hazard::Drowning)
        } else if burning && rules.fire_damage {
            Some(Hazard::Burning)
        } else {
            None
        }
    }
}
