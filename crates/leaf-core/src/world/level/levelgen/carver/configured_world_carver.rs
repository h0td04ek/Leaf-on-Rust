//! `net.minecraft.world.level.levelgen.carver.ConfiguredWorldCarver` —
//! карвер + конфиг.

use super::world_carver::WorldCarver;
use std::sync::Arc;

/// `ConfiguredWorldCarver<C>` — типизированный конфиг стёрт (дженерик).
#[derive(Clone)]
pub struct ConfiguredWorldCarver {
    /// `carver`.
    pub carver: Arc<dyn WorldCarver>,
    /// `config`.
    pub config: Arc<dyn std::any::Any + Send + Sync>,
}

impl ConfiguredWorldCarver {
    /// `new ConfiguredWorldCarver(carver, config)`.
    pub fn new(carver: Arc<dyn WorldCarver>, config: Arc<dyn std::any::Any + Send + Sync>) -> Self {
        Self { carver, config }
    }
}
