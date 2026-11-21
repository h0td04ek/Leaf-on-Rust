//! Перенос `net.minecraft.world.entity.projectile.ProjectileUtil`.

use crate::entity::humanoid_arm::HumanoidArm;
use crate::resources::identifier::Identifier;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::interaction_hand::InteractionHand;

/// `ProjectileUtil.getWeaponHoldingHand(shooter, weapon)` —
/// рука, в которой держится оружие: `MAIN_HAND`, если предмет
/// в основной руке совпадает, иначе `OFF_HAND`.
pub fn get_weapon_holding_hand(shooter: &LivingEntity, weapon: &Identifier) -> InteractionHand {
    if shooter.get_main_hand_item().is(weapon) {
        InteractionHand::MainHand
    } else {
        InteractionHand::OffHand
    }
}

/// `InteractionHand` → `HumanoidArm` руки сущности
/// (`LivingEntity.startUsingItem(InteractionHand)` конвертирует
/// через `getMainArm()`/`getOpposite()`).
pub fn hand_to_arm(shooter: &LivingEntity, hand: InteractionHand) -> HumanoidArm {
    let main = shooter.get_main_arm();
    match hand {
        InteractionHand::MainHand => main,
        InteractionHand::OffHand => main.get_opposite(),
    }
}

use std::sync::Arc;

use super::Projectile;
use crate::world::entity::entity::Entity;
use crate::world::level::clip_context::{ClipBlockMode, ClipContext, ClipFluidMode};
use crate::world::level::level_reader::LevelReader;
use crate::world::level::Level;
use crate::world::phys::{Aabb, BlockHitResult, Vec3};

pub struct EntityHitResult {
    pub entity: Entity,
    pub location: Vec3,
}

pub enum HitResult {
    Entity(Box<EntityHitResult>),
    Block(BlockHitResult),
}

/// Сразу после выстрела рамка попадания совпадает с рамкой цели.
pub fn compute_margin(source: &Entity) -> f32 {
    ((source.tick_count().saturating_sub(2)) as f32 / 20.0).clamp(0.0, 0.3)
}

pub fn get_hit_result_on_move_vector(
    projectile: &Projectile,
    level: &dyn Level,
    block_mode: ClipBlockMode,
) -> HitResult {
    let source = &projectile.entity;
    let start = source.position();
    let movement = source.get_delta_movement();
    let end = start.add_vec(&movement);
    let clip = ClipContext::new(
        start,
        end,
        block_mode,
        ClipFluidMode::None,
        Some(Arc::new(source.clone())),
    );
    let block_hit = level.clip_including_border(&clip);
    let end = if block_hit.miss {
        end
    } else {
        block_hit.location
    };
    let bounds = source
        .get_bounding_box()
        .expand_towards_vec(&movement)
        .inflate_all(1.0);
    let entity_hit = get_entity_hit_result(
        level,
        source,
        start,
        end,
        &bounds,
        compute_margin(source),
        |target| projectile.can_hit_entity(target),
    );
    match entity_hit {
        Some(hit) => HitResult::Entity(Box::new(hit)),
        None => HitResult::Block(block_hit),
    }
}

pub fn get_entity_hit_result(
    level: &dyn LevelReader,
    source: &Entity,
    start: Vec3,
    end: Vec3,
    bounds: &Aabb,
    margin: f32,
    filter: impl Fn(&Entity) -> bool,
) -> Option<EntityHitResult> {
    let mut closest = None;
    let mut max_distance = f64::MAX;
    for entity in level.entities_in_box(bounds) {
        if entity.uuid == source.uuid || !filter(&entity) {
            continue;
        }
        let entity_bounds = entity.get_bounding_box().inflate_all(f64::from(margin));
        if let Some(location) = entity_bounds.clip(&start, &end) {
            let distance = start.distance_to_sqr(&location);
            if distance < max_distance {
                max_distance = distance;
                closest = Some(EntityHitResult { entity, location });
            }
        }
    }
    closest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::block_pos::BlockPos;
    use crate::core::holder::Holder;
    use crate::util::uuid::Uuid;
    use crate::world::entity::entity_type::item_entity_type;
    use crate::world::level::block::state::block_state::BlockState;

    struct TestLevel(Vec<Entity>);

    impl LevelReader for TestLevel {
        fn get_block_state(&self, _pos: BlockPos) -> Option<BlockState> {
            None
        }

        fn has_chunk_at(&self, _pos: BlockPos) -> bool {
            true
        }

        fn entities_in_box(&self, aabb: &Aabb) -> Vec<Entity> {
            self.0
                .iter()
                .filter(|entity| entity.get_bounding_box().intersects(aabb))
                .cloned()
                .collect()
        }
    }

    #[test]
    fn selects_the_nearest_target_before_the_block() {
        let entity_at = |x, id| {
            let mut entity = Entity::new(Holder::Direct(item_entity_type()), None);
            entity.uuid = Uuid::new(0, id);
            entity.set_pos(x, 0.0, 0.0);
            entity
        };
        let projectile = Projectile::new(entity_at(0.0, 1));
        let near = entity_at(2.0, 2);
        let far = entity_at(4.0, 3);
        let level = TestLevel(vec![far, projectile.entity.clone(), near.clone()]);
        let bounds = Aabb::new(-1.0, -1.0, -1.0, 6.0, 1.0, 1.0);
        let start = Vec3::new(0.0, 0.1, 0.0);
        let hit = |end_x| {
            get_entity_hit_result(
                &level,
                &projectile.entity,
                start,
                Vec3::new(end_x, 0.1, 0.0),
                &bounds,
                0.0,
                |target| projectile.can_hit_entity(target),
            )
        };
        assert_eq!(hit(5.0).unwrap().entity.uuid, near.uuid);
        assert!(hit(1.0).is_none());
    }
}
