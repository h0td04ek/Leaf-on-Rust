//! `net.minecraft.world.level.lighting` — движок освещения.

pub mod light_engine;
pub mod level_light_engine;

pub use light_engine::LightEngine;
pub use level_light_engine::LevelLightEngine;
