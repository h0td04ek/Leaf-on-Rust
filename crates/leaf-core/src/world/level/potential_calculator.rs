//! Перенос `net.minecraft.world.level.PotentialCalculator` —
//! аккумулятор "потенциальных зарядов" для mob-spawn energy budget.

use crate::core::block_pos::BlockPos;

/// `PotentialCalculator.PointCharge`.
#[derive(Clone, Copy, Debug)]
struct PointCharge {
    /// `pos`.
    pos: BlockPos,
    /// `charge`.
    charge: f64,
}

impl PointCharge {
    /// `getPotentialChange(pos)` — `charge / sqrt(distSqr)`
    /// (POSITIVE_INFINITY при совпадении позиций).
    fn get_potential_change(&self, pos: BlockPos) -> f64 {
        let d = self.pos.dist_sqr(pos);
        if d == 0.0 {
            f64::INFINITY
        } else {
            self.charge / d.sqrt()
        }
    }
}

/// `PotentialCalculator`.
#[derive(Default)]
pub struct PotentialCalculator {
    /// `charges`.
    charges: Vec<PointCharge>,
}

impl PotentialCalculator {
    /// `new PotentialCalculator()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `addCharge(pos, charge)` — charge == 0 пропускается.
    pub fn add_charge(&mut self, pos: BlockPos, charge: f64) {
        if charge != 0.0 {
            self.charges.push(PointCharge { pos, charge });
        }
    }

    /// `getPotentialEnergyChange(pos, charge)` — сумма
    /// `pointCharge.getPotentialChange(pos) * charge`.
    pub fn get_potential_energy_change(&self, pos: BlockPos, charge: f64) -> f64 {
        if charge == 0.0 {
            return 0.0;
        }
        let mut d = 0.0;
        for point_charge in &self.charges {
            d += point_charge.get_potential_change(pos);
        }
        d * charge
    }
}
