//! `net.minecraft.world.entity.EntitySelector` — статические
//! предикаты отбора сущностей.

use crate::world::entity::entity::Entity;

/// `Predicate<Entity>`.
pub type EntityPredicate = dyn Fn(&Entity) -> bool + Send + Sync;

/// `ENTITY_STILL_ALIVE` = `Entity::isAlive`.
pub fn entity_still_alive(e: &Entity) -> bool {
    e.is_alive()
}

/// `LIVING_ENTITY_STILL_ALIVE` = `isAlive && instanceof LivingEntity`.
pub fn living_entity_still_alive(e: &Entity) -> bool {
    e.is_alive() && e.is_living
}

/// `ENTITY_NOT_BEING_RIDDEN` = `isAlive && !isVehicle && !isPassenger`.
pub fn entity_not_being_ridden(e: &Entity) -> bool {
    e.is_alive() && !e.is_vehicle() && !e.is_passenger()
}

/// `CONTAINER_ENTITY_SELECTOR` = `instanceof Container && isAlive`.
/// `Container`-сущности ещё не портированы — проверка через хук
/// `is_container_entity` (default false).
pub fn container_entity_selector(e: &Entity) -> bool {
    e.is_container_entity() && e.is_alive()
}

/// `NO_CREATIVE_OR_SPECTATOR` = `!(player && (spectator || creative))`.
pub fn no_creative_or_spectator(e: &Entity) -> bool {
    !(e.is_player() && (e.is_spectator() || e.is_creative()))
}

/// `NO_SPECTATORS` = `!isSpectator`.
pub fn no_spectators(e: &Entity) -> bool {
    !e.is_spectator()
}

/// `CAN_BE_COLLIDED_WITH` = `NO_SPECTATORS && canBeCollidedWith(null)`.
pub fn can_be_collided_with(e: &Entity) -> bool {
    no_spectators(e) && e.can_be_collided_with(None)
}

/// `CAN_BE_PICKED` = `NO_SPECTATORS && isPickable`.
pub fn can_be_picked(e: &Entity) -> bool {
    no_spectators(e) && e.is_pickable()
}

/// `PLAYER_AFFECTS_SPAWNING` (Paper) = `!spectator && isAlive &&
/// player && affectsSpawning`.
pub fn player_affects_spawning(e: &Entity) -> bool {
    !e.is_spectator() && e.is_alive() && e.is_player() && e.affects_spawning()
}

/// Композиция предикатов (`Predicate.and`).
pub fn and<'a>(
    a: &'a EntityPredicate,
    b: &'a EntityPredicate,
) -> impl Fn(&Entity) -> bool + Send + Sync + 'a {
    move |e| a(e) && b(e)
}

/// `withinDistance(x, y, z, range)` — `distanceToSqr <= range²`.
pub fn within_distance(
    x: f64,
    y: f64,
    z: f64,
    range: f64,
) -> impl Fn(&Entity) -> bool + Send + Sync {
    let d = range * range;
    move |e| e.distance_to_sqr_pos(x, y, z) <= d
}
