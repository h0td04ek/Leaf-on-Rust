//! `net.minecraft.world.level.levelgen.carver` — карверы пещер.
pub mod configured_world_carver;
pub mod world_carver;
pub use configured_world_carver::ConfiguredWorldCarver;
pub use world_carver::{CarvingContext, WorldCarver};
