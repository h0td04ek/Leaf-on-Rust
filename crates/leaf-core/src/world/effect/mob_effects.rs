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

// ---------------------------------------------------------------------------
// `MobEffects` bootstrap — полные ванильные определения всех 40 эффектов
// (категория/цвет/атрибутные модификаторы/kind/звук/blend) из
// `MobEffects.java` 1.21.11.
// ---------------------------------------------------------------------------

use crate::entity::ai::attributes::attribute_modifier::AttributeModifierOperation as Op;
use crate::entity::ai::attributes::attributes as attrs;
use crate::world::effect::mob_effect::{MobEffectCategory as Cat, MobEffectKind as Kind};

fn eff(
    path: &str,
    cat: Cat,
    color: i32,
    kind: Kind,
) -> (ResourceKey<MobEffect>, MobEffect) {
    (
        key(path),
        MobEffect::with_category_color(Identifier::minecraft(path), cat, color).with_kind(kind),
    )
}

/// `MobEffects` bootstrap — `(ResourceKey, MobEffect)` для всех
/// ванильных эффектов в порядке реестра.
pub fn bootstrap() -> Vec<(ResourceKey<MobEffect>, MobEffect)> {
    use Cat::*;
    let attr_speed = |id: &str| Identifier::with_default_namespace(id).unwrap();
    let mut v: Vec<(ResourceKey<MobEffect>, MobEffect)> = Vec::new();

    // speed — MOVEMENT_SPEED +0.2 ADD_MULTIPLIED_TOTAL
    let (k, mut e) = eff("speed", Beneficial, 3402751, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::movement_speed(), attr_speed("effect.speed"), 0.2, Op::AddMultipliedTotal);
    v.push((k, e));

    // slowness — MOVEMENT_SPEED -0.15 ADD_MULTIPLIED_TOTAL
    let (k, mut e) = eff("slowness", Harmful, 9154528, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::movement_speed(), attr_speed("effect.slowness"), -0.15, Op::AddMultipliedTotal);
    v.push((k, e));

    // haste — ATTACK_SPEED +0.1 ADD_MULTIPLIED_TOTAL
    let (k, mut e) = eff("haste", Beneficial, 14270531, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::attack_speed(), attr_speed("effect.haste"), 0.1, Op::AddMultipliedTotal);
    v.push((k, e));

    // mining_fatigue — ATTACK_SPEED -0.1 ADD_MULTIPLIED_TOTAL
    let (k, mut e) = eff("mining_fatigue", Harmful, 4866583, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::attack_speed(), attr_speed("effect.mining_fatigue"), -0.1, Op::AddMultipliedTotal);
    v.push((k, e));

    // strength — ATTACK_DAMAGE +3.0 ADD_VALUE
    let (k, mut e) = eff("strength", Beneficial, 16762624, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::attack_damage(), attr_speed("effect.strength"), 3.0, Op::AddValue);
    v.push((k, e));

    v.push(eff("instant_health", Beneficial, 16262179, Kind::HealOrHarm { is_harm: false }));
    v.push(eff("instant_damage", Harmful, 11101546, Kind::HealOrHarm { is_harm: true }));

    // jump_boost — SAFE_FALL_DISTANCE +1.0 ADD_VALUE
    let (k, mut e) = eff("jump_boost", Beneficial, 16646020, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::safe_fall_distance(), attr_speed("effect.jump_boost"), 1.0, Op::AddValue);
    v.push((k, e));

    // nausea — blend(150, 20, 60)
    let (k, e) = eff("nausea", Harmful, 5578058, Kind::AttributeOnly);
    v.push((k, e.set_blend_duration(150, 20, 60)));

    v.push(eff("regeneration", Beneficial, 13458603, Kind::Regeneration));
    v.push(eff("resistance", Beneficial, 9520880, Kind::AttributeOnly));
    v.push(eff("fire_resistance", Beneficial, 16750848, Kind::AttributeOnly));
    v.push(eff("water_breathing", Beneficial, 10017472, Kind::AttributeOnly));

    // invisibility — WAYPOINT_TRANSMIT_RANGE -1.0 ADD_MULTIPLIED_TOTAL
    let (k, mut e) = eff("invisibility", Beneficial, 16185078, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::waypoint_transmit_range(), attr_speed("effect.waypoint_transmit_range_hide"), -1.0, Op::AddMultipliedTotal);
    v.push((k, e));

    v.push(eff("blindness", Harmful, 2039587, Kind::AttributeOnly));
    v.push(eff("night_vision", Beneficial, 12779366, Kind::AttributeOnly));
    v.push(eff("hunger", Harmful, 5797459, Kind::Hunger));

    // weakness — ATTACK_DAMAGE -4.0 ADD_VALUE
    let (k, mut e) = eff("weakness", Harmful, 4738376, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::attack_damage(), attr_speed("effect.weakness"), -4.0, Op::AddValue);
    v.push((k, e));

    v.push(eff("poison", Harmful, 8889187, Kind::Poison));
    v.push(eff("wither", Harmful, 7561558, Kind::Wither));

    // health_boost — MAX_HEALTH +4.0 ADD_VALUE
    let (k, mut e) = eff("health_boost", Beneficial, 16284963, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::max_health(), attr_speed("effect.health_boost"), 4.0, Op::AddValue);
    v.push((k, e));

    // absorption — MAX_ABSORPTION +4.0 ADD_VALUE
    let (k, mut e) = eff("absorption", Beneficial, 2445989, Kind::Absorption);
    e = e.add_attribute_modifier(attrs::max_absorption(), attr_speed("effect.absorption"), 4.0, Op::AddValue);
    v.push((k, e));

    v.push(eff("saturation", Beneficial, 16262179, Kind::Saturation));
    v.push(eff("glowing", Neutral, 9740385, Kind::AttributeOnly));
    v.push(eff("levitation", Harmful, 13565951, Kind::AttributeOnly));

    // luck — LUCK +1.0 ADD_VALUE
    let (k, mut e) = eff("luck", Beneficial, 5882118, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::luck(), attr_speed("effect.luck"), 1.0, Op::AddValue);
    v.push((k, e));

    // unluck — LUCK -1.0 ADD_VALUE
    let (k, mut e) = eff("unluck", Harmful, 12624973, Kind::AttributeOnly);
    e = e.add_attribute_modifier(attrs::luck(), attr_speed("effect.unluck"), -1.0, Op::AddValue);
    v.push((k, e));

    v.push(eff("slow_falling", Beneficial, 15978425, Kind::AttributeOnly));
    v.push(eff("conduit_power", Beneficial, 1950417, Kind::AttributeOnly));
    v.push(eff("dolphins_grace", Beneficial, 8954814, Kind::AttributeOnly));

    // bad_omen — sound APPLY_EFFECT_BAD_OMEN
    let (k, e) = eff("bad_omen", Neutral, 745784, Kind::BadOmen);
    v.push((k, e.with_sound_on_added(crate::sounds::sound_events::apply_effect_bad_omen())));

    v.push(eff("hero_of_the_village", Beneficial, 4521796, Kind::AttributeOnly));

    // darkness — blend(22)
    let (k, e) = eff("darkness", Harmful, 2696993, Kind::AttributeOnly);
    v.push((k, e.set_blend_duration(22, 22, 22)));

    // trial_omen — sound APPLY_EFFECT_TRIAL_OMEN
    let (k, e) = eff("trial_omen", Neutral, 1484454, Kind::AttributeOnly);
    v.push((k, e.with_sound_on_added(crate::sounds::sound_events::apply_effect_trial_omen())));

    // raid_omen — sound APPLY_EFFECT_RAID_OMEN
    let (k, e) = eff("raid_omen", Neutral, 14565464, Kind::RaidOmen);
    v.push((k, e.with_sound_on_added(crate::sounds::sound_events::apply_effect_raid_omen())));

    v.push(eff("wind_charged", Harmful, 12438015, Kind::WindCharged));
    v.push(eff("weaving", Harmful, 7891290, Kind::Weaving { min_max: (2, 3) }));
    v.push(eff("oozing", Harmful, 10092451, Kind::Oozing { count: 2 }));
    v.push(eff("infested", Harmful, 9214860, Kind::Infested { chance: 0.1, min_max: (1, 2) }));
    v.push(eff("breath_of_the_nautilus", Beneficial, 65518, Kind::AttributeOnly));

    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys() {
        assert_eq!(all_keys().len(), 40);
    }
}
