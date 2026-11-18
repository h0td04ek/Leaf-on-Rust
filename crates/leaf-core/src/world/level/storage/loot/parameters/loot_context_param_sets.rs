//! Перенос `net.minecraft.world.level.storage.loot.parameters.LootContextParamSets`.

use crate::util::context::context_key_set::ContextKeySet;
use crate::world::level::storage::loot::parameters::loot_context_params::*;

/// `LootContextParamSets.paramSet(name)` — набор допустимых параметров
/// по имени набора (как в реестре `LOOT_CONTEXT_PARAM_SET`).
pub fn param_set(name: &str) -> Option<ContextKeySet> {
    match name {
        "empty" => Some(ContextKeySet::builder().build()),
        "chest" => {
            let mut b = ContextKeySet::builder();
            b.optional(this_entity()).optional(interacting_entity()).optional(tool());
            Some(b.build())
        }
        "command" | "selector" => {
            let mut b = ContextKeySet::builder();
            b.required(origin());
            Some(b.build())
        }
        "fish" => {
            let mut b = ContextKeySet::builder();
            b.required(tool()).required(origin());
            Some(b.build())
        }
        "entity" => {
            let mut b = ContextKeySet::builder();
            b.required(this_entity());
            Some(b.build())
        }
        "equipment" => {
            let mut b = ContextKeySet::builder();
            b.required(this_entity()).optional(interacting_entity()).required(tool());
            Some(b.build())
        }
        "block_interact" => {
            let mut b = ContextKeySet::builder();
            b.required(block_state())
                .optional(origin())
                .optional(interacting_entity())
                .optional(tool());
            Some(b.build())
        }
        "enchanted_damage" => {
            let mut b = ContextKeySet::builder();
            b.required(this_entity())
                .required(enchantment_level())
                .required(origin())
                .required(damage_source())
                .optional(direct_attacking_entity())
                .optional(attacking_entity());
            Some(b.build())
        }
        "enchanted_item" => {
            let mut b = ContextKeySet::builder();
            b.required(tool()).required(enchantment_level());
            Some(b.build())
        }
        "enchanted_location" => {
            let mut b = ContextKeySet::builder();
            b.required(this_entity())
                .required(enchantment_level())
                .required(origin())
                .required(enchantment_active());
            Some(b.build())
        }
        "enchanted_entity" => {
            let mut b = ContextKeySet::builder();
            b.required(this_entity())
                .required(enchantment_level())
                .required(origin());
            Some(b.build())
        }
        "hit_block" => {
            let mut b = ContextKeySet::builder();
            b.required(this_entity())
                .required(enchantment_level())
                .required(origin())
                .required(block_state());
            Some(b.build())
        }
        "advancement_reward" | "advancement_entity" | "advancement_location" => {
            let mut b = ContextKeySet::builder();
            b.optional(this_entity()).optional(target_entity()).optional(interacting_entity()).optional(tool());
            Some(b.build())
        }
        "advancement_kill" => {
            let mut b = ContextKeySet::builder();
            b.required(this_entity()).optional(attacking_entity()).optional(direct_attacking_entity()).optional(last_damage_player());
            Some(b.build())
        }
        "advancement_reward_entity" => {
            let mut b = ContextKeySet::builder();
            b.required(this_entity());
            Some(b.build())
        }
        "barter" | "gift" | "piglin_barter" => {
            let mut b = ContextKeySet::builder();
            b.required(this_entity()).optional(origin());
            Some(b.build())
        }
        "vault" => {
            let mut b = ContextKeySet::builder();
            b.required(this_entity());
            Some(b.build())
        }
        "all_params" => {
            let mut b = ContextKeySet::builder();
            b.optional(this_entity())
                .optional(interacting_entity())
                .optional(target_entity())
                .optional(last_damage_player())
                .optional(damage_source())
                .optional(attacking_entity())
                .optional(direct_attacking_entity())
                .optional(origin())
                .optional(block_state())
                .optional(tool())
                .optional(explosion_radius())
                .optional(enchantment_level())
                .optional(enchantment_active());
            Some(b.build())
        }
        _ => None,
    }
}
