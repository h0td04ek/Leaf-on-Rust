//! Перенос `net.minecraft.world.timeline.Timelines` —
//! ключи и bootstrap-данные встроенных таймлайнов
//! (`day`, `moon`, `villager_schedule`, `early_game`).

use crate::core::registry::registries;
use crate::entity::schedule::activity::{activity_by_name, Activity};
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::util::argb;
use crate::util::easing_type::{constant, EasingType};
use crate::util::tri_state::TriState;
use crate::world::attribute::attribute_modifier::{AttributeModifier, AttributeModifierKind};
use crate::world::attribute::attribute_value::AttributeValue;
use crate::world::attribute::environment_attribute::EnvironmentAttribute;
use crate::world::attribute::environment_attributes::environment_attribute_by_identifier;
use crate::world::level::dimension::dimension_type::DimensionType;
use crate::world::level::moon_phase::MoonPhase;
use crate::world::timeline::timeline::Timeline;
use crate::world::timeline::TimelineBuilder;
use std::sync::Arc;


/// `Timelines.DAY`.
pub fn day() -> ResourceKey<Timeline> {
    key("day")
}
/// `Timelines.MOON`.
pub fn moon() -> ResourceKey<Timeline> {
    key("moon")
}
/// `Timelines.VILLAGER_SCHEDULE`.
pub fn villager_schedule() -> ResourceKey<Timeline> {
    key("villager_schedule")
}
/// `Timelines.EARLY_GAME`.
pub fn early_game() -> ResourceKey<Timeline> {
    key("early_game")
}

/// `DAY_SKY_LIGHT_LEVEL`.
pub const DAY_SKY_LIGHT_LEVEL: f32 = 15.0;
/// `NIGHT_SKY_LIGHT_LEVEL`.
pub const NIGHT_SKY_LIGHT_LEVEL: f32 = 4.0;
/// `NIGHT_SKY_LIGHT_COLOR`.
pub fn night_sky_light_color() -> i32 {
    argb::color_from_float(1.0, 0.48, 0.48, 1.0)
}
/// `NIGHT_SKY_LIGHT_FACTOR`.
pub const NIGHT_SKY_LIGHT_FACTOR: f32 = 0.24;
/// `NIGHT_SKY_COLOR_MULTIPLIER`.
pub const NIGHT_SKY_COLOR_MULTIPLIER: i32 = -16777216;
/// `NIGHT_FOG_COLOR_MULTIPLIER`.
pub fn night_fog_color_multiplier() -> i32 {
    argb::color_from_float(1.0, 0.06, 0.06, 0.09)
}
/// `NIGHT_CLOUD_COLOR_MULTIPLIER`.
pub fn night_cloud_color_multiplier() -> i32 {
    argb::color_from_float(1.0, 0.1, 0.1, 0.15)
}

fn key(id: &str) -> ResourceKey<Timeline> {
    ResourceKey::create_with_registry_name(
        registries::timeline(),
        Identifier::with_default_namespace(id).unwrap(),
    )
}

fn attr(path: &str) -> Arc<EnvironmentAttribute> {
    environment_attribute_by_identifier(&Identifier::minecraft(path))
        .unwrap_or_else(|| panic!("missing environment attribute {}", path))
}

fn kf_float(v: f32) -> AttributeValue {
    AttributeValue::Float(v)
}
fn kf_int(v: i32) -> AttributeValue {
    AttributeValue::Int(v)
}
fn kf_bool(v: bool) -> AttributeValue {
    AttributeValue::Boolean(v)
}
fn kf_tristate(v: TriState) -> AttributeValue {
    AttributeValue::TriState(v)
}
fn kf_activity(name: &str) -> AttributeValue {
    AttributeValue::Activity(
        activity_by_name(name)
            .map(|a| (*a).clone())
            .unwrap_or_else(|| panic!("missing activity {}", name)),
    )
}
fn kf_moon_phase(p: MoonPhase) -> AttributeValue {
    AttributeValue::MoonPhase(p)
}

fn or_modifier() -> AttributeModifier {
    AttributeModifier::Boolean(AttributeModifierKind::Or)
}
fn and_modifier() -> AttributeModifier {
    AttributeModifier::Boolean(AttributeModifierKind::And)
}
fn mul_rgb() -> AttributeModifier {
    AttributeModifier::Color(AttributeModifierKind::MultiplyRgb)
}
fn mul_argb() -> AttributeModifier {
    AttributeModifier::Color(AttributeModifierKind::MultiplyArgb)
}
fn max_float() -> AttributeModifier {
    AttributeModifier::Float(AttributeModifierKind::Maximum)
}
fn mul_float() -> AttributeModifier {
    AttributeModifier::Float(AttributeModifierKind::Multiply)
}

/// `Timelines.bootstrap(context)` — регистрирует все
/// встроенные таймлайны. `register(key, timeline)` —
/// аналог `BootstrapContext.register`.
pub fn bootstrap<F>(mut register: F)
where
    F: FnMut(ResourceKey<Timeline>, Timeline),
{
    // --- DAY ---
    let easing = EasingType::symmetric_cubic_bezier(0.362, 0.241);
    let mut b = TimelineBuilder::new();
    b.set_period_ticks(24000);
    b.add_track(attr("visual/sun_angle"), |k| {
        k.set_easing(easing.clone())
            .add_keyframe(6000, kf_float(360.0))
            .add_keyframe(6000, kf_float(0.0));
    });
    b.add_track(attr("visual/moon_angle"), |k| {
        k.set_easing(easing.clone())
            .add_keyframe(6000, kf_float(540.0))
            .add_keyframe(6000, kf_float(180.0));
    });
    b.add_track(attr("visual/star_angle"), |k| {
        k.set_easing(easing.clone())
            .add_keyframe(6000, kf_float(360.0))
            .add_keyframe(6000, kf_float(0.0));
    });
    b.add_modifier_track(attr("audio/firefly_bush_sounds"), or_modifier(), |k| {
        k.add_keyframe(12600, kf_bool(true)).add_keyframe(23401, kf_bool(false));
    });
    b.add_modifier_track(attr("visual/fog_color"), mul_rgb(), |k| {
        k.add_keyframe(133, kf_int(-1))
            .add_keyframe(11867, kf_int(-1))
            .add_keyframe(13670, kf_int(night_fog_color_multiplier()))
            .add_keyframe(22330, kf_int(night_fog_color_multiplier()));
    });
    b.add_modifier_track(attr("visual/sky_color"), mul_rgb(), |k| {
        k.add_keyframe(133, kf_int(-1))
            .add_keyframe(11867, kf_int(-1))
            .add_keyframe(13670, kf_int(NIGHT_SKY_COLOR_MULTIPLIER))
            .add_keyframe(22330, kf_int(NIGHT_SKY_COLOR_MULTIPLIER));
    });
    b.add_modifier_track(attr("visual/sky_light_color"), mul_rgb(), |k| {
        k.add_keyframe(730, kf_int(-1))
            .add_keyframe(11270, kf_int(-1))
            .add_keyframe(13140, kf_int(night_sky_light_color()))
            .add_keyframe(22860, kf_int(night_sky_light_color()));
    });
    b.add_modifier_track(attr("visual/sky_light_factor"), mul_float(), |k| {
        k.add_keyframe(730, kf_float(1.0))
            .add_keyframe(11270, kf_float(1.0))
            .add_keyframe(13140, kf_float(NIGHT_SKY_LIGHT_FACTOR))
            .add_keyframe(22860, kf_float(NIGHT_SKY_LIGHT_FACTOR));
    });
    b.add_modifier_track(attr("gameplay/sky_light_level"), mul_float(), |k| {
        k.add_keyframe(133, kf_float(1.0))
            .add_keyframe(11867, kf_float(1.0))
            .add_keyframe(13670, kf_float(0.26666668))
            .add_keyframe(22330, kf_float(0.26666668));
    });
    b.add_track(attr("visual/sunrise_sunset_color"), |k| {
        for (t, v) in [
            (71, 1609540403i32), (310, 703969843), (565, 117167155),
            (730, 16770355), (11270, 16770355), (11397, 83679283),
            (11522, 268028723), (11690, 703969843), (11929, 1609540403),
            (12243, -1310226637), (12358, -857440717), (12512, -371166669),
            (12613, -153261261), (12732, -19242189), (12841, -19440589),
            (13035, -321760973), (13252, -1043577037), (13775, 918435635),
            (13888, 532362547), (14039, 163001139), (14192, 11744051),
            (21807, 11678515), (21961, 163001139), (22112, 532362547),
            (22225, 918435635), (22748, -1043577037), (22965, -321760973),
            (23159, -19440589), (23272, -19242189), (23488, -371166669),
            (23642, -857440717), (23757, -1310226637),
        ] {
            k.add_keyframe(t, kf_int(v));
        }
    });
    b.add_modifier_track(attr("visual/star_brightness"), max_float(), |k| {
        for (t, v) in [
            (92, 0.037f32), (627, 0.0), (11373, 0.0), (11732, 0.016),
            (11959, 0.044), (12399, 0.143), (12729, 0.258), (13228, 0.5),
            (22772, 0.5), (23032, 0.364), (23356, 0.225), (23758, 0.101),
        ] {
            k.add_keyframe(t, kf_float(v));
        }
    });
    b.add_modifier_track(attr("visual/cloud_color"), mul_argb(), |k| {
        k.add_keyframe(133, kf_int(-1))
            .add_keyframe(11867, kf_int(-1))
            .add_keyframe(13670, kf_int(night_cloud_color_multiplier()))
            .add_keyframe(22330, kf_int(night_cloud_color_multiplier()));
    });
    b.add_track(attr("gameplay/eyeblossom_open"), |k| {
        k.add_keyframe(12600, kf_tristate(TriState::True))
            .add_keyframe(23401, kf_tristate(TriState::False));
    });
    b.add_modifier_track(attr("gameplay/creaking_active"), or_modifier(), |k| {
        k.add_keyframe(12600, kf_bool(true)).add_keyframe(23401, kf_bool(false));
    });
    b.add_modifier_track(attr("gameplay/turtle_egg_hatch_chance"), max_float(), |k| {
        k.set_easing(constant())
            .add_keyframe(21062, kf_float(1.0))
            .add_keyframe(21905, kf_float(0.002));
    });
    b.add_modifier_track(attr("gameplay/cat_waking_up_gift_chance"), max_float(), |k| {
        k.set_easing(constant())
            .add_keyframe(362, kf_float(0.0))
            .add_keyframe(23667, kf_float(0.7));
    });
    b.add_modifier_track(attr("gameplay/bees_stay_in_hive"), or_modifier(), |k| {
        k.add_keyframe(12542, kf_bool(true)).add_keyframe(23460, kf_bool(false));
    });
    b.add_modifier_track(attr("gameplay/monsters_burn"), or_modifier(), |k| {
        k.add_keyframe(12542, kf_bool(false)).add_keyframe(23460, kf_bool(true));
    });
    register(day(), b.build().expect("day timeline"));

    // --- MOON ---
    let mut b = TimelineBuilder::new();
    b.set_period_ticks(24000 * MoonPhase::COUNT as i32);
    b.add_track(attr("visual/moon_phase"), |k| {
        for p in MoonPhase::VALUES {
            k.add_keyframe(p.start_tick(), kf_moon_phase(p));
        }
    });
    b.add_modifier_track(attr("gameplay/surface_slime_spawn_chance"), max_float(), |k| {
        k.set_easing(constant());
        for p in MoonPhase::VALUES {
            k.add_keyframe(
                p.start_tick(),
                kf_float(DimensionType::MOON_BRIGHTNESS_PER_PHASE[p.index() as usize] * 0.5),
            );
        }
    });
    register(moon(), b.build().expect("moon timeline"));

    // --- VILLAGER_SCHEDULE ---
    let mut b = TimelineBuilder::new();
    b.set_period_ticks(24000);
    b.add_track(attr("gameplay/villager_activity"), |k| {
        k.add_keyframe(10, kf_activity("idle"))
            .add_keyframe(2000, kf_activity("work"))
            .add_keyframe(9000, kf_activity("meet"))
            .add_keyframe(11000, kf_activity("idle"))
            .add_keyframe(12000, kf_activity("rest"));
    });
    b.add_track(attr("gameplay/baby_villager_activity"), |k| {
        k.add_keyframe(10, kf_activity("idle"))
            .add_keyframe(3000, kf_activity("play"))
            .add_keyframe(6000, kf_activity("idle"))
            .add_keyframe(10000, kf_activity("play"))
            .add_keyframe(12000, kf_activity("rest"));
    });
    register(villager_schedule(), b.build().expect("villager_schedule timeline"));

    // --- EARLY_GAME ---
    let mut b = TimelineBuilder::new();
    b.add_modifier_track(attr("gameplay/can_pillager_patrol_spawn"), and_modifier(), |k| {
        k.add_keyframe(0, kf_bool(false)).add_keyframe(120000, kf_bool(true));
    });
    register(early_game(), b.build().expect("early_game timeline"));
}
