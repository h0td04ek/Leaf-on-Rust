//! Перенос `net.minecraft.world.entity.ai.sensing.SensorType` —
//! реестр типов сенсоров + фабрика `create()`.

use crate::resources::identifier::Identifier;
use crate::world::entity::ai::sensing::sensor::Sensor;
use std::sync::Arc;

/// `SensorType<U extends Sensor>` — registry-запись с фабрикой.
/// В Rust `U` стёрт до `Box<dyn Sensor>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SensorType {
    /// Registry-ключ (`minecraft:nearest_items` и т.д.).
    pub id: Identifier,
}

impl SensorType {
    fn new(name: &'static str) -> Self {
        Self { id: Identifier::minecraft(name) }
    }
}

/// Все vanilla `SensorType` (порядок = Java-поля).
pub mod keys {
    use super::SensorType;

    macro_rules! st {
        ($name:ident, $id:literal) => {
            /// `SensorType::$name`.
            pub fn $name() -> SensorType {
                SensorType::new($id)
            }
        };
    }

    st!(dummy, "dummy");
    st!(nearest_items, "nearest_items");
    st!(nearest_living_entities, "nearest_living_entities");
    st!(nearest_players, "nearest_players");
    st!(nearest_bed, "nearest_bed");
    st!(hurt_by, "hurt_by");
    st!(villager_hostiles, "villager_hostiles");
    st!(villager_babies, "villager_babies");
    st!(secondary_pois, "secondary_pois");
    st!(golem_detected, "golem_detected");
    st!(armadillo_scare_detected, "armadillo_scare_detected");
    st!(piglin_specific_sensor, "piglin_specific_sensor");
    st!(piglin_brute_specific_sensor, "piglin_brute_specific_sensor");
    st!(hoglin_specific_sensor, "hoglin_specific_sensor");
    st!(nearest_adult, "nearest_adult");
    st!(nearest_adult_any_type, "nearest_adult_any_type");
    st!(axolotl_attackables, "axolotl_attackables");
    st!(food_temptations, "food_temptations");
    st!(axolotl_temptations, "axolotl_temptations");
    st!(goat_temptations, "goat_temptations");
    st!(camel_temptations, "camel_temptations");
    st!(armadillo_temptations, "armadillo_temptations");
    st!(sniffer_temptations, "sniffer_temptations");
    st!(happy_ghast_temptations, "happy_ghast_temptations");
    st!(frog_temptations, "frog_temptations");
    st!(nautilus_temptations, "nautilus_temptations");
    st!(frog_attackables, "frog_attackables");
    st!(is_in_water, "is_in_water");
    st!(warden_entity_sensor, "warden_entity_sensor");
    st!(breeze_attack_entity_sensor, "breeze_attack_entity_sensor");

    /// Полный список (`SensorType` реестр для Brain.provider).
    pub fn all() -> Vec<SensorType> {
        vec![
            dummy(), nearest_items(), nearest_living_entities(), nearest_players(),
            nearest_bed(), hurt_by(), villager_hostiles(), villager_babies(),
            secondary_pois(), golem_detected(), armadillo_scare_detected(),
            piglin_specific_sensor(), piglin_brute_specific_sensor(),
            hoglin_specific_sensor(), nearest_adult(), nearest_adult_any_type(),
            axolotl_attackables(), food_temptations(), axolotl_temptations(),
            goat_temptations(), camel_temptations(), armadillo_temptations(),
            sniffer_temptations(), happy_ghast_temptations(), frog_temptations(),
            nautilus_temptations(), frog_attackables(), is_in_water(),
            warden_entity_sensor(), breeze_attack_entity_sensor(),
        ]
    }
}

use keys as k;

/// `SensorType.create()` — фабрика по типу. Возвращает None для
/// неизвестных типов.
pub fn create_sensor(ty: SensorType) -> Option<Box<dyn Sensor>> {
    use crate::world::entity::ai::sensing::sensors;
    Some(match ty.id.path.as_str() {
        "dummy" => Box::new(sensors::DummySensor::new()),
        "nearest_items" => Box::new(sensors::NearestItemSensor::new()),
        "nearest_living_entities" => Box::new(sensors::NearestLivingEntitySensor::new()),
        "nearest_players" => Box::new(sensors::PlayerSensor::new()),
        "nearest_bed" => Box::new(sensors::NearestBedSensor::new()),
        "hurt_by" => Box::new(sensors::HurtBySensor::new()),
        "villager_hostiles" => Box::new(sensors::VillagerHostilesSensor::new()),
        "villager_babies" => Box::new(sensors::VillagerBabiesSensor::new()),
        "secondary_pois" => Box::new(sensors::SecondaryPoiSensor::new()),
        "golem_detected" => Box::new(sensors::GolemSensor::new()),
        "armadillo_scare_detected" => Box::new(sensors::MobSensor::armadillo_scare()),
        "piglin_specific_sensor" => Box::new(sensors::PiglinSpecificSensor::new()),
        "piglin_brute_specific_sensor" => Box::new(sensors::PiglinBruteSpecificSensor::new()),
        "hoglin_specific_sensor" => Box::new(sensors::HoglinSpecificSensor::new()),
        "nearest_adult" => Box::new(sensors::AdultSensor::new()),
        "nearest_adult_any_type" => Box::new(sensors::AdultSensorAnyType::new()),
        "axolotl_attackables" => Box::new(sensors::AxolotlAttackablesSensor::new()),
        "food_temptations" => Box::new(sensors::TemptingSensor::for_animal()),
        "axolotl_temptations" => Box::new(sensors::TemptingSensor::axolotl()),
        "goat_temptations" => Box::new(sensors::TemptingSensor::goat()),
        "camel_temptations" => Box::new(sensors::TemptingSensor::camel()),
        "armadillo_temptations" => Box::new(sensors::TemptingSensor::armadillo()),
        "sniffer_temptations" => Box::new(sensors::TemptingSensor::sniffer()),
        "happy_ghast_temptations" => Box::new(sensors::TemptingSensor::happy_ghast()),
        "frog_temptations" => Box::new(sensors::TemptingSensor::frog()),
        "nautilus_temptations" => Box::new(sensors::TemptingSensor::nautilus()),
        "frog_attackables" => Box::new(sensors::FrogAttackablesSensor::new()),
        "is_in_water" => Box::new(sensors::IsInWaterSensor::new()),
        "warden_entity_sensor" => Box::new(sensors::WardenEntitySensor::new()),
        "breeze_attack_entity_sensor" => Box::new(sensors::BreezeAttackEntitySensor::new()),
        _ => return None,
    })
}
