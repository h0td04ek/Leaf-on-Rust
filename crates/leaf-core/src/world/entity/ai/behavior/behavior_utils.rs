//! Перенос `BehaviorUtils` — статические хелперы Brain-поведений.

use std::sync::Arc;

use crate::core::section_pos::SectionPos;
use crate::core::block_pos::BlockPos;
use crate::util::uuid::Uuid;
use crate::world::entity::ai::memory::memory::{keys, EntityTracker, MemoryModuleType, WalkTarget};
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;

/// `entityIsVisible(brain, target)` — NVLE содержит цель.
pub fn entity_is_visible(brain: &crate::world::entity::ai::brain::Brain, target: &LivingEntity) -> bool {
    brain
        .get_memory::<crate::world::entity::ai::memory::memory::NearestVisibleLivingEntities>(
            &keys::nearest_visible_living_entities(),
        )
        .map(|nv| nv.nearby_entities.iter().any(|e| e.get_uuid() == target.get_uuid()))
        .unwrap_or(false)
}

/// `targetIsValid(brain, memoryType, entityType)` — memory present
/// и цель нужного типа.
pub fn target_is_valid(
    brain: &crate::world::entity::ai::brain::Brain,
    memory_type: &MemoryModuleType,
    entity_type_id: &str,
) -> bool {
    let target: Option<LivingEntity> = brain.get_memory(memory_type);
    target
        .map(|t| {
            crate::world::entity::ai::sensing::sensor::entity_type_is(&t, entity_type_id)
        })
        .unwrap_or(false)
}

/// `lookAtEntity(entity, target)` — LOOK_TARGET = EntityTracker(target,true).
pub fn look_at_entity(entity: &Mob, target: &LivingEntity) {
    // Mob-обёртка: brain через living.
    let tracker = EntityTracker::new(target.entity.clone(), true);
    entity.living.brain_set_value(
        &keys::look_target(),
        std::rc::Rc::new(tracker)
            as std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>,
    );
}

/// `setWalkAndLookTargetMemories(entity, target, speed, dist)`.
pub fn set_walk_and_look_target_memories_entity(
    entity: &Mob,
    target: &LivingEntity,
    speed: f32,
    distance: i32,
) {
    let look = EntityTracker::new(target.entity.clone(), true);
    let walk = EntityTracker::new(target.entity.clone(), false);
    entity.living.brain_set_value(
        &keys::look_target(),
        std::rc::Rc::new(look)
            as std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>,
    );
    entity.living.brain_set_value(
        &keys::walk_target(),
        WalkTarget::of_tracker(
            std::rc::Rc::new(walk)
                as std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>,
            speed,
            distance,
        ),
    );
}

/// `setWalkAndLookTargetMemories(entity, blockPos, speed, dist)`.
pub fn set_walk_and_look_target_memories_pos(
    entity: &Mob,
    pos: BlockPos,
    speed: f32,
    distance: i32,
) {
    let tracker = crate::world::entity::ai::memory::memory::BlockPosTracker::new(pos);
    let tracker2 = crate::world::entity::ai::memory::memory::BlockPosTracker::new(pos);
    entity.living.brain_set_value(
        &keys::look_target(),
        std::rc::Rc::new(tracker)
            as std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>,
    );
    entity.living.brain_set_value(
        &keys::walk_target(),
        WalkTarget::of_tracker(
            std::rc::Rc::new(tracker2)
                as std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>,
            speed,
            distance,
        ),
    );
}

/// `setWalkAndLookTargetMemories(entity, positionTracker, speed, dist)`.
pub fn set_walk_and_look_target_memories_tracker(
    entity: &Mob,
    tracker: std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>,
    speed: f32,
    distance: i32,
) {
    entity.living.brain_set_value(&keys::look_target(), tracker.clone());
    entity.living.brain_set_value(
        &keys::walk_target(),
        WalkTarget::of_tracker(tracker, speed, distance),
    );
}

/// `isWithinAttackRange(mob, target, cooldown)` — projectile-оружие
/// → defaultProjectileRange - cooldown; иначе melee.
pub fn is_within_attack_range(mob: &Mob, target: &LivingEntity, cooldown: i32) -> bool {
    // ProjectileWeaponItem + canUseNonMeleeWeapon — через хуки
    // LivingEntity; default melee.
    if mob.living.can_use_non_melee_weapon() {
        let range = mob.living.non_melee_weapon_range() as i32 - cooldown;
        return mob.living.entity.closer_than_sqr(&target.entity, (range * range) as f64);
    }
    mob.is_within_melee_attack_range(target)
}

/// `isOtherTargetMuchFurtherAwayThanCurrentAttackTarget`.
pub fn is_other_target_much_further_away(
    living_entity: &LivingEntity,
    target: &LivingEntity,
    distance: f64,
) -> bool {
    let memory: Option<LivingEntity> = living_entity.brain_get(&keys::attack_target());
    let Some(current) = memory else { return false };
    let d = living_entity.entity.position().distance_to_sqr(&current.entity.position());
    let d1 = living_entity.entity.position().distance_to_sqr(&target.entity.position());
    d1 > d + distance * distance
}

/// `canSee(livingEntity, target)` — NVLE содержит target.
pub fn can_see(living_entity: &LivingEntity, target: &LivingEntity) -> bool {
    living_entity
        .brain_get::<crate::world::entity::ai::memory::memory::NearestVisibleLivingEntities>(
            &keys::nearest_visible_living_entities(),
        )
        .map(|nv| nv.nearby_entities.iter().any(|e| e.get_uuid() == target.get_uuid()))
        .unwrap_or(false)
}

/// `getNearestTarget(center, optional, fallback)`.
pub fn get_nearest_target(
    center_entity: &LivingEntity,
    optional_entity: Option<LivingEntity>,
    living_entity: LivingEntity,
) -> LivingEntity {
    match optional_entity {
        None => living_entity,
        Some(e) => get_target_nearest_me(center_entity, &e, &living_entity).clone(),
    }
}

/// `getTargetNearestMe(center, a, b)` — ближайшая к center.
pub fn get_target_nearest_me<'a>(
    center_entity: &LivingEntity,
    living_entity1: &'a LivingEntity,
    living_entity2: &'a LivingEntity,
) -> &'a LivingEntity {
    let v1 = living_entity1.entity.position();
    let v2 = living_entity2.entity.position();
    let d1 = center_entity.entity.position().distance_to_sqr(&v1);
    let d2 = center_entity.entity.position().distance_to_sqr(&v2);
    if d1 < d2 {
        living_entity1
    } else {
        living_entity2
    }
}

/// `getLivingEntityFromUUIDMemory(entity, memoryType)` —
/// UUID → level.entity_by_uuid → LivingEntity.
pub fn get_living_entity_from_uuid_memory(
    living_entity: &LivingEntity,
    target_memory: &MemoryModuleType,
) -> Option<LivingEntity> {
    let uuid: Option<Uuid> = living_entity.brain_get(target_memory);
    let uuid = uuid?;
    let level = living_entity.entity.level.as_ref()?;
    let entity = level.entity_by_uuid(uuid)?;
    if entity.is_living_entity() {
        Some(LivingEntity::from_entity(entity))
    } else {
        None
    }
}

/// `findSectionClosestToVillage(level, sectionPos, radius)` —
/// ближайшая секция к деревне (через LevelReader-хук
/// `find_section_closest_to_village`).
pub fn find_section_closest_to_village(
    level: &Arc<dyn LevelReader>,
    section_pos: SectionPos,
    radius: i32,
) -> SectionPos {
    level
        .find_section_closest_to_village(section_pos, radius)
        .unwrap_or(section_pos)
}

/// `isBreeding(entity)` — BREED_TARGET present.
pub fn is_breeding(entity: &LivingEntity) -> bool {
    entity.brain_has(&keys::breed_target())
}

/// `BehaviorUtils.lockGazeAndWalkToEachOther(entity, target,
/// speedModifier, closeEnoughDistance)` — LOOK_TARGET друг на друга
/// + WALK_TARGET каждый к другому.
pub fn lock_gaze_and_walk_to_each_other_living(
    entity: &LivingEntity,
    target: &LivingEntity,
    speed_modifier: f32,
    close_enough_distance: i32,
) {
    entity.brain_set_value(&keys::look_target(), tracker_entity(target, true));
    target.brain_set_value(&keys::look_target(), tracker_entity(entity, true));
    entity.brain_set_value(
        &keys::walk_target(),
        WalkTarget::of_tracker(tracker_entity(target, false), speed_modifier, close_enough_distance),
    );
    target.brain_set_value(
        &keys::walk_target(),
        WalkTarget::of_tracker(tracker_entity(entity, false), speed_modifier, close_enough_distance),
    );
}

fn tracker_entity(
    e: &LivingEntity,
    track_eye_height: bool,
) -> std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker> {
    std::rc::Rc::new(EntityTracker::new(e.entity.clone(), track_eye_height))
        as std::rc::Rc<dyn crate::world::entity::ai::memory::memory::PositionTracker>
}
