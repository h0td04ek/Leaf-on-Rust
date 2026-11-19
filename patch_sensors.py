import io

p = 'crates/leaf-core/src/world/entity/living_entity.rs'
s = io.open(p, encoding='utf-8').read()
old = '''    /// `entity.getType()` — EntityType за Holder.
    pub fn entity_type_value(&self) -> crate::world::entity::entity_type::EntityType {
        self.entity.entity_type.value().cloned().unwrap_or_else(|| {
            crate::world::entity::entity_type::EntityType::empty()
        })
    }'''
new = '''    /// `entity.getType()` — EntityType за Holder (None если
    /// holder пуст — эквивалент отсутствия типа).
    pub fn entity_type_value(&self) -> Option<&crate::world::entity::entity_type::EntityType> {
        self.entity.entity_type.value()
    }'''
assert old in s, 'entity_type_value'
s = s.replace(old, new)
io.open(p, 'w', encoding='utf-8').write(s)

p = 'crates/leaf-core/src/world/entity/ai/sensing/sensors.rs'
s = io.open(p, encoding='utf-8').read()

s = s.replace('''                |_level, _owner, target| {
                    crate::tags::entity_type_tags::entity_type_is(
                        &target.entity_type_value(),
                        crate::tags::entity_type_tags::keys::followable_friendly_mobs(),
                    ) && !target.entity.is_baby()
                },''', '''                |_level, _owner, target| {
                    target
                        .entity_type_value()
                        .map(|t| {
                            crate::tags::entity_type_tags::entity_type_is(
                                t,
                                crate::tags::entity_type_tags::keys::followable_friendly_mobs(),
                            )
                        })
                        .unwrap_or(false)
                        && !target.entity.is_baby()
                },''')

s = s.replace('''fn entity_type_tag_check(entity: &LivingEntity, tag: &str) -> bool {
    let tag_key = match tag {
        "axolotl_hunt_targets" => crate::tags::entity_type_tags::keys::axolotl_hunt_targets(),
        "axolotl_always_hostiles" => crate::tags::entity_type_tags::keys::axolotl_always_hostiles(),
        _ => return false,
    };
    crate::tags::entity_type_tags::entity_type_is(&entity.entity_type_value(), tag_key)
}''', '''fn entity_type_tag_check(entity: &LivingEntity, tag: &str) -> bool {
    let Some(t) = entity.entity_type_value() else { return false };
    let tag_key = match tag {
        "axolotl_hunt_targets" => crate::tags::entity_type_tags::keys::axolotl_hunt_targets(),
        "axolotl_always_hostiles" => crate::tags::entity_type_tags::keys::axolotl_always_hostiles(),
        _ => return false,
    };
    crate::tags::entity_type_tags::entity_type_is(t, tag_key)
}''')

s = s.replace('''            } else if zombified.is_none()
                && LivingEntity::piglin_ai_is_zombified_type(&target.entity.entity_type.value().id)
            {
                zombified = Some(target.clone());
            }''', '''            } else if zombified.is_none()
                && target
                    .entity
                    .entity_type
                    .value()
                    .map(|t| LivingEntity::piglin_ai_is_zombified_type(&t.id))
                    .unwrap_or(false)
            {
                zombified = Some(target.clone());
            }''')

s = s.replace('''            match candidate {
                Some(poi) => {
                    mob.living.brain_set_value(&keys::nearest_bed(), poi);
                }''', '''            match candidate {
                Some(poi) => {
                    // Java хранит GlobalPos(dimension, pos).
                    let gpos = level.dimension_id().map(|dim| {
                        crate::core::global_pos::GlobalPos::of(
                            crate::resources::ResourceKey::create_with_registry_name(
                                crate::core::registry::registries::dimension(),
                                dim.clone(),
                            ),
                            poi,
                        )
                    });
                    mob.living.brain_set(&keys::nearest_bed(), gpos);
                }''')

s = s.replace('''                            list.push(crate::core::global_pos::GlobalPos::new(
                                dim.clone(),
                                pos,
                            ));''', '''                            list.push(crate::core::global_pos::GlobalPos::of(
                                dim.clone(),
                                pos,
                            ));''')

s = s.replace('''        let dim = level.dimension_id().cloned();
        let Some(dim) = dim else {
            mob.living.brain_erase(&keys::secondary_job_site());
            return;
        };''', '''        let dim_id = level.dimension_id().cloned();
        let Some(dim_id) = dim_id else {
            mob.living.brain_erase(&keys::secondary_job_site());
            return;
        };
        let dim = crate::resources::ResourceKey::<crate::world::level::Level>::create_with_registry_name(
            crate::core::registry::registries::dimension(),
            dim_id,
        );''')

# TemptingSensor item id checks -> item_identifier()
import re
start = s.index('    /// `GOAT_TEMPTATIONS`')
end = s.index('    /// `playerHoldingTemptation')
new_block = '''    /// `item_path(stack)` -> path-строка предмета ("" если пусто).
    fn item_path(stack: &ItemStack) -> String {
        stack
            .item_identifier()
            .map(|i| i.path)
            .unwrap_or_default()
    }

    /// `GOAT_TEMPTATIONS` — goat food.
    pub fn goat() -> Self {
        Self::new(|_mob, stack| {
            matches!(
                Self::item_path(stack).as_str(),
                "wheat" | "hay_block" | "carrot" | "potato" | "beetroot"
                    | "torchflower_seeds" | "pitcher_pod"
            )
        })
    }

    /// `CAMEL_TEMPTATIONS`.
    pub fn camel() -> Self {
        Self::new(|_mob, stack| {
            matches!(Self::item_path(stack).as_str(), "cactus" | "wheat" | "hay_block")
        })
    }

    /// `ARMADILLO_TEMPTATIONS` — spider_eyes/seeds.
    pub fn armadillo() -> Self {
        Self::new(|_mob, stack| {
            matches!(
                Self::item_path(stack).as_str(),
                "spider_eye" | "sweet_berries" | "glow_berries" | "torchflower_seeds"
            )
        })
    }

    /// `SNIFFER_TEMPTATIONS` — torchflower/pitcher pod.
    pub fn sniffer() -> Self {
        Self::new(|_mob, stack| {
            matches!(Self::item_path(stack).as_str(), "torchflower_seeds" | "pitcher_pod")
        })
    }

    /// `HAPPY_GHAST_TEMPTATIONS` — ghast food.
    pub fn happy_ghast() -> Self {
        Self::new(|_mob, stack| {
            matches!(Self::item_path(stack).as_str(), "snowball" | "powder_snow_bucket")
        })
    }

    /// `FROG_TEMPTATIONS` — slimeballs.
    pub fn frog() -> Self {
        Self::new(|_mob, stack| Self::item_path(stack) == "slime_ball")
    }

    /// `NAUTILUS_TEMPTATIONS` — nautilus food.
    pub fn nautilus() -> Self {
        Self::new(|_mob, stack| {
            matches!(Self::item_path(stack).as_str(), "clownfish" | "tropical_fish")
        })
    }

'''
s = s[:start] + new_block + s[end:]

s = s.replace('item.item_entity_stack().unwrap_or_else(ItemStack::empty_static)',
              'item.item_entity_stack().unwrap_or_else(ItemStack::empty)')

# unused imports cleanup
s = s.replace('use crate::world::phys::vec3::Vec3;\n', '')
s = s.replace('use std::collections::HashSet;\n', '')
s = s.replace('use crate::nbt::tag::Tag;\n', '')
s = s.replace('use crate::world::damagesource::damage_source::DamageSource;\n', '')
s = s.replace('''/// Сбор memory-типов сенсора в requires()-множество.
fn reqs(list: &[MMT]) -> HashSet<MMT> {
    list.iter().cloned().collect()
}

/// Unit-память (`Unit.INSTANCE`) — `()`.
pub type UnitMemory = ();

''', '')
s = s.replace('''use crate::world::entity::ai::memory::memory::{
    ExpirableValue, MemoryModuleType, MemoryStatus, NearestVisibleLivingEntities,
};
use crate::world::entity::ai::memory::memory::MemoryModuleType as MMT;''',
'''use crate::world::entity::ai::memory::memory::{
    MemoryModuleType, NearestVisibleLivingEntities,
};
use crate::world::entity::ai::memory::memory::MemoryModuleType as MMT;''')

io.open(p, 'w', encoding='utf-8').write(s)
print('patched')
