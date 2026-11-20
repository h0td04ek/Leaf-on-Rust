//! `net.minecraft.world.level.EntityGetter` — доступ к сущностям
//! в области (уровень/регион).

use std::sync::Arc;

use crate::world::entity::entity::Entity;
use crate::world::entity::entity_selector::{self, EntityPredicate};
use crate::world::phys::aabb::Aabb;
use crate::world::phys::shapes::{single_aabb_representation, voxel_shape_intersect_no_empty, VoxelShape};
use crate::world::phys::shapes::voxel_shape::COLLISION_EPSILON;

/// `EntityGetter`.
///
/// Абстрактный метод — `get_entities(entity, area, predicate)`.
/// Все остальные — default-реализации как в Java (включая
/// Paper-оптимизации коллизий).
pub trait EntityGetter {
    /// `getEntities(entity, area, predicate)`.
    fn get_entities(
        &self,
        entity: Option<&Entity>,
        area: &Aabb,
        predicate: &EntityPredicate,
    ) -> Vec<Arc<Entity>>;

    /// `players()` — список игроков (ServerLevel переопределяет).
    fn players(&self) -> Vec<Arc<Entity>> {
        Vec::new()
    }

    /// `getEntities(entity, area)` — NO_SPECTATORS.
    fn get_entities_no_filter(&self, entity: Option<&Entity>, area: &Aabb) -> Vec<Arc<Entity>> {
        self.get_entities(entity, area, &|e| entity_selector::no_spectators(e))
    }

    /// `getEntitiesOfClass` — в Rust Entity один тип;
    /// аналог — `get_entities` с предикатом (EntityTypeTest-эквивалент
    /// через `entity_type_test` — см. `world/level/entity/entity_type_test`).
    fn get_entities_of_class(
        &self,
        entity: Option<&Entity>,
        area: &Aabb,
        predicate: &EntityPredicate,
    ) -> Vec<Arc<Entity>> {
        self.get_entities(entity, area, predicate)
    }

    /// `moonrise$getHardCollidingEntities` — default → `getEntities`.
    fn moonrise_get_hard_colliding_entities(
        &self,
        entity: Option<&Entity>,
        area: &Aabb,
        predicate: Option<&EntityPredicate>,
    ) -> Vec<Arc<Entity>> {
        match predicate {
            Some(p) => self.get_entities(entity, area, p),
            None => self.get_entities(entity, area, &|_| true),
        }
    }

    /// `moonrise$isHardColliding` — сущность «жёстко» коллидирует
    /// (moonrise chunk-system; default false — переопределяется
    /// имплеменаторами entity).
    fn moonrise_is_hard_colliding(&self, _entity: &Entity) -> bool {
        false
    }

    /// `isUnobstructed(entity, voxel)` — Paper-оптимизированная
    /// проверка (одиночный AABB или попарное пересечение).
    fn is_unobstructed(&self, entity: Option<&Entity>, voxel: &VoxelShape) -> bool {
        if voxel.is_empty() {
            return true;
        }

        let single_aabb = single_aabb_representation(voxel);
        let check_area = match single_aabb {
            Some(a) => a.inflate(-COLLISION_EPSILON, -COLLISION_EPSILON, -COLLISION_EPSILON),
            None => voxel.bounds().unwrap_or(Aabb::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
        };
        let entities = self.get_entities(entity, &check_area, &|_| true);

        for other in &entities {
            if other.is_removed() || !other.blocks_building {
                continue;
            }
            if let Some(e) = entity {
                if other.is_passenger_of_same_vehicle(e) {
                    continue;
                }
            }

            if single_aabb.is_none() {
                let entity_bb = other.get_bounding_box();
                let empty = (entity_bb.max_x - entity_bb.min_x) < COLLISION_EPSILON
                    || (entity_bb.max_y - entity_bb.min_y) < COLLISION_EPSILON
                    || (entity_bb.max_z - entity_bb.min_z) < COLLISION_EPSILON;
                if empty || !voxel_shape_intersect_no_empty(voxel, &entity_bb) {
                    continue;
                }
            }

            return false;
        }

        true
    }

    /// `getEntityCollisions(entity, collisionBox)` — Paper-оптимизация:
    /// empty-check, сжатие на -epsilon, фильтр spectator/canCollide.
    fn get_entity_collisions(
        &self,
        entity: Option<&Entity>,
        collision_box: &Aabb,
    ) -> Vec<VoxelShape> {
        use crate::world::phys::shapes::collision_aabb_is_empty;

        if collision_aabb_is_empty(collision_box) {
            return Vec::new();
        }

        let collision_box = collision_box.inflate(
            -COLLISION_EPSILON,
            -COLLISION_EPSILON,
            -COLLISION_EPSILON,
        );

        let entities = match entity {
            Some(e) if self.moonrise_is_hard_colliding(e) => {
                self.get_entities(Some(e), &collision_box, &|_| true)
            }
            _ => self.moonrise_get_hard_colliding_entities(entity, &collision_box, None),
        };

        let mut ret = Vec::with_capacity(entities.len().min(25));
        for other in &entities {
            if other.is_spectator() {
                continue;
            }
            let collides = match entity {
                None => other.can_be_collided_with(None),
                Some(e) => e.can_collide_with(Some(other.get_bounding_box())),
            };
            if collides {
                ret.push(VoxelShape::of(other.get_bounding_box()));
            }
        }
        ret
    }

    /// `hasNearbyAlivePlayerThatAffectsSpawningForSpawner` —
    /// NearbyPlayers-оптимизация (GENERAL 33 / SPAWN_RANGE 8).
    /// Без chunk-system NearbyPlayers — fallback на `players()`
    /// с радиусом (эквивалент `...ForLargerRangeSpawner`).
    fn has_nearby_alive_player_that_affects_spawning_for_spawner(
        &self,
        x: f64,
        y: f64,
        z: f64,
        range: f64,
    ) -> bool {
        self.has_nearby_alive_player_that_affects_spawning_for_larger_range_spawner(
            x, y, z, range,
        )
    }

    /// `hasNearbyAlivePlayerThatAffectsSpawningForLargerRangeSpawner`.
    fn has_nearby_alive_player_that_affects_spawning_for_larger_range_spawner(
        &self,
        x: f64,
        y: f64,
        z: f64,
        range: f64,
    ) -> bool {
        for player in self.players() {
            let distance_sqr = player.distance_to_sqr_pos(x, y, z);
            if (range < 0.0 || distance_sqr < range * range)
                && entity_selector::player_affects_spawning(&player)
            {
                return true;
            }
        }
        false
    }

    /// `hasNearbyAlivePlayerThatAffectsSpawningForZombie` —
    /// NearbyPlayers SPAWN_RANGE (8) — fallback как выше.
    fn has_nearby_alive_player_that_affects_spawning_for_zombie(
        &self,
        x: f64,
        y: f64,
        z: f64,
        range: f64,
    ) -> bool {
        self.has_nearby_alive_player_that_affects_spawning_for_larger_range_spawner(
            x, y, z, range,
        )
    }

    /// `findNearbyPlayer(entity, maxDistance, predicate)` (Paper).
    fn find_nearby_player(
        &self,
        entity: &Entity,
        max_distance: f64,
        predicate: Option<&EntityPredicate>,
    ) -> Option<Arc<Entity>> {
        self.get_nearest_player_pos(
            entity.get_x(),
            entity.get_y(),
            entity.get_z(),
            max_distance,
            predicate,
        )
    }

    /// `getNearestPlayer(x, y, z, distance, predicate)` —
    /// Leaf-оптимизированный обход `players()`.
    fn get_nearest_player_pos(
        &self,
        x: f64,
        y: f64,
        z: f64,
        distance: f64,
        predicate: Option<&EntityPredicate>,
    ) -> Option<Arc<Entity>> {
        let mut distance = if distance < 0.0 {
            f64::MAX
        } else {
            distance * distance
        };
        let mut player: Option<Arc<Entity>> = None;
        for target in self.players() {
            if let Some(p) = predicate {
                if !p(&target) {
                    continue;
                }
            }
            let target_distance = target.distance_to_sqr_pos(x, y, z);
            if target_distance < distance {
                distance = target_distance;
                player = Some(target);
            }
        }
        player
    }

    /// `getNearestPlayer(entity, distance)`.
    fn get_nearest_player(&self, entity: &Entity, distance: f64) -> Option<Arc<Entity>> {
        self.get_nearest_player_pred(
            entity.get_x(),
            entity.get_y(),
            entity.get_z(),
            distance,
            false,
            None,
        )
    }

    /// `getNearestPlayer(x, y, z, distance, creativePlayers)`.
    fn get_nearest_player_pred(
        &self,
        x: f64,
        y: f64,
        z: f64,
        distance: f64,
        creative_players: bool,
        extra: Option<&EntityPredicate>,
    ) -> Option<Arc<Entity>> {
        let base: &EntityPredicate = if creative_players {
            &entity_selector::no_creative_or_spectator
        } else {
            &entity_selector::no_spectators
        };
        // `Predicate.and(base, extra)` — инлайн, чтобы не создавать
        // заимствующий closure (предикаты в API без 'static).
        let mut best_dist = if distance < 0.0 {
            f64::MAX
        } else {
            distance * distance
        };
        let mut best: Option<Arc<Entity>> = None;
        for target in self.players() {
            if !base(&target) {
                continue;
            }
            if let Some(p) = extra {
                if !p(&target) {
                    continue;
                }
            }
            let d = target.distance_to_sqr_pos(x, y, z);
            if d < best_dist {
                best_dist = d;
                best = Some(target);
            }
        }
        best
    }

    /// `getNearestPlayerThatAffectsSpawning` (Paper).
    fn get_nearest_player_that_affects_spawning(
        &self,
        x: f64,
        y: f64,
        z: f64,
        max_dist: f64,
        filter_out_creative: bool,
    ) -> Option<Arc<Entity>> {
        let base: &EntityPredicate = if filter_out_creative {
            &entity_selector::no_creative_or_spectator
        } else {
            &entity_selector::no_spectators
        };
        let mut best_dist = if max_dist < 0.0 {
            f64::MAX
        } else {
            max_dist * max_dist
        };
        let mut best: Option<Arc<Entity>> = None;
        for target in self.players() {
            if !base(&target) || !target.affects_spawning() {
                continue;
            }
            let d = target.distance_to_sqr_pos(x, y, z);
            if d < best_dist {
                best_dist = d;
                best = Some(target);
            }
        }
        best
    }

    /// `hasNearbyAlivePlayerThatAffectsSpawning` (Paper).
    fn has_nearby_alive_player_that_affects_spawning(
        &self,
        x: f64,
        y: f64,
        z: f64,
        range: f64,
    ) -> bool {
        for player in self.players() {
            if entity_selector::player_affects_spawning(&player) {
                let player_dist = player.distance_to_sqr_pos(x, y, z);
                if range < 0.0 || player_dist < range * range {
                    return true;
                }
            }
        }
        false
    }

    /// `hasNearbyAlivePlayer` — NO_SPECTATORS && LIVING_STILL_ALIVE &&
    /// !afk (Purpur).
    fn has_nearby_alive_player(&self, x: f64, y: f64, z: f64, distance: f64) -> bool {
        for player in self.players() {
            if entity_selector::no_spectators(&player)
                && entity_selector::living_entity_still_alive(&player)
                && !player.is_afk()
            {
                let d = player.distance_to_sqr_pos(x, y, z);
                if distance < 0.0 || d < distance * distance {
                    return true;
                }
            }
        }
        false
    }

    /// `getPlayerByUUID(uuid)`.
    fn get_player_by_uuid(&self, unique_id: crate::util::uuid::Uuid) -> Option<Arc<Entity>> {
        self.players()
            .into_iter()
            .find(|p| p.get_uuid() == unique_id)
    }

    /// `getGlobalPlayerByUUID` (Paper) — default → `getPlayerByUUID`.
    fn get_global_player_by_uuid(&self, uuid: crate::util::uuid::Uuid) -> Option<Arc<Entity>> {
        self.get_player_by_uuid(uuid)
    }
}
