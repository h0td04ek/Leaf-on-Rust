//! Перенос `net.minecraft.world.item.enchantment.EnchantmentHelper` —
//! итерация по зачарованиям экипировки и вычисление эффектов.

use crate::core::holder::Holder;
use crate::entity::equipment_slot::EquipmentSlot;
use crate::entity::equipment_slot_group::EquipmentSlotGroup;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::entity::entity::Entity;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::item::enchantment::conditional_effect::ConditionalEffect;
use crate::world::item::enchantment::effects::enchantment_value_effect::EnchantmentValueEffect;
use crate::world::item::enchantment::enchantment::Enchantment;
use crate::world::item::enchantment::item_enchantments::ItemEnchantments;
use crate::world::item::item_stack::ItemStack;
use crate::world::level::storage::loot::loot_context::{LootContext, LootParams};
use crate::world::level::storage::loot::parameters::loot_context_params as params;
use crate::world::phys::vec3::Vec3;
use std::sync::Arc;

/// `ConditionalEffect<EnchantmentValueEffect>` — элемент effects-карты.
pub type DamageProtectionEffect = ConditionalEffect<EnchantmentValueEffect>;

/// id компонента `damage_protection` в `Enchantment.effects`.
pub fn damage_protection_component_id() -> Identifier {
    Identifier::minecraft("damage_protection")
}

/// `runIterationOnEquipment` — обобщённая итерация: для каждого
/// слота группы вызывает `f(stack, enchantments, level, holder)`.
/// Возвращает результат `f` (как `Optional`/`sum` в Java — здесь
/// суммирование через `acc`).
pub fn run_iteration_on_equipment<F>(
    entity: &LivingEntity,
    group: EquipmentSlotGroup,
    mut f: F,
) -> f32
where
    F: FnMut(&ItemStack, &ItemEnchantments, i32, &Holder<Enchantment>) -> f32,
{
    let mut acc = 0.0f32;
    for slot in EquipmentSlot::VALUES {
        if !group.test(slot) {
            continue;
        }
        let stack = entity.get_item_slot(slot);
        if stack.is_empty() {
            continue;
        }
        if let Some(enchantments) = stack
            .components
            .get_by_id::<ItemEnchantments>(&Identifier::minecraft("enchantments"))
        {
            for (holder, level) in &enchantments.enchantments {
                acc += f(stack, enchantments, *level, holder);
            }
        }
    }
    acc
}

/// `createEnchantedDamageContext`/`EnchantmentHelper` loot-context —
/// параметры как в Java `EnchantmentHelper#getDamageProtection`:
/// this_entity, damage_source, tool, origin, attacking/direct
/// attacking entity.
pub fn damage_context(
    entity: &LivingEntity,
    damage_source: &DamageSource,
    tool: &ItemStack,
) -> Option<LootContext> {
    let level = entity.entity.level.clone()?;
    let mut params = LootParams::new()
        .with_parameter(params::this_entity(), entity.entity.clone())
        .with_parameter(params::damage_source(), damage_source.clone())
        .with_parameter(params::tool(), tool.clone())
        .with_parameter(params::origin(), entity.entity.pos);
    if let Some(att) = damage_source.get_causing_entity() {
        params = params.with_parameter(params::attacking_entity(), att.clone());
    }
    if let Some(direct) = damage_source.get_direct_entity() {
        params = params.with_parameter(params::direct_attacking_entity(), direct.clone());
    }
    Some(LootContext::new(
        params,
        crate::random::legacy::create_unique(),
        Some(level),
    ))
}

/// `getDamageProtection` — сумма `EnchantmentValueEffect` из
/// `damage_protection` по armor-слотам с учётом условий.
pub fn get_damage_protection(entity: &LivingEntity, damage_source: &DamageSource) -> f32 {
    let mut total = 0.0f32;
    for slot in [
        EquipmentSlot::Head,
        EquipmentSlot::Chest,
        EquipmentSlot::Legs,
        EquipmentSlot::Feet,
    ] {
        let stack = entity.get_item_slot(slot);
        if stack.is_empty() {
            continue;
        }
        let Some(enchantments) = stack
            .components
            .get_by_id::<ItemEnchantments>(&Identifier::minecraft("enchantments"))
        else {
            continue;
        };
        for (holder, level) in &enchantments.enchantments {
            let Some(enchantment) = holder.value() else {
                continue;
            };
            let Some(effects) = enchantment
                .effects
                .get_by_id::<Vec<DamageProtectionEffect>>(&damage_protection_component_id())
            else {
                continue;
            };
            let Some(mut context) = damage_context(entity, damage_source, stack) else {
                continue;
            };
            for conditional in effects {
                if !conditional.matches(&context) {
                    continue;
                }
                total += conditional
                    .effect
                    .process(*level, context.random(), 0.0)
                    .abs();
            }
        }
    }
    total
}

/// `processDurabilityChange`/`processAmmoUse` и прочие —
/// обобщённый `runIterationOnEquipment` с эффектом по id
/// компонента (для будущих getters: getDamage, getKnockback и т.д.).
pub fn process_equipment_value_effects(
    entity: &LivingEntity,
    group: EquipmentSlotGroup,
    component_id: &Identifier,
    damage_source: Option<&DamageSource>,
    tool: Option<&ItemStack>,
    base_value: f32,
) -> f32 {
    let mut result = base_value;
    for slot in EquipmentSlot::VALUES {
        if !group.test(slot) {
            continue;
        }
        let stack = entity.get_item_slot(slot);
        if stack.is_empty() {
            continue;
        }
        let Some(enchantments) = stack
            .components
            .get_by_id::<ItemEnchantments>(&Identifier::minecraft("enchantments"))
        else {
            continue;
        };
        for (holder, level) in &enchantments.enchantments {
            let Some(enchantment) = holder.value() else {
                continue;
            };
            let Some(effects) = enchantment
                .effects
                .get_by_id::<Vec<DamageProtectionEffect>>(component_id)
            else {
                continue;
            };
            let context = damage_source
                .zip(tool)
                .and_then(|(s, t)| damage_context(entity, s, t));
            for conditional in effects {
                if let Some(ctx) = context.as_ref() {
                    if !conditional.matches(ctx) {
                        continue;
                    }
                }
                // Без контекста (нет level) — применяем без условий.
                if context.is_none() && conditional.requirements.is_some() {
                    continue;
                }
                let mut random = crate::random::legacy::create_unique();
                result = conditional.effect.process(*level, random.as_mut(), result);
            }
        }
    }
    result
}
