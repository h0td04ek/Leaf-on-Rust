//! Перенос `net.minecraft.world.effect.MobEffects` — константы всех
//! ванильных эффектов (registry `minecraft:mob_effect`).

use crate::core::holder::Holder;
use crate::core::registry::registries;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::world::effect::mob_effect::MobEffect;

fn key(path: &str) -> ResourceKey<MobEffect> {
    ResourceKey::create_with_registry_name(registries::mob_effect(), Identifier::minecraft(path))
}

/// Holder по registry-ключу эффекта.
pub fn holder_of(key: ResourceKey<MobEffect>) -> Holder<MobEffect> {
    Holder::Key(key)
}

macro_rules! effect {
    ($name:ident, $path:literal) => {
        /// `MobEffects::$name`.
        pub fn $name() -> Holder<MobEffect> {
            Holder::Key(key($path))
        }
    };
}

effect!(speed, "speed");
effect!(slowness, "slowness");
effect!(haste, "haste");
effect!(mining_fatigue, "mining_fatigue");
effect!(strength, "strength");
effect!(instant_health, "instant_health");
effect!(instant_damage, "instant_damage");
effect!(jump_boost, "jump_boost");
effect!(nausea, "nausea");
effect!(regeneration, "regeneration");
effect!(resistance, "resistance");
effect!(fire_resistance, "fire_resistance");
effect!(water_breathing, "water_breathing");
effect!(invisibility, "invisibility");
effect!(blindness, "blindness");
effect!(night_vision, "night_vision");
effect!(hunger, "hunger");
effect!(weakness, "weakness");
effect!(poison, "poison");
effect!(wither, "wither");
effect!(health_boost, "health_boost");
effect!(absorption, "absorption");
effect!(saturation, "saturation");
effect!(glowing, "glowing");
effect!(levitation, "levitation");
effect!(luck, "luck");
effect!(unluck, "unluck");
effect!(slow_falling, "slow_falling");
effect!(conduit_power, "conduit_power");
effect!(dolphins_grace, "dolphins_grace");
effect!(bad_omen, "bad_omen");
effect!(hero_of_the_village, "hero_of_the_village");
effect!(darkness, "darkness");
effect!(trial_omen, "trial_omen");
effect!(raid_omen, "raid_omen");
effect!(wind_charged, "wind_charged");
effect!(weaving, "weaving");
effect!(oozing, "oozing");
effect!(infested, "infested");
effect!(breath_of_the_nautilus, "breath_of_the_nautilus");

/// Все registry-ключи эффектов в порядке реестра.
pub fn all_keys() -> Vec<ResourceKey<MobEffect>> {
    vec![
        key("speed"), key("slowness"), key("haste"), key("mining_fatigue"),
        key("strength"), key("instant_health"), key("instant_damage"),
        key("jump_boost"), key("nausea"), key("regeneration"), key("resistance"),
        key("fire_resistance"), key("water_breathing"), key("invisibility"),
        key("blindness"), key("night_vision"), key("hunger"), key("weakness"),
        key("poison"), key("wither"), key("health_boost"), key("absorption"),
        key("saturation"), key("glowing"), key("levitation"), key("luck"),
        key("unluck"), key("slow_falling"), key("conduit_power"),
        key("dolphins_grace"), key("bad_omen"), key("hero_of_the_village"),
        key("darkness"), key("trial_omen"), key("raid_omen"),
        key("wind_charged"), key("weaving"), key("oozing"),
        key("infested"), key("breath_of_the_nautilus"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys() {
        assert_eq!(all_keys().len(), 40);
    }
}
