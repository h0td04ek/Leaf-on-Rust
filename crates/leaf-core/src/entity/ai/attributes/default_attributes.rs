//! Перенос `net.minecraft.world.entity.ai.attributes.DefaultAttributes`.
//!
//! Каждый `EntityType` получает `AttributeSupplier` из своего `createAttributes()`
//! в иерархии классов. Здесь — базовые билдеры (`LivingEntity`/`Mob`/`Monster`/
//! `Animal`/`Player`/`AbstractHorse`...) и таблица vanilla-переопределений для
//! всех мобов, чьи `createAttributes` уже известны из Java-исходников.

use super::attribute::Attribute;
use super::attribute_supplier::{AttributeSupplier, AttributeSupplierBuilder};
use super::attributes::{all_attributes, *};
use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::OnceLock;

/// `Attribute` из реестра `all_attributes` (clone).
fn attr(id: &Identifier) -> Attribute {
    all_attributes()
        .get(id)
        .expect("attribute must exist")
        .base
        .clone()
}

/// `LivingEntity.createLivingAttributes()`.
pub fn create_living_attributes() -> AttributeSupplierBuilder {
    let mut b = AttributeSupplier::create();
    for id in [
        max_health(),
        knockback_resistance(),
        movement_speed(),
        armor(),
        armor_toughness(),
        max_absorption(),
        step_height(),
        scale(),
        gravity(),
        safe_fall_distance(),
        fall_damage_multiplier(),
        jump_strength(),
        oxygen_bonus(),
        burning_time(),
        explosion_knockback_resistance(),
        water_movement_efficiency(),
        movement_efficiency(),
        attack_knockback(),
        camera_distance(),
        waypoint_transmit_range(),
    ] {
        b = b.add(attr(&id));
    }
    b
}

/// `Mob.createMobAttributes()` = living + follow_range=16.
pub fn create_mob_attributes() -> AttributeSupplierBuilder {
    create_living_attributes()
        .add(Attribute {
            default_value: 16.0,
            ..attr(&follow_range())
        })
}

/// `Monster.createMonsterAttributes()` = mob + attack_damage.
pub fn create_monster_attributes() -> AttributeSupplierBuilder {
    create_mob_attributes().add(attr(&attack_damage()))
}

/// `Animal.createAnimalAttributes()` = mob + tempt_range=10.
pub fn create_animal_attributes() -> AttributeSupplierBuilder {
    create_mob_attributes().add(Attribute {
        default_value: 10.0,
        ..attr(&tempt_range())
    })
}

/// `WaterAnimal.createWaterCreatureAttributes()` = mob + knockback_resistance=0.0.
pub fn create_water_creature_attributes() -> AttributeSupplierBuilder {
    create_mob_attributes()
}

/// `FlyingMonster.createFlyingMonsterAttributes()` = monster + flying_speed=0.02.
pub fn create_flying_monster_attributes() -> AttributeSupplierBuilder {
    create_monster_attributes().add(Attribute {
        default_value: 0.02,
        ..attr(&flying_speed())
    })
}

/// `Player.createAttributes()`.
pub fn create_player_attributes() -> AttributeSupplierBuilder {
    let b = create_living_attributes()
        .add(Attribute { default_value: 1.0, ..attr(&attack_damage()) })
        .add(Attribute { default_value: 0.1, ..attr(&movement_speed()) })
        .add(attr(&attack_speed()))
        .add(attr(&luck()))
        .add(Attribute { default_value: 4.5, ..attr(&block_interaction_range()) })
        .add(Attribute { default_value: 3.0, ..attr(&entity_interaction_range()) })
        .add(attr(&block_break_speed()))
        .add(attr(&submerged_mining_speed()))
        .add(Attribute { default_value: 0.3, ..attr(&sneaking_speed()) })
        .add(attr(&mining_efficiency()))
        .add(attr(&sweeping_damage_ratio()))
        .add(Attribute { default_value: 6.0e7, ..attr(&waypoint_transmit_range()) })
        .add(Attribute { default_value: 6.0e7, ..attr(&waypoint_receive_range()) });
    b
}

/// `AbstractHorse.createBaseHorseAttributes()`.
pub fn create_base_horse_attributes() -> AttributeSupplierBuilder {
    create_animal_attributes()
        .add(Attribute { default_value: 0.7, ..attr(&jump_strength()) })
        .add(Attribute { default_value: 53.0, ..attr(&max_health()) })
        .add(Attribute { default_value: 0.225, ..attr(&movement_speed()) })
        .add(Attribute { default_value: 1.0, ..attr(&step_height()) })
        .add(Attribute { default_value: 6.0, ..attr(&safe_fall_distance()) })
        .add(Attribute { default_value: 0.5, ..attr(&fall_damage_multiplier()) })
}

/// `AbstractChestedHorse.createBaseChestedHorseAttributes()` = horse + donkey speed/health.
pub fn create_base_chested_horse_attributes() -> AttributeSupplierBuilder {
    create_base_horse_attributes()
        .add(Attribute { default_value: 15.0, ..attr(&max_health()) })
        .add(Attribute { default_value: 0.175, ..attr(&movement_speed()) })
        .add(Attribute { default_value: 0.5, ..attr(&jump_strength()) })
}

/// Таблица `DefaultAttributes.SUPPLIERS` — entity id -> supplier.
pub fn suppliers() -> &'static HashMap<Identifier, AttributeSupplier> {
    static MAP: OnceLock<HashMap<Identifier, AttributeSupplier>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        let mut reg = |id: &str, b: AttributeSupplierBuilder| {
            m.insert(Identifier::minecraft(id), b.build());
        };

        // --- Базовые сущности ---
        reg("player", create_player_attributes());
        reg("armor_stand", create_living_attributes());
        reg("mannequin", create_living_attributes());

        // --- Животные (Animal.createAnimalAttributes + overrides) ---
        reg("allay", create_animal_attributes()
            .add(attr(&attack_damage()))
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.1, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 48.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 1.0, ..attr(&jump_strength()) }));
        reg("armadillo", create_animal_attributes()
            .add(Attribute { default_value: 0.15, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 12.0, ..attr(&max_health()) })
            .add(attr(&attack_damage())));
        reg("axolotl", create_animal_attributes()
            .add(Attribute { default_value: 14.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 1.0, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 2.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 1.0, ..attr(&step_height()) }));
        reg("bat", create_mob_attributes()
            .add(Attribute { default_value: 6.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.6, ..attr(&flying_speed()) }));
        reg("bee", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.7, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 2.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.3, ..attr(&flying_speed()) })
            .add(Attribute { default_value: 24.0, ..attr(&follow_range()) }));
        reg("camel", create_base_horse_attributes()
            .add(Attribute { default_value: 32.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.09, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 0.42, ..attr(&jump_strength()) })
            .add(Attribute { default_value: 1.5, ..attr(&step_height()) }));
        reg("camel_husk", create_base_horse_attributes()
            .add(Attribute { default_value: 32.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.09, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 0.42, ..attr(&jump_strength()) })
            .add(Attribute { default_value: 1.5, ..attr(&step_height()) }));
        reg("cat", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) }));
        reg("chicken", create_animal_attributes()
            .add(Attribute { default_value: 4.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) })
            .add(attr(&attack_damage()))
            .add(Attribute { default_value: 2.0, ..attr(&jump_strength()) }));
        reg("cod", create_water_creature_attributes()
            .add(Attribute { default_value: 3.0, ..attr(&max_health()) }));
        reg("cow", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.2, ..attr(&movement_speed()) }));
        reg("mooshroom", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.2, ..attr(&movement_speed()) }));
        reg("copper_golem", create_animal_attributes()
            .add(Attribute { default_value: 16.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.2, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) }));
        reg("dolphin", create_water_creature_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 1.2, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) }));
        reg("donkey", create_base_chested_horse_attributes());
        reg("horse", create_base_horse_attributes()
            .add(Attribute { default_value: 0.6, ..attr(&jump_strength()) }));
        reg("mule", create_base_chested_horse_attributes());
        reg("skeleton_horse", create_base_horse_attributes()
            .add(Attribute { default_value: 0.6, ..attr(&jump_strength()) })
            .add(Attribute { default_value: 1.0, ..attr(&step_height()) }));
        reg("zombie_horse", create_base_horse_attributes());
        reg("fox", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 2.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 5.0, ..attr(&safe_fall_distance()) })
            .add(Attribute { default_value: 32.0, ..attr(&follow_range()) }));
        reg("frog", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 1.2, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 10.0, ..attr(&jump_strength()) }));
        reg("tadpole", create_animal_attributes()
            .add(Attribute { default_value: 6.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.1, ..attr(&movement_speed()) }));
        reg("glow_squid", create_mob_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) }));
        reg("squid", create_mob_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) }));
        reg("goat", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.2, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 2.0, ..attr(&attack_damage()) }));
        reg("happy_ghast", create_animal_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.5, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 4.0, ..attr(&jump_strength()) }));
        reg("iron_golem", create_animal_attributes()
            .add(Attribute { default_value: 100.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 15.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 5.0, ..attr(&knockback_resistance()) })
            .add(Attribute { default_value: 1.0, ..attr(&step_height()) }));
        reg("llama", create_base_horse_attributes()
            .add(Attribute { default_value: 9.0, ..attr(&jump_strength()) })
            .add(Attribute { default_value: 15.0, ..attr(&step_height()) })
            .add(Attribute { default_value: 64.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 2.0, ..attr(&attack_knockback()) })
            .add(Attribute { default_value: 15.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.175, ..attr(&movement_speed()) }));
        reg("trader_llama", create_base_horse_attributes()
            .add(Attribute { default_value: 9.0, ..attr(&jump_strength()) })
            .add(Attribute { default_value: 15.0, ..attr(&step_height()) })
            .add(Attribute { default_value: 64.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 2.0, ..attr(&attack_knockback()) })
            .add(Attribute { default_value: 15.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.175, ..attr(&movement_speed()) }));
        reg("nautilus", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 1.0, ..attr(&movement_speed()) }));
        reg("zombie_nautilus", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 1.0, ..attr(&movement_speed()) }));
        reg("ocelot", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) }));
        reg("panda", create_animal_attributes()
            .add(Attribute { default_value: 0.15, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 6.0, ..attr(&attack_damage()) }));
        reg("parrot", create_animal_attributes()
            .add(Attribute { default_value: 6.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.4, ..attr(&flying_speed()) })
            .add(Attribute { default_value: 0.2, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) }));
        reg("pig", create_animal_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) }));
        reg("polar_bear", create_animal_attributes()
            .add(Attribute { default_value: 30.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) })
            .add(attr(&attack_damage()))
            .add(Attribute { default_value: 2.0, ..attr(&attack_knockback()) }));
        reg("pufferfish", create_water_creature_attributes()
            .add(Attribute { default_value: 3.0, ..attr(&max_health()) }));
        reg("rabbit", create_animal_attributes()
            .add(Attribute { default_value: 3.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) }));
        reg("salmon", create_water_creature_attributes()
            .add(Attribute { default_value: 3.0, ..attr(&max_health()) }));
        reg("sheep", create_animal_attributes()
            .add(Attribute { default_value: 8.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.23, ..attr(&movement_speed()) }));
        reg("sniffer", create_animal_attributes()
            .add(Attribute { default_value: 14.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) }));
        reg("snow_golem", create_animal_attributes()
            .add(Attribute { default_value: 4.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.2, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 2.0, ..attr(&attack_damage()) }));
        reg("strider", create_animal_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.7, ..attr(&movement_speed()) })
            .add(attr(&attack_damage())));
        reg("turtle", create_animal_attributes()
            .add(Attribute { default_value: 30.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 1.0, ..attr(&step_height()) }));
        reg("villager", create_animal_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.5, ..attr(&movement_speed()) }));
        reg("wandering_trader", create_animal_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.5, ..attr(&movement_speed()) }));
        reg("wolf", create_animal_attributes()
            .add(Attribute { default_value: 8.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 4.0, ..attr(&attack_damage()) }));

        // --- Монстры (Monster.createMonsterAttributes + overrides) ---
        reg("blaze", create_monster_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.23, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 6.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.5, ..attr(&knockback_resistance()) })
            .add(Attribute { default_value: 0.5, ..attr(&follow_range()) }));
        reg("bogged", create_monster_attributes()
            .add(Attribute { default_value: 16.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) }));
        reg("breeze", create_monster_attributes()
            .add(Attribute { default_value: 30.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 20.0, ..attr(&follow_range()) }));
        reg("creaking", create_monster_attributes()
            .add(Attribute { default_value: 1.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.4, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 32.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 1.0, ..attr(&armor()) }));
        reg("creeper", create_monster_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) }));
        reg("drowned", create_monster_attributes()
            .add(Attribute { default_value: 35.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 2.0, ..attr(&armor()) })
            .add(Attribute { default_value: 1.0, ..attr(&step_height()) })
            .add(Attribute { default_value: 0.0, ..attr(&spawn_reinforcements()) }));
        reg("elder_guardian", create_monster_attributes()
            .add(Attribute { default_value: 80.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 8.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 10.0, ..attr(&armor()) })
            .add(Attribute { default_value: 10.0, ..attr(&armor_toughness()) })
            .add(Attribute { default_value: 1.0, ..attr(&knockback_resistance()) }));
        reg("ender_dragon", create_monster_attributes()
            .add(Attribute { default_value: 200.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 10.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 1.0, ..attr(&knockback_resistance()) })
            .add(Attribute { default_value: 64.0, ..attr(&follow_range()) }));
        reg("enderman", create_monster_attributes()
            .add(Attribute { default_value: 40.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 7.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 64.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 1.0, ..attr(&step_height()) }));
        reg("endermite", create_monster_attributes()
            .add(Attribute { default_value: 8.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 2.0, ..attr(&attack_damage()) }));
        reg("evoker", create_monster_attributes()
            .add(Attribute { default_value: 24.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.5, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 1.0, ..attr(&attack_knockback()) })
            .add(Attribute { default_value: 1.0, ..attr(&attack_speed()) }));
        reg("ghast", create_flying_monster_attributes()
            .add(Attribute { default_value: 10.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 1.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 6.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.06, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 0.02, ..attr(&flying_speed()) }));
        reg("giant", create_monster_attributes()
            .add(Attribute { default_value: 100.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.5, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 50.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.5, ..attr(&knockback_resistance()) }));
        reg("guardian", create_monster_attributes()
            .add(Attribute { default_value: 30.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 10.0, ..attr(&armor()) })
            .add(Attribute { default_value: 10.0, ..attr(&armor_toughness()) })
            .add(Attribute { default_value: 1.0, ..attr(&knockback_resistance()) }));
        reg("hoglin", create_monster_attributes()
            .add(Attribute { default_value: 40.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 1.6, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 6.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 1.0, ..attr(&knockback_resistance()) })
            .add(Attribute { default_value: 0.0, ..attr(&knockback_resistance()) }));
        reg("husk", create_monster_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.23, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 2.0, ..attr(&armor()) })
            .add(Attribute { default_value: 35.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 0.0, ..attr(&spawn_reinforcements()) }));
        reg("illusioner", create_monster_attributes()
            .add(Attribute { default_value: 32.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.5, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 1.0, ..attr(&attack_knockback()) })
            .add(Attribute { default_value: 1.0, ..attr(&attack_speed()) }));
        reg("magma_cube", create_monster_attributes()
            .add(Attribute { default_value: 16.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.2, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 6.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.2, ..attr(&knockback_resistance()) })
            .add(Attribute { default_value: 0.0, ..attr(&spawn_reinforcements()) }));
        reg("phantom", create_flying_monster_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 6.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.1, ..attr(&flying_speed()) }));
        reg("piglin", create_monster_attributes()
            .add(Attribute { default_value: 16.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.35, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 5.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.0, ..attr(&spawn_reinforcements()) }));
        reg("piglin_brute", create_monster_attributes()
            .add(Attribute { default_value: 50.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.35, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 7.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 1.0, ..attr(&knockback_resistance()) })
            .add(Attribute { default_value: 0.0, ..attr(&spawn_reinforcements()) }));
        reg("pillager", create_monster_attributes()
            .add(Attribute { default_value: 24.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.35, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 1.0, ..attr(&attack_knockback()) })
            .add(Attribute { default_value: 1.0, ..attr(&attack_speed()) }));
        reg("ravager", create_monster_attributes()
            .add(Attribute { default_value: 100.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 12.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 10.0, ..attr(&armor()) })
            .add(Attribute { default_value: 0.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 1.0, ..attr(&knockback_resistance()) }));
        reg("shulker", create_monster_attributes()
            .add(Attribute { default_value: 30.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.0, ..attr(&movement_speed()) })
            .add(attr(&attack_damage()))
            .add(Attribute { default_value: 1.0, ..attr(&knockback_resistance()) }));
        reg("silverfish", create_monster_attributes()
            .add(Attribute { default_value: 8.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 1.0, ..attr(&attack_damage()) }));
        reg("skeleton", create_monster_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) }));
        reg("stray", create_monster_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) }));
        reg("wither_skeleton", create_monster_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 4.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.0, ..attr(&knockback_resistance()) })
            .add(Attribute { default_value: 1.0, ..attr(&knockback_resistance()) }));
        reg("slime", create_monster_attributes()
            .add(Attribute { default_value: 16.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.2, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 2.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.4, ..attr(&knockback_resistance()) }));
        reg("spider", create_monster_attributes()
            .add(Attribute { default_value: 16.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) }));
        reg("cave_spider", create_monster_attributes()
            .add(Attribute { default_value: 12.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) }));
        reg("vex", create_flying_monster_attributes()
            .add(Attribute { default_value: 14.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.5, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 1.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 32.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 0.0, ..attr(&knockback_resistance()) }));
        reg("warden", create_monster_attributes()
            .add(Attribute { default_value: 500.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.3, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 30.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.99, ..attr(&knockback_resistance()) })
            .add(Attribute { default_value: 1.0, ..attr(&step_height()) }));
        reg("witch", create_monster_attributes()
            .add(Attribute { default_value: 26.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.25, ..attr(&movement_speed()) })
            .add(attr(&attack_damage()))
            .add(Attribute { default_value: 8.0, ..attr(&follow_range()) }));
        reg("wither", create_monster_attributes()
            .add(Attribute { default_value: 300.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.6, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 8.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 0.4, ..attr(&knockback_resistance()) })
            .add(Attribute { default_value: 128.0, ..attr(&follow_range()) }));
        reg("zoglin", create_monster_attributes()
            .add(Attribute { default_value: 40.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 1.5, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 6.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 1.0, ..attr(&knockback_resistance()) })
            .add(Attribute { default_value: 0.0, ..attr(&spawn_reinforcements()) }));
        reg("zombie", create_monster_attributes()
            .add(Attribute { default_value: 35.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 2.0, ..attr(&armor()) })
            .add(Attribute { default_value: 0.23, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 0.0, ..attr(&spawn_reinforcements()) }));
        reg("zombie_villager", create_monster_attributes()
            .add(Attribute { default_value: 35.0, ..attr(&follow_range()) })
            .add(Attribute { default_value: 3.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 2.0, ..attr(&armor()) })
            .add(Attribute { default_value: 0.23, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 0.0, ..attr(&spawn_reinforcements()) }));
        reg("zombified_piglin", create_monster_attributes()
            .add(Attribute { default_value: 20.0, ..attr(&max_health()) })
            .add(Attribute { default_value: 0.23, ..attr(&movement_speed()) })
            .add(Attribute { default_value: 5.0, ..attr(&attack_damage()) })
            .add(Attribute { default_value: 2.0, ..attr(&armor()) })
            .add(Attribute { default_value: 0.0, ..attr(&spawn_reinforcements()) }));

        m
    })
}

/// `DefaultAttributes.getSupplier(entityType)`.
pub fn get_supplier(entity_type: &Identifier) -> Option<&'static AttributeSupplier> {
    suppliers().get(entity_type)
}

/// `DefaultAttributes.hasSupplier(entityType)`.
pub fn has_supplier(entity_type: &Identifier) -> bool {
    suppliers().contains_key(entity_type)
}
