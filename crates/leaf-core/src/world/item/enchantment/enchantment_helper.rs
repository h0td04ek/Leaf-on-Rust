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
                if !conditional.matches(&mut context) {
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
            let mut context = damage_source
                .zip(tool)
                .and_then(|(s, t)| damage_context(entity, s, t));
            for conditional in effects {
                if let Some(ctx) = context.as_mut() {
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

/// `EnchantmentHelper.getItemEnchantmentLevel(enchantment, stack)`.
pub fn get_item_enchantment_level(
    enchantment: &Holder<Enchantment>,
    stack: &ItemStack,
) -> i32 {
    stack
        .components
        .get_by_id::<ItemEnchantments>(&Identifier::minecraft("enchantments"))
        .map(|e| e.get_level(enchantment))
        .unwrap_or(0)
}

/// `EnchantmentHelper.getEnchantmentLevel(enchantment, livingEntity)` —
/// sum over all equipment slots.
pub fn get_enchantment_level(
    enchantment: &Holder<Enchantment>,
    entity: &LivingEntity,
) -> i32 {
    let mut level = 0;
    for slot in EquipmentSlot::VALUES {
        let stack = entity.get_item_slot(slot);
        if stack.is_empty() {
            continue;
        }
        level += get_item_enchantment_level(enchantment, stack);
    }
    level
}


// ---- Статические контексты `Enchantment` (damageContext, entityContext,
// itemContext, locationContext, blockHitContext) ----

use crate::world::phys::vec3::Vec3 as Vec3Type;

/// `Enchantment.damageContext(level, enchantmentLevel, entity, damageSource)`.
/// Параметры: THIS_ENTITY, ENCHANTMENT_LEVEL, ORIGIN,
/// DAMAGE_SOURCE, ATTACKING_ENTITY?, DIRECT_ATTACKING_ENTITY?.
pub fn damage_context_with_level(
    entity: &LivingEntity,
    enchantment_level: i32,
    damage_source: &DamageSource,
) -> Option<LootContext> {
    let level = entity.entity.level.clone()?;
    let mut params = LootParams::new()
        .with_parameter(params::this_entity(), entity.entity.clone())
        .with_parameter(params::enchantment_level(), enchantment_level)
        .with_parameter(params::origin(), entity.entity.pos)
        .with_parameter(params::damage_source(), damage_source.clone());
    if let Some(att) = damage_source.get_causing_entity() {
        params = params.with_optional_parameter(params::attacking_entity(), Some(att.clone()));
    }
    if let Some(direct) = damage_source.get_direct_entity() {
        params = params.with_optional_parameter(params::direct_attacking_entity(), Some(direct.clone()));
    }
    Some(LootContext::new(params, crate::random::legacy::create_unique(), Some(level)))
}

/// `Enchantment.entityContext(level, enchantmentLevel, entity, origin)`.
pub fn entity_context_with_level(
    entity: &LivingEntity,
    enchantment_level: i32,
    origin: Vec3Type,
) -> Option<LootContext> {
    let level = entity.entity.level.clone()?;
    let params = LootParams::new()
        .with_parameter(params::this_entity(), entity.entity.clone())
        .with_parameter(params::enchantment_level(), enchantment_level)
        .with_parameter(params::origin(), origin);
    Some(LootContext::new(params, crate::random::legacy::create_unique(), Some(level)))
}

/// `Enchantment.locationContext(level, enchantmentLevel, entity, active)`.
pub fn location_context_with_level(
    entity: &LivingEntity,
    enchantment_level: i32,
    pos: Vec3Type,
) -> Option<LootContext> {
    let level = entity.entity.level.clone()?;
    let params = LootParams::new()
        .with_parameter(params::this_entity(), entity.entity.clone())
        .with_parameter(params::enchantment_level(), enchantment_level)
        .with_parameter(params::origin(), pos)
        .with_parameter(params::enchantment_active(), true);
    Some(LootContext::new(params, crate::random::legacy::create_unique(), Some(level)))
}

/// `Enchantment.blockHitContext(level, enchantmentLevel, entity, origin, state)`.
pub fn block_hit_context_with_level(
    entity: &LivingEntity,
    enchantment_level: i32,
    origin: Vec3Type,
    state: &crate::world::level::block::state::block_state::BlockState,
) -> Option<LootContext> {
    let level = entity.entity.level.clone()?;
    let params = LootParams::new()
        .with_parameter(params::this_entity(), entity.entity.clone())
        .with_parameter(params::enchantment_level(), enchantment_level)
        .with_parameter(params::origin(), origin)
        .with_parameter(params::block_state(), state.clone());
    Some(LootContext::new(params, crate::random::legacy::create_unique(), Some(level)))
}

/// `Enchantment.itemContext(level, enchantmentLevel, tool)` — без уровня
/// (TOOL + ENCHANTMENT_LEVEL; уровень неизвестен → None).
pub fn item_context(
    enchantment_level: i32,
    tool: &ItemStack,
) -> Option<LootContext> {
    let params = LootParams::new()
        .with_parameter(params::tool(), tool.clone())
        .with_parameter(params::enchantment_level(), enchantment_level);
    Some(LootContext::new(params, crate::random::legacy::create_unique(), None))
}


// ---- Итерация по зачарованиям одного предмета
// (`EnchantmentHelper.runIterationOnItem`) и entity-wide
// модификаторы (modifyDamage/modifyKnockback/doPostAttack/
// doLunge/tick). ----

use crate::world::item::enchantment::effects::enchanted_item_in_use::EnchantedItemInUse;
use crate::world::item::enchantment::effects::enchantment_target::EnchantmentTarget;
use crate::entity::equipment_slot::EquipmentSlot as Slot;

/// `runIterationOnItem(stack, visitor)` — для каждого
/// зачарования вызывает `f(enchantment, level)`.
pub fn run_iteration_on_item<F>(stack: &ItemStack, mut f: F)
where
    F: FnMut(&Enchantment, i32),
{
    let Some(enchantments) = stack
        .components
        .get_by_id::<ItemEnchantments>(&Identifier::minecraft("enchantments"))
    else {
        return;
    };
    for (holder, level) in &enchantments.enchantments {
        if let Some(enchantment) = holder.value() {
            f(enchantment, *level);
        }
    }
}

/// `EnchantmentHelper.modifyDamage(level, tool, entity, source, damage)`.
/// `entity` — цель (victim), не атакующий.
pub fn modify_damage(
    tool: &ItemStack,
    entity: &LivingEntity,
    damage_source: &DamageSource,
    damage: f32,
) -> f32 {
    let mut result = damage;
    run_iteration_on_item(tool, |enchantment, level| {
        enchantment.modify_damage(level, tool, entity, damage_source, &mut result);
    });
    result
}

/// `EnchantmentHelper.modifyKnockback`.
pub fn modify_knockback(
    tool: &ItemStack,
    entity: &LivingEntity,
    damage_source: &DamageSource,
    knockback: f32,
) -> f32 {
    let mut result = knockback;
    run_iteration_on_item(tool, |enchantment, level| {
        enchantment.modify_knockback(level, tool, entity, damage_source, &mut result);
    });
    result
}

/// `EnchantmentHelper.modifyArmorEffectiveness`.
pub fn modify_armor_effectiveness(
    tool: &ItemStack,
    entity: &LivingEntity,
    damage_source: &DamageSource,
    effectiveness: f32,
) -> f32 {
    let mut result = effectiveness;
    run_iteration_on_item(tool, |enchantment, level| {
        enchantment.modify_armor_effectiveness(
            level,
            tool,
            entity,
            damage_source,
            &mut result,
        );
    });
    result
}

/// `EnchantmentHelper.doPostAttackEffects(level, entity, damageSource)` —
/// `entity` — получившая урон сущность (victim). Пост-эффекты
/// снаряжения жертвы (target=VICTIM) + оружия атакующего
/// (target=ATTACKER).
pub fn do_post_attack_effects(
    entity: &mut LivingEntity,
    damage_source: &DamageSource,
) {
    // Атакующий — causing entity источника урона.
    let attacker = damage_source
        .get_causing_entity()
        .map(|e| LivingEntity::from_entity(e.clone()));
    let weapon = attacker
        .as_ref()
        .map(|a| a.get_weapon_item())
        .unwrap_or_else(ItemStack::empty);

    // 1) Экипировка жертвы — эффекты на VICTIM.
    for slot in Slot::VALUES {
        let stack = entity.get_item_by_slot(slot).clone();
        if stack.is_empty() {
            continue;
        }
        let owner = Some(entity.clone());
        let mut item_in_use =
            EnchantedItemInUse::new(stack.clone(), Some(slot), owner);
        run_iteration_on_item(&stack, |enchantment, level| {
            enchantment.do_post_attack(
                level,
                &mut item_in_use,
                EnchantmentTarget::Victim,
                entity,
                damage_source,
            );
        });
    }

    // 2) Оружие атакующего — эффекты на ATTACKER.
    if !weapon.is_empty() {
        if let Some(attacker_living) = attacker {
            let mut attacker_mut = attacker_living;
            let mut item_in_use =
                EnchantedItemInUse::new(weapon.clone(), Some(Slot::Mainhand), Some(attacker_mut.clone()));
            run_iteration_on_item(&weapon, |enchantment, level| {
                enchantment.do_post_attack(
                    level,
                    &mut item_in_use,
                    EnchantmentTarget::Attacker,
                    &mut attacker_mut,
                    damage_source,
                );
            });
        }
    }
}

/// `EnchantmentHelper.doLungeEffects(level, entity)` —
/// POST_PIERCING_ATTACK эффекты оружия.
pub fn do_lunge_effects(entity: &mut LivingEntity) {
    let weapon = entity.get_weapon_item();
    if weapon.is_empty() {
        return;
    }
    let owner = Some(entity.clone());
    let mut item_in_use =
        EnchantedItemInUse::new(weapon.clone(), Some(Slot::Mainhand), owner);
    run_iteration_on_item(&weapon, |enchantment, level| {
        enchantment.do_lunge(level, &mut item_in_use, entity);
    });
}

/// `EnchantmentHelper.tickEffects(level, entity)` — TICK-эффекты
/// всей экипировки.
pub fn tick_effects(entity: &mut LivingEntity) {
    for slot in Slot::VALUES {
        let stack = entity.get_item_by_slot(slot).clone();
        if stack.is_empty() {
            continue;
        }
        let owner = Some(entity.clone());
        let mut item_in_use =
            EnchantedItemInUse::new(stack.clone(), Some(slot), owner);
        run_iteration_on_item(&stack, |enchantment, level| {
            enchantment.tick(level, &mut item_in_use, entity);
        });
    }
}
