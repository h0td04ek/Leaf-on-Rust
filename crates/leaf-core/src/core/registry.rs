//! Перенос `net.minecraft.core.Registry` — на данном этапе только
//! типовая метка и константы имён реестров (`Registries`).
//! Полная реализация маппингов/holders — следующий этап.

use crate::resources::Identifier;
use std::marker::PhantomData;

pub struct Registry<T> {
    pub _phantom: PhantomData<fn() -> T>,
}

/// `net.minecraft.core.registries.Registries` — константы имён реестров.
pub mod registries {
    use crate::resources::Identifier;

    /// `Registries.ROOT_REGISTRY_NAME`
    pub fn root_registry_name() -> Identifier {
        Identifier::minecraft("minecraft")
    }

    pub fn block() -> Identifier {
        Identifier::minecraft("block")
    }
    pub fn item() -> Identifier {
        Identifier::minecraft("item")
    }
    pub fn entity_type() -> Identifier {
        Identifier::minecraft("entity_type")
    }
    pub fn menu() -> Identifier {
        Identifier::minecraft("menu")
    }
    pub fn sound_event() -> Identifier {
        Identifier::minecraft("sound_event")
    }
    pub fn mob_effect() -> Identifier {
        Identifier::minecraft("mob_effect")
    }
    pub fn dimension_type() -> Identifier {
        Identifier::minecraft("dimension_type")
    }
    pub fn dimension() -> Identifier {
        Identifier::minecraft("dimension")
    }
    pub fn biome() -> Identifier {
        Identifier::minecraft("worldgen/biome")
    }
    pub fn configured_carver() -> Identifier {
        Identifier::minecraft("worldgen/configured_carver")
    }
    pub fn placed_feature() -> Identifier {
        Identifier::minecraft("worldgen/placed_feature")
    }
    pub fn structure() -> Identifier {
        Identifier::minecraft("worldgen/structure")
    }
    pub fn structure_set() -> Identifier {
        Identifier::minecraft("worldgen/structure_set")
    }
    pub fn enchantment() -> Identifier {
        Identifier::minecraft("enchantment")
    }
    pub fn damage_type() -> Identifier {
        Identifier::minecraft("damage_type")
    }
    pub fn trim_pattern() -> Identifier {
        Identifier::minecraft("trim_pattern")
    }
    pub fn trim_material() -> Identifier {
        Identifier::minecraft("trim_material")
    }
    pub fn banner_pattern() -> Identifier {
        Identifier::minecraft("banner_pattern")
    }
    pub fn instrument() -> Identifier {
        Identifier::minecraft("instrument")
    }
    pub fn painting_variant() -> Identifier {
        Identifier::minecraft("painting_variant")
    }
    pub fn wolf_variant() -> Identifier {
        Identifier::minecraft("wolf_variant")
    }
    pub fn wolf_sound_variant() -> Identifier {
        Identifier::minecraft("wolf_sound_variant")
    }
    pub fn cat_variant() -> Identifier {
        Identifier::minecraft("cat_variant")
    }
    pub fn cat_sound_variant() -> Identifier {
        Identifier::minecraft("cat_sound_variant")
    }
    pub fn chicken_variant() -> Identifier {
        Identifier::minecraft("chicken_variant")
    }
    pub fn cow_variant() -> Identifier {
        Identifier::minecraft("cow_variant")
    }
    pub fn frog_variant() -> Identifier {
        Identifier::minecraft("frog_variant")
    }
    pub fn pig_variant() -> Identifier {
        Identifier::minecraft("pig_variant")
    }
    pub fn wolf_sound_variant_legacy() -> Identifier {
        Identifier::minecraft("wolf_sound_variant")
    }
    pub fn armadillo_sound_variant() -> Identifier {
        Identifier::minecraft("armadillo_sound_variant")
    }
    pub fn armadillo_variant() -> Identifier {
        Identifier::minecraft("armadillo_variant")
    }
    pub fn trial_spawner() -> Identifier {
        Identifier::minecraft("trial_spawner")
    }
    pub fn trial_chamber() -> Identifier {
        Identifier::minecraft("trial_chamber")
    }
    pub fn point_of_interest_type() -> Identifier {
        Identifier::minecraft("point_of_interest_type")
    }
    pub fn memory_module_type() -> Identifier {
        Identifier::minecraft("memory_module_type")
    }
    pub fn sensor_type() -> Identifier {
        Identifier::minecraft("sensor_type")
    }
    pub fn schedule() -> Identifier {
        Identifier::minecraft("schedule")
    }
    pub fn activity() -> Identifier {
        Identifier::minecraft("activity")
    }
    pub fn world_clock() -> Identifier {
        Identifier::minecraft("world_clock")
    }
    pub fn loot_pool_entry_type() -> Identifier {
        Identifier::minecraft("loot_pool_entry_type")
    }
    pub fn loot_function_type() -> Identifier {
        Identifier::minecraft("loot_function_type")
    }
    pub fn loot_condition_type() -> Identifier {
        Identifier::minecraft("loot_condition_type")
    }
    pub fn loot_number_provider_type() -> Identifier {
        Identifier::minecraft("loot_number_provider_type")
    }
    pub fn loot_score_provider_type() -> Identifier {
        Identifier::minecraft("loot_score_provider_type")
    }
    pub fn float_provider_type() -> Identifier {
        Identifier::minecraft("float_provider_type")
    }
    pub fn argument_type_info() -> Identifier {
        Identifier::minecraft("command_argument_type")
    }
    pub fn test_environment_definition() -> Identifier {
        Identifier::minecraft("test_environment")
    }
    pub fn test_instance_type() -> Identifier {
        Identifier::minecraft("test_instance_type")
    }
    pub fn game_event() -> Identifier {
        Identifier::minecraft("game_event")
    }
    pub fn recipe_type() -> Identifier {
        Identifier::minecraft("recipe_type")
    }
    pub fn recipe_serializer() -> Identifier {
        Identifier::minecraft("recipe_serializer")
    }
    pub fn attribute() -> Identifier {
        Identifier::minecraft("attribute")
    }
    pub fn stats() -> Identifier {
        Identifier::minecraft("stats")
    }
    pub fn custom_stat() -> Identifier {
        Identifier::minecraft("custom_stat")
    }
    pub fn dialog_type() -> Identifier {
        Identifier::minecraft("dialog_type")
    }
    pub fn position_source_type() -> Identifier {
        Identifier::minecraft("position_source_type")
    }
    pub fn particle_type() -> Identifier {
        Identifier::minecraft("particle_type")
    }
    pub fn block_state_predicate_type() -> Identifier {
        Identifier::minecraft("block_state_predicate_type")
    }
    pub fn rule_block_entity_modifier() -> Identifier {
        Identifier::minecraft("rule_block_entity_modifier")
    }
    pub fn rule_test() -> Identifier {
        Identifier::minecraft("rule_test")
    }
    pub fn decorator() -> Identifier {
        Identifier::minecraft("decorator")
    }
    pub fn material_condition() -> Identifier {
        Identifier::minecraft("material_condition")
    }
    pub fn material_rule() -> Identifier {
        Identifier::minecraft("material_rule")
    }
    pub fn map_decoration_type() -> Identifier {
        Identifier::minecraft("map_decoration_type")
    }
    pub fn chunk_status() -> Identifier {
        Identifier::minecraft("chunk_status")
    }
    pub fn worldgen_height_provider_type() -> Identifier {
        Identifier::minecraft("worldgen/height_provider_type")
    }
    pub fn worldgen_block_predicate_type() -> Identifier {
        Identifier::minecraft("worldgen/block_predicate_type")
    }
    pub fn worldgen_foliage_placer_type() -> Identifier {
        Identifier::minecraft("worldgen/foliage_placer_type")
    }
    pub fn worldgen_tree_decorator_type() -> Identifier {
        Identifier::minecraft("worldgen/tree_decorator_type")
    }
    pub fn worldgen_trunk_placer_type() -> Identifier {
        Identifier::minecraft("worldgen/trunk_placer_type")
    }
    pub fn worldgen_feature() -> Identifier {
        Identifier::minecraft("worldgen/feature")
    }
    pub fn worldgen_noise_settings() -> Identifier {
        Identifier::minecraft("worldgen/noise_settings")
    }
    pub fn worldgen_density_function_type() -> Identifier {
        Identifier::minecraft("worldgen/density_function_type")
    }
    pub fn worldgen_noise() -> Identifier {
        Identifier::minecraft("worldgen/noise")
    }
    pub fn worldgen_surface_rule() -> Identifier {
        Identifier::minecraft("worldgen/surface_rule")
    }
    pub fn worldgen_carver() -> Identifier {
        Identifier::minecraft("worldgen/carver")
    }
    pub fn worldgen_placement_modifier_type() -> Identifier {
        Identifier::minecraft("worldgen/placement_modifier_type")
    }
    pub fn worldgen_structure_placement_type() -> Identifier {
        Identifier::minecraft("worldgen/structure_placement")
    }
    pub fn worldgen_structure_piece() -> Identifier {
        Identifier::minecraft("worldgen/structure_piece")
    }
    pub fn worldgen_structure_type() -> Identifier {
        Identifier::minecraft("worldgen/structure")
    }
    pub fn worldgen_template_pool() -> Identifier {
        Identifier::minecraft("worldgen/template_pool")
    }
    pub fn banner_pattern_legacy() -> Identifier {
        Identifier::minecraft("banner_pattern")
    }
}

/// `Registries.ROOT_REGISTRY_NAME` — как функция (Identifier не const).
pub fn root_registry_name() -> Identifier {
    registries::root_registry_name()
}
