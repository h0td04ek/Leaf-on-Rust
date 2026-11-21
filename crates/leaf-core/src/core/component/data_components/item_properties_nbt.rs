use std::ops::RangeInclusive;

use super::{
    with_nbt, AttackRange, FoodProperties, SwingAnimation, UseCooldown, UseEffects, Weapon,
};
use crate::core::component::DataComponentType;
use crate::nbt::{CompoundTag, Tag};
use crate::resources::Identifier;
use crate::world::item::swing_animation_type::SwingAnimationType;

fn record<T>(
    ty: DataComponentType,
    encode: fn(&T) -> CompoundTag,
    decode: fn(&CompoundTag) -> Result<T, String>,
) -> DataComponentType
where
    T: Clone + Send + Sync + PartialEq + std::fmt::Debug + 'static,
{
    with_nbt(
        ty,
        move |value| {
            let tag = encode(value);
            decode(&tag)?;
            Ok(Tag::Compound(tag))
        },
        move |tag| decode(tag.as_compound().ok_or("expected an NBT compound")?),
    )
}

fn field<T: Copy>(
    tag: &CompoundTag,
    key: &str,
    default: Option<T>,
    read: fn(&Tag) -> Option<T>,
) -> Result<T, String> {
    match tag.get(key) {
        Some(tag) => read(tag).ok_or_else(|| format!("invalid {key}")),
        None => default.ok_or_else(|| format!("missing {key}")),
    }
}

fn bounded<T: PartialOrd>(key: &str, value: T, limits: RangeInclusive<T>) -> Result<T, String> {
    if !limits.contains(&value) {
        return Err(format!("{key} is out of range"));
    }
    Ok(value)
}

fn optional_float(tag: &mut CompoundTag, key: &str, value: f32, default: f32) {
    if value.to_bits() != default.to_bits() {
        tag.put_float(key, value);
    }
}

pub(super) fn attack_range(ty: DataComponentType) -> DataComponentType {
    record(
        ty,
        |range: &AttackRange| {
            let mut tag = CompoundTag::new();
            let default = AttackRange::DEFAULT;
            for (key, value, default) in [
                ("min_reach", range.min_range, default.min_range),
                ("max_reach", range.max_range, default.max_range),
                (
                    "min_creative_reach",
                    range.min_creative_range,
                    default.min_creative_range,
                ),
                (
                    "max_creative_reach",
                    range.max_creative_range,
                    default.max_creative_range,
                ),
                ("hitbox_margin", range.hitbox_margin, default.hitbox_margin),
                ("mob_factor", range.mob_factor, default.mob_factor),
            ] {
                optional_float(&mut tag, key, value, default);
            }
            tag
        },
        |tag| {
            let default = AttackRange::DEFAULT;
            let number = |name, default, limit| {
                bounded(
                    name,
                    field(tag, name, Some(default), Tag::as_float)?,
                    0.0..=limit,
                )
            };
            Ok(AttackRange {
                min_range: number("min_reach", default.min_range, 64.0)?,
                max_range: number("max_reach", default.max_range, 64.0)?,
                min_creative_range: number("min_creative_reach", default.min_creative_range, 64.0)?,
                max_creative_range: number("max_creative_reach", default.max_creative_range, 64.0)?,
                hitbox_margin: number("hitbox_margin", default.hitbox_margin, 1.0)?,
                mob_factor: number("mob_factor", default.mob_factor, 2.0)?,
            })
        },
    )
}

pub(super) fn use_effects(ty: DataComponentType) -> DataComponentType {
    record(
        ty,
        |effects: &UseEffects| {
            let mut tag = CompoundTag::new();
            let default = UseEffects::DEFAULT;
            if effects.can_sprint != default.can_sprint {
                tag.put_bool("can_sprint", effects.can_sprint);
            }
            if effects.interact_vibrations != default.interact_vibrations {
                tag.put_bool("interact_vibrations", effects.interact_vibrations);
            }
            optional_float(
                &mut tag,
                "speed_multiplier",
                effects.speed_multiplier,
                default.speed_multiplier,
            );
            tag
        },
        |tag| {
            let default = UseEffects::DEFAULT;
            let name = "speed_multiplier";
            let speed = field(tag, name, Some(default.speed_multiplier), Tag::as_float)?;
            Ok(UseEffects {
                can_sprint: field(tag, "can_sprint", Some(default.can_sprint), Tag::as_bool)?,
                interact_vibrations: field(
                    tag,
                    "interact_vibrations",
                    Some(default.interact_vibrations),
                    Tag::as_bool,
                )?,
                speed_multiplier: bounded(name, speed, 0.0..=1.0)?,
            })
        },
    )
}

pub(super) fn use_cooldown(ty: DataComponentType) -> DataComponentType {
    record(
        ty,
        |cooldown: &UseCooldown| {
            let mut tag = CompoundTag::new();
            tag.put_float("seconds", cooldown.seconds);
            if let Some(group) = &cooldown.cooldown_group {
                tag.put_string("cooldown_group", &group.to_string());
            }
            tag
        },
        |tag| {
            let seconds = field(tag, "seconds", None, Tag::as_float)?;
            if !(seconds > 0.0 && seconds <= f32::MAX) {
                return Err("cooldown seconds must be positive and finite".into());
            }
            let cooldown_group = tag
                .get("cooldown_group")
                .map(|group| {
                    Identifier::parse(
                        group
                            .as_string()
                            .ok_or("expected a cooldown group identifier")?,
                    )
                    .map_err(|error| error.to_string())
                })
                .transpose()?;
            Ok(UseCooldown {
                seconds,
                cooldown_group,
            })
        },
    )
}

pub(super) fn weapon(ty: DataComponentType) -> DataComponentType {
    record(
        ty,
        |weapon: &Weapon| {
            let mut tag = CompoundTag::new();
            if weapon.item_damage_per_attack != 1 {
                tag.put_int("item_damage_per_attack", weapon.item_damage_per_attack);
            }
            optional_float(
                &mut tag,
                "disable_blocking_for_seconds",
                weapon.disable_blocking_for_seconds,
                0.0,
            );
            tag
        },
        |tag| {
            let damage = "item_damage_per_attack";
            let disable = "disable_blocking_for_seconds";
            Ok(Weapon {
                item_damage_per_attack: bounded(
                    damage,
                    field(tag, damage, Some(1), Tag::as_int)?,
                    0..=i32::MAX,
                )?,
                disable_blocking_for_seconds: bounded(
                    disable,
                    field(tag, disable, Some(0.0), Tag::as_float)?,
                    0.0..=f32::MAX,
                )?,
            })
        },
    )
}

pub(super) fn food(ty: DataComponentType) -> DataComponentType {
    record(
        ty,
        |food: &FoodProperties| {
            let mut tag = CompoundTag::new();
            tag.put_int("nutrition", food.nutrition);
            tag.put_float("saturation", food.saturation);
            if food.can_always_eat {
                tag.put_bool("can_always_eat", true);
            }
            tag
        },
        |tag| {
            let nutrition = field(tag, "nutrition", None, Tag::as_int)?;
            Ok(FoodProperties {
                nutrition: bounded("nutrition", nutrition, 0..=i32::MAX)?,
                saturation: field(tag, "saturation", None, Tag::as_float)?,
                can_always_eat: field(tag, "can_always_eat", Some(false), Tag::as_bool)?,
            })
        },
    )
}

pub(super) fn swing_animation(ty: DataComponentType) -> DataComponentType {
    record(
        ty,
        |swing: &SwingAnimation| {
            let mut tag = CompoundTag::new();
            if swing.type_ != SwingAnimation::DEFAULT.type_ {
                tag.put_string("type", swing.type_.get_serialized_name());
            }
            if swing.duration != SwingAnimation::DEFAULT.duration {
                tag.put_int("duration", swing.duration);
            }
            tag
        },
        |tag| {
            let type_ = match tag.get("type") {
                None => SwingAnimation::DEFAULT.type_,
                Some(Tag::String(value)) => match value.as_str() {
                    "none" => SwingAnimationType::NONE,
                    "whack" => SwingAnimationType::WHACK,
                    "stab" => SwingAnimationType::STAB,
                    _ => return Err(format!("unknown swing animation type '{value}'")),
                },
                Some(_) => return Err("expected a swing animation type".into()),
            };
            let duration = field(
                tag,
                "duration",
                Some(SwingAnimation::DEFAULT.duration),
                Tag::as_int,
            )?;
            Ok(SwingAnimation {
                type_,
                duration: bounded("duration", duration, 1..=i32::MAX)?,
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::DataComponentPatch;
    use crate::nbt::snbt_parse::parse_compound;
    use crate::world::item::component::dyed_item_color::DyedItemColor;

    #[test]
    fn canonicalizes_typed_properties_and_checks_required_fields_and_ranges() {
        let nbt = parse_compound(
            r#"{
            attack_range:{min_reach:0.0f,max_reach:4.25f},
            use_effects:{can_sprint:true,interact_vibrations:true},
            use_cooldown:{seconds:0.5f,cooldown_group:"minecraft:trident"},
            weapon:{item_damage_per_attack:1,disable_blocking_for_seconds:5.0f},
            food:{nutrition:4,saturation:2.5f,can_always_eat:false},
            swing_animation:{type:"stab",duration:6},
            dyed_color:[1.0f,0.0f,0.0f]
        }"#,
        )
        .unwrap();
        let patch = DataComponentPatch::from_nbt(&nbt).unwrap();
        let saved = patch.to_nbt().unwrap();
        let record = |name| saved.get_compound(name).unwrap();
        assert_eq!(
            record("minecraft:attack_range").get_float("max_reach"),
            Some(4.25)
        );
        assert_eq!(record("minecraft:attack_range").len(), 1);
        assert_eq!(
            record("minecraft:use_effects").get_bool("can_sprint"),
            Some(true)
        );
        assert_eq!(record("minecraft:use_effects").len(), 1);
        assert_eq!(
            record("minecraft:use_cooldown").get_string("cooldown_group"),
            Some("minecraft:trident")
        );
        assert_eq!(record("minecraft:weapon").len(), 1);
        assert_eq!(record("minecraft:food").len(), 2);
        assert_eq!(
            record("minecraft:swing_animation").get_string("type"),
            Some("stab")
        );
        assert_eq!(record("minecraft:swing_animation").len(), 1);
        let color = crate::core::component::data_component_type::data_component_type_by_identifier(
            &Identifier::minecraft("dyed_color"),
        )
        .unwrap();
        assert_eq!(
            patch.get::<DyedItemColor>(&color).unwrap().rgb,
            0xffff0000u32 as i32
        );
        assert_eq!(DataComponentPatch::from_nbt(&saved).unwrap(), patch);

        for (invalid, field) in [
            ("{attack_range:{mob_factor:3.0f}}", "mob_factor"),
            ("{use_effects:{speed_multiplier:-0.1f}}", "speed_multiplier"),
            ("{use_cooldown:{seconds:0.0f}}", "seconds"),
            (
                "{weapon:{item_damage_per_attack:-1}}",
                "item_damage_per_attack",
            ),
            ("{food:{nutrition:4}}", "saturation"),
            ("{swing_animation:{type:jump}}", "jump"),
            ("{dyed_color:[1,0]}", "channels"),
        ] {
            let error =
                DataComponentPatch::from_nbt(&parse_compound(invalid).unwrap()).unwrap_err();
            assert!(error.contains(field), "{error}");
        }
    }
}
