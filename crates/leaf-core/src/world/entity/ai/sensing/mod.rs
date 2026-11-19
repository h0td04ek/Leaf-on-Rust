//! `net.minecraft.world.entity.ai.sensing`.

pub mod sensing;
pub mod sensor;
pub mod sensors;
pub mod sensor_type;

pub use sensing::Sensing;
pub use sensor::{
    entity_type_is, is_entity_attackable, is_entity_attackable_ignoring_los,
    is_entity_targetable, remember_positives, was_entity_attackable_last_n_ticks,
    was_entity_attackable_last_n_ticks as _, Sensor, SensorState,
};

pub use sensors::*;
pub use sensor_type::SensorType;
