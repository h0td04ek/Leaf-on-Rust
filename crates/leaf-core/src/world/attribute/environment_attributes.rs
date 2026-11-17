//! `net.minecraft.world.attribute.EnvironmentAttributes` —
//! реестр `BuiltInRegistries.ENVIRONMENT_ATTRIBUTE` (49 атрибутов
//! в vanilla-порядке).

use super::attribute_range::AttributeRange;
use super::attribute_type::attribute_type_by_identifier;
use super::attribute_value::{
    AmbientParticle, AmbientSounds, BackgroundMusic, AttributeValue,
};
use super::environment_attribute::{EnvironmentAttribute, EnvironmentAttributeBuilder};
use crate::core::holder::Holder;
use crate::core::particles::particle_options::ParticleOptions;
use crate::entity::schedule::activity::{activity_by_name, Activity};
use crate::resources::identifier::Identifier;
use crate::util::tri_state::TriState;
use crate::world::attribute::bed_rule::BedRule;
use crate::world::level::moon_phase::MoonPhase;
use std::sync::{Arc, OnceLock, RwLock};

/// Реестр всех `EnvironmentAttribute` (порядок регистрации vanilla).
pub fn environment_attributes() -> &'static [Arc<EnvironmentAttribute>] {
    static ATTRS: OnceLock<Vec<Arc<EnvironmentAttribute>>> = OnceLock::new();
    ATTRS.get_or_init(|| {
        let mut v = Vec::new();
        let mut reg = |path: &str, builder: EnvironmentAttributeBuilder| {
            let id = Identifier::minecraft(path);
            v.push(Arc::new(builder.build(id)));
        };

        let ty = |name: &str| attribute_type_by_identifier(&Identifier::minecraft(name)).unwrap();
        let float = |v: f32| AttributeValue::Float(v);
        let boolean = |v: bool| AttributeValue::Boolean(v);
        let int = |v: i32| AttributeValue::Int(v);
        let unit = AttributeRange::UNIT_FLOAT;
        let non_neg = AttributeRange::NON_NEGATIVE_FLOAT;

        // visual/*
        reg("visual/fog_color", EnvironmentAttributeBuilder::new(ty("rgb_color"))
            .default_value(int(0)).spatially_interpolated().syncable());
        reg("visual/fog_start_distance", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(0.0)).spatially_interpolated().syncable());
        reg("visual/fog_end_distance", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(1024.0)).value_range(non_neg).spatially_interpolated().syncable());
        reg("visual/sky_fog_end_distance", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(512.0)).value_range(non_neg).spatially_interpolated().syncable());
        reg("visual/cloud_fog_end_distance", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(2048.0)).value_range(non_neg).spatially_interpolated().syncable());
        reg("visual/water_fog_color", EnvironmentAttributeBuilder::new(ty("rgb_color"))
            .default_value(int(-16448205)).spatially_interpolated().syncable());
        reg("visual/water_fog_start_distance", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(-8.0)).spatially_interpolated().syncable());
        reg("visual/water_fog_end_distance", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(96.0)).value_range(non_neg).spatially_interpolated().syncable());
        reg("visual/sky_color", EnvironmentAttributeBuilder::new(ty("rgb_color"))
            .default_value(int(0)).spatially_interpolated().syncable());
        reg("visual/sunrise_sunset_color", EnvironmentAttributeBuilder::new(ty("argb_color"))
            .default_value(int(0)).spatially_interpolated().syncable());
        reg("visual/cloud_color", EnvironmentAttributeBuilder::new(ty("argb_color"))
            .default_value(int(0)).spatially_interpolated().syncable());
        reg("visual/cloud_height", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(192.33)).spatially_interpolated().syncable());
        reg("visual/sun_angle", EnvironmentAttributeBuilder::new(ty("angle_degrees"))
            .default_value(float(0.0)).spatially_interpolated().syncable());
        reg("visual/moon_angle", EnvironmentAttributeBuilder::new(ty("angle_degrees"))
            .default_value(float(0.0)).spatially_interpolated().syncable());
        reg("visual/star_angle", EnvironmentAttributeBuilder::new(ty("angle_degrees"))
            .default_value(float(0.0)).spatially_interpolated().syncable());
        reg("visual/moon_phase", EnvironmentAttributeBuilder::new(ty("moon_phase"))
            .default_value(AttributeValue::MoonPhase(MoonPhase::FullMoon)).syncable());
        reg("visual/star_brightness", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(0.0)).value_range(unit).spatially_interpolated().syncable());
        reg("visual/sky_light_color", EnvironmentAttributeBuilder::new(ty("rgb_color"))
            .default_value(int(-1)).spatially_interpolated().syncable());
        reg("visual/sky_light_factor", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(1.0)).value_range(unit).spatially_interpolated().syncable());
        let dripstone_particle = crate::core::particles::particle_types::particle_type_by_identifier(
            &Identifier::minecraft("dripping_dripstone_water"),
        )
        .expect("particle dripping_dripstone_water");
        reg("visual/default_dripstone_particle", EnvironmentAttributeBuilder::new(ty("particle"))
            .default_value(AttributeValue::Particle(ParticleOptions::simple(dripstone_particle))).syncable());
        reg("visual/ambient_particles", EnvironmentAttributeBuilder::new(ty("ambient_particles"))
            .default_value(AttributeValue::AmbientParticles(Vec::<AmbientParticle>::new())).syncable());

        // audio/*
        reg("audio/background_music", EnvironmentAttributeBuilder::new(ty("background_music"))
            .default_value(AttributeValue::BackgroundMusic(BackgroundMusic::empty())).syncable());
        reg("audio/music_volume", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(1.0)).value_range(unit).syncable());
        reg("audio/ambient_sounds", EnvironmentAttributeBuilder::new(ty("ambient_sounds"))
            .default_value(AttributeValue::AmbientSounds(AmbientSounds::empty())).syncable());
        reg("audio/firefly_bush_sounds", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(false)).syncable());

        // gameplay/*
        reg("gameplay/sky_light_level", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(15.0)).value_range(AttributeRange::of_float(0.0, 15.0)).not_positional().syncable());
        reg("gameplay/can_start_raid", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(true)));
        reg("gameplay/water_evaporates", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(false)).syncable());
        reg("gameplay/bed_rule", EnvironmentAttributeBuilder::new(ty("bed_rule"))
            .default_value(AttributeValue::BedRule(BedRule::can_sleep_when_dark())));
        reg("gameplay/respawn_anchor_works", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(false)));
        reg("gameplay/nether_portal_spawns_piglin", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(false)));
        reg("gameplay/fast_lava", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(false)).not_positional().syncable());
        reg("gameplay/increased_fire_burnout", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(false)));
        reg("gameplay/eyeblossom_open", EnvironmentAttributeBuilder::new(ty("tri_state"))
            .default_value(AttributeValue::TriState(TriState::Default)));
        reg("gameplay/turtle_egg_hatch_chance", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(0.0)).value_range(unit));
        reg("gameplay/piglins_zombify", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(true)).syncable());
        reg("gameplay/snow_golem_melts", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(false)));
        reg("gameplay/creaking_active", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(false)).syncable());
        reg("gameplay/surface_slime_spawn_chance", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(0.0)).value_range(unit));
        reg("gameplay/cat_waking_up_gift_chance", EnvironmentAttributeBuilder::new(ty("float"))
            .default_value(float(0.0)).value_range(unit));
        reg("gameplay/bees_stay_in_hive", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(false)));
        reg("gameplay/monsters_burn", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(false)));
        reg("gameplay/can_pillager_patrol_spawn", EnvironmentAttributeBuilder::new(ty("boolean"))
            .default_value(boolean(true)));
        let idle = AttributeValue::Activity(
            activity_by_name("idle").map(|a| Activity { id: a.id.clone(), activity_id: a.activity_id })
                .unwrap_or_else(|| panic!("activity idle")),
        );
        reg("gameplay/villager_activity", EnvironmentAttributeBuilder::new(ty("activity"))
            .default_value(idle.clone()));
        reg("gameplay/baby_villager_activity", EnvironmentAttributeBuilder::new(ty("activity"))
            .default_value(idle));

        v
    })
}

/// Атрибут по registry-id.
pub fn environment_attribute_by_id(id: i32) -> Option<Arc<EnvironmentAttribute>> {
    environment_attributes().get(id as usize).cloned()
}

/// Атрибут по identifier.
pub fn environment_attribute_by_identifier(id: &Identifier) -> Option<Arc<EnvironmentAttribute>> {
    environment_attributes().iter().find(|a| &a.id == id).cloned()
}
