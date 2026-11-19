//! Методы эффектов `Enchantment` — порт методов из Java
//! `Enchantment.java` (effects dispatch).

use crate::resources::identifier::Identifier;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::item::enchantment::effects::enchantment_effect_components::{
    ids, EntityEffects, LocationEffects, TargetedEntityEffects, ValueEffects,
};
use crate::world::item::enchantment::effects::enchantment_target::EnchantmentTarget;
use crate::world::item::enchantment::effects::enchanted_item_in_use::EnchantedItemInUse;
use crate::world::item::enchantment::effects::targeted_conditional_effect::TargetedConditionalEffect;
use crate::world::item::enchantment::effects::enchantment_entity_effect::EnchantmentEntityEffect;
use crate::world::item::enchantment::enchantment::Enchantment;
use crate::world::item::item_stack::ItemStack;
use crate::world::phys::Vec3;

impl Enchantment {
    /// `getEffects(component)` — `effects.get(component)`.
    pub fn get_effects<T: 'static>(
        &self,
        component_id: &Identifier,
    ) -> Option<&T> {
        self.effects.get_by_id::<T>(component_id)
    }

    /// `matchingSlot(slot)`.
    pub fn matching_slot(&self, slot: crate::entity::equipment_slot::EquipmentSlot) -> bool {
        self.definition.slots.iter().any(|g| g.test(slot))
    }

    /// `modifyDamage`.
    #[allow(clippy::too_many_arguments)]
    pub fn modify_damage(
        &self,
        enchantment_level: i32,
        tool: &ItemStack,
        entity: &LivingEntity,
        damage_source: &DamageSource,
        damage: &mut f32,
    ) {
        self.modify_damage_filtered_value(
            &ids::damage(),
            enchantment_level,
            tool,
            entity,
            damage_source,
            damage,
        );
    }

    /// `modifyFallBasedDamage`.
    #[allow(clippy::too_many_arguments)]
    pub fn modify_fall_based_damage(
        &self,
        enchantment_level: i32,
        tool: &ItemStack,
        entity: &LivingEntity,
        damage_source: &DamageSource,
        fall_based_damage: &mut f32,
    ) {
        self.modify_damage_filtered_value(
            &ids::smash_damage_per_fallen_block(),
            enchantment_level,
            tool,
            entity,
            damage_source,
            fall_based_damage,
        );
    }

    /// `modifyKnockback`.
    #[allow(clippy::too_many_arguments)]
    pub fn modify_knockback(
        &self,
        enchantment_level: i32,
        tool: &ItemStack,
        entity: &LivingEntity,
        damage_source: &DamageSource,
        knockback: &mut f32,
    ) {
        self.modify_damage_filtered_value(
            &ids::knockback(),
            enchantment_level,
            tool,
            entity,
            damage_source,
            knockback,
        );
    }

    /// `modifyArmorEffectivness`.
    #[allow(clippy::too_many_arguments)]
    pub fn modify_armor_effectiveness(
        &self,
        enchantment_level: i32,
        tool: &ItemStack,
        entity: &LivingEntity,
        damage_source: &DamageSource,
        armor_effectiveness: &mut f32,
    ) {
        self.modify_damage_filtered_value(
            &ids::armor_effectiveness(),
            enchantment_level,
            tool,
            entity,
            damage_source,
            armor_effectiveness,
        );
    }

    /// `doPostAttack`.
    #[allow(clippy::too_many_arguments)]
    pub fn do_post_attack(
        &self,
        enchantment_level: i32,
        item: &mut EnchantedItemInUse,
        target: EnchantmentTarget,
        entity: &mut LivingEntity,
        damage_source: &DamageSource,
    ) {
        let Some(effects) = self.get_effects::<TargetedEntityEffects>(&ids::post_attack())
        else {
            return;
        };
        for targeted in effects {
            if target == targeted.enchanted {
                Self::do_post_attack_effect(
                    targeted,
                    enchantment_level,
                    item,
                    entity,
                    damage_source,
                );
            }
        }
    }

    /// `doPostAttack(TargetedConditionalEffect, ...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn do_post_attack_effect(
        effect: &TargetedConditionalEffect<EnchantmentEntityEffect>,
        enchantment_level: i32,
        item: &mut EnchantedItemInUse,
        entity: &mut LivingEntity,
        damage_source: &DamageSource,
    ) {
        let Some(level) = entity.entity.level.clone() else {
            return;
        };
        let Some(mut ctx) = crate::world::item::enchantment::enchantment_helper::damage_context_with_level(
            entity,
            enchantment_level,
            damage_source,
        ) else {
            return;
        };
        if !effect.matches(&mut ctx) {
            return;
        }
        let affected: Option<LivingEntity> = match effect.affected {
            EnchantmentTarget::Attacker => damage_source
                .get_causing_entity()
                .map(|e| LivingEntity::from_entity(e.clone())),
            EnchantmentTarget::DamagingEntity => damage_source
                .get_direct_entity()
                .map(|e| LivingEntity::from_entity(e.clone())),
            EnchantmentTarget::Victim => Some(entity.clone()),
        };
        if let Some(mut affected_entity) = affected {
            let origin = affected_entity.entity.position();
            effect.effect.apply(
                &level,
                enchantment_level,
                item,
                &mut affected_entity,
                origin,
            );
        }
    }

    /// `doLunge`.
    #[allow(clippy::too_many_arguments)]
    pub fn do_lunge(
        &self,
        enchantment_level: i32,
        item: &mut EnchantedItemInUse,
        entity: &mut LivingEntity,
    ) {
        let Some(effects) = self.get_effects::<EntityEffects>(&ids::post_piercing_attack())
        else {
            return;
        };
        if let Some(level) = entity.entity.level.clone() {
            let origin = entity.entity.position();
            for conditional in effects {
                if let Some(mut ctx) = crate::world::item::enchantment::enchantment_helper::entity_context_with_level(
                    entity,
                    enchantment_level,
                    origin,
                ) {
                    if !conditional.matches(&mut ctx) {
                        continue;
                    }
                    conditional.effect.apply(&level, enchantment_level, item, entity, origin);
                }
            }
        }
    }

    /// `tick`.
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &self,
        enchantment_level: i32,
        item: &mut EnchantedItemInUse,
        entity: &mut LivingEntity,
    ) {
        let Some(effects) = self.get_effects::<EntityEffects>(&ids::tick()) else {
            return;
        };
        if let Some(level) = entity.entity.level.clone() {
            let origin = entity.entity.position();
            for conditional in effects {
                if let Some(mut ctx) = crate::world::item::enchantment::enchantment_helper::entity_context_with_level(
                    entity,
                    enchantment_level,
                    origin,
                ) {
                    if !conditional.matches(&mut ctx) {
                        continue;
                    }
                    conditional.effect.apply(&level, enchantment_level, item, entity, origin);
                }
            }
        }
    }

    /// `onHitBlock`.
    #[allow(clippy::too_many_arguments)]
    pub fn on_hit_block(
        &self,
        enchantment_level: i32,
        item: &mut EnchantedItemInUse,
        entity: &mut LivingEntity,
        pos: Vec3,
        _state: &crate::world::level::block::state::block_state::BlockState,
    ) {
        let Some(effects) = self.get_effects::<EntityEffects>(&ids::hit_block()) else {
            return;
        };
        if let Some(level) = entity.entity.level.clone() {
            for conditional in effects {
                if conditional.requirements.is_some() {
                    // block_hit_context требует block_state параметр —
                    // строится отдельным контекстом.
                    continue;
                }
                conditional.effect.apply(&level, enchantment_level, item, entity, pos);
            }
        }
    }

    /// `runLocationChangedEffects`.
    #[allow(clippy::too_many_arguments)]
    pub fn run_location_changed_effects(
        &self,
        enchantment_level: i32,
        item: &mut EnchantedItemInUse,
        entity: &mut LivingEntity,
        pos: Vec3,
        apply_transient_effects: bool,
    ) {
        let Some(effects) = self.get_effects::<LocationEffects>(&ids::location_changed())
        else {
            return;
        };
        if let Some(level) = entity.entity.level.clone() {
            for conditional in effects {
                if let Some(mut ctx) = crate::world::item::enchantment::enchantment_helper::location_context_with_level(
                    entity,
                    enchantment_level,
                    pos,
                ) {
                    if !conditional.matches(&mut ctx) {
                        continue;
                    }
                    conditional.effect.on_changed_block(
                        &level,
                        enchantment_level,
                        item,
                        entity,
                        pos,
                        apply_transient_effects,
                    );
                }
            }
        }
    }

    /// `stopLocationBasedEffects`.
    #[allow(clippy::too_many_arguments)]
    pub fn stop_location_based_effects(
        &self,
        item: &EnchantedItemInUse,
        entity: &mut LivingEntity,
        pos: Vec3,
        enchantment_level: i32,
    ) {
        let Some(effects) = self.get_effects::<LocationEffects>(&ids::location_changed())
        else {
            return;
        };
        for conditional in effects {
            conditional.effect.on_deactivated(item, entity, pos, enchantment_level);
        }
    }

    /// `modifyItemFilteredCount` — приватная в Java; обёртка `pub`.
    #[allow(clippy::too_many_arguments)]
    pub fn modify_item_filtered_count(
        &self,
        component: &Identifier,
        enchantment_level: i32,
        tool: &ItemStack,
        value: &mut f32,
    ) {
        let Some(effects) = self.get_effects::<ValueEffects>(component) else {
            return;
        };
        let mut random = crate::random::legacy::create_unique();
        for conditional in effects {
            if let Some(mut ctx) = crate::world::item::enchantment::enchantment_helper::item_context(
                enchantment_level,
                tool,
            ) {
                if !conditional.matches(&mut ctx) {
                    continue;
                }
                *value = conditional.effect.process(enchantment_level, random.as_mut(), *value);
            }
        }
    }

    /// `modifyEntityFilteredValue`.
    #[allow(clippy::too_many_arguments)]
    pub fn modify_entity_filtered_value(
        &self,
        component: &Identifier,
        enchantment_level: i32,
        _tool: &ItemStack,
        entity: &LivingEntity,
        value: &mut f32,
    ) {
        let Some(effects) = self.get_effects::<ValueEffects>(component) else {
            return;
        };
        let mut random = entity.entity_random();
        let origin = entity.entity.position();
        for conditional in effects {
            if let Some(mut ctx) = crate::world::item::enchantment::enchantment_helper::entity_context_with_level(
                entity,
                enchantment_level,
                origin,
            ) {
                if !conditional.matches(&mut ctx) {
                    continue;
                }
                let mut borrow = random.lock().expect("entity random poisoned");
                *value = conditional
                    .effect
                    .process(enchantment_level, &mut **borrow, *value);
            }
        }
    }

    /// `modifyDamageFilteredValue`.
    #[allow(clippy::too_many_arguments)]
    fn modify_damage_filtered_value(
        &self,
        component: &Identifier,
        enchantment_level: i32,
        tool: &ItemStack,
        entity: &LivingEntity,
        damage_source: &DamageSource,
        value: &mut f32,
    ) {
        let Some(effects) = self.get_effects::<ValueEffects>(component) else {
            return;
        };
        let mut random = entity.entity_random();
        for conditional in effects {
            if let Some(mut ctx) = crate::world::item::enchantment::enchantment_helper::damage_context_with_level(
                entity,
                enchantment_level,
                damage_source,
            ) {
                if !conditional.matches(&mut ctx) {
                    continue;
                }
                let mut borrow = random.lock().expect("entity random poisoned");
                *value = conditional
                    .effect
                    .process(enchantment_level, &mut **borrow, *value);
            }
        }
    }

    /// `modifyUnfilteredValue`.
    pub fn modify_unfiltered_value(
        &self,
        component: &Identifier,
        random: &mut dyn crate::random::RandomSource,
        enchantment_level: i32,
        value: &mut f32,
    ) {
        if let Some(effect) = self.effects.get_by_id::<
            crate::world::item::enchantment::effects::enchantment_value_effect::EnchantmentValueEffect,
        >(component)
        {
            *value = effect.process(enchantment_level, random, *value);
        }
    }

    /// `modifyProjectileCount`.
    #[allow(clippy::too_many_arguments)]
    pub fn modify_projectile_count(
        &self,
        enchantment_level: i32,
        tool: &ItemStack,
        entity: &LivingEntity,
        projectile_count: &mut f32,
    ) {
        self.modify_entity_filtered_value(
            &ids::projectile_count(),
            enchantment_level,
            tool,
            entity,
            projectile_count,
        );
    }

    /// `modifyProjectileSpread`.
    #[allow(clippy::too_many_arguments)]
    pub fn modify_projectile_spread(
        &self,
        enchantment_level: i32,
        tool: &ItemStack,
        entity: &LivingEntity,
        projectile_spread: &mut f32,
    ) {
        self.modify_entity_filtered_value(
            &ids::projectile_spread(),
            enchantment_level,
            tool,
            entity,
            projectile_spread,
        );
    }

    /// `modifyCrossbowChargeTime`.
    pub fn modify_crossbow_charge_time(
        &self,
        random: &mut dyn crate::random::RandomSource,
        enchantment_level: i32,
        value: &mut f32,
    ) {
        self.modify_unfiltered_value(
            &ids::crossbow_charge_time(),
            random,
            enchantment_level,
            value,
        );
    }

    /// `onProjectileSpawned`.
    #[allow(clippy::too_many_arguments)]
    pub fn on_projectile_spawned(
        &self,
        enchantment_level: i32,
        item: &mut EnchantedItemInUse,
        entity: &mut LivingEntity,
    ) {
        let Some(effects) = self.get_effects::<EntityEffects>(&ids::projectile_spawned())
        else {
            return;
        };
        if let Some(level) = entity.entity.level.clone() {
            let origin = entity.entity.position();
            for conditional in effects {
                if let Some(mut ctx) = crate::world::item::enchantment::enchantment_helper::entity_context_with_level(
                    entity,
                    enchantment_level,
                    origin,
                ) {
                    if !conditional.matches(&mut ctx) {
                        continue;
                    }
                    conditional.effect.apply(&level, enchantment_level, item, entity, origin);
                }
            }
        }
    }
}
