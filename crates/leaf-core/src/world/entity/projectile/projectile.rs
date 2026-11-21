//! Базовое состояние и баллистика запуска Projectile.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::entity::entity_type::EntityType as NetworkEntityType;
use crate::nbt::CompoundTag;
use crate::network::protocol::game::clientbound_add_entity_packet::ClientboundAddEntityPacket;
use crate::tags::entity_type_tags::{is_entity_type_id_in_tag, keys};
use crate::tags::TagKey;
use crate::util::mth;
use crate::util::uuid::Uuid;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::entity::entity::{Entity, RemovalReason};
use crate::world::entity::entity_reference::EntityReference;
use crate::world::item::item_stack::ItemStack;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::gameevent::game_event::{self, Context};
use crate::world::level::level_reader::LevelReader;
use crate::world::level::Level;
use crate::world::phys::{BlockHitResult, Vec3};

use super::super::entity_type::{holder_type_id, EntityType};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProjectileDeflection {
    #[default]
    None,
    Reverse,
    Aim,
    Momentum,
}

pub enum ProjectileHit<'a> {
    Entity { target: &'a Entity, location: Vec3 },
    Block(&'a BlockHitResult),
}

/// Лимиты загрузок чанков из конфигурации Gale.
#[derive(Clone, Copy, Debug)]
pub struct ProjectileChunkLoadLimits {
    pub per_tick: i32,
    pub per_projectile: i32,
    pub after_limit: ChunkLoadLimitAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkLoadLimitAction {
    Stop,
    Remove,
    ResetHorizontalMovement,
}

/// Общий счётчик загрузок снарядами за серверный тик.
#[derive(Default)]
pub struct ProjectileChunkLoadBudget {
    tick: i64,
    loaded: i32,
}

pub struct Projectile {
    pub entity: Entity,
    owner: Option<EntityReference>,
    pub left_owner: bool,
    left_owner_checked: bool,
    last_deflected_by: Option<Uuid>,
    chunks_loaded: i32,
    pub has_been_shot: bool,
    pub needs_sync: bool,
}

impl Projectile {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            owner: None,
            left_owner: false,
            left_owner_checked: false,
            last_deflected_by: None,
            chunks_loaded: 0,
            has_been_shot: false,
            needs_sync: false,
        }
    }

    pub fn set_owner(&mut self, owner: Option<&Entity>) {
        self.owner = owner.map(EntityReference::of);
    }

    pub fn get_owner(&self) -> Option<Entity> {
        self.owner.as_ref()?.get(self.entity.level.as_ref())
    }

    pub fn owned_by(&self, entity: &Entity) -> bool {
        self.owner
            .as_ref()
            .is_some_and(|owner| owner.matches(entity))
    }

    pub fn set_pos(
        &mut self,
        pos: Vec3,
        server_tick: i64,
        budget: &mut ProjectileChunkLoadBudget,
        limits: ProjectileChunkLoadLimits,
    ) -> bool {
        if budget.tick != server_tick {
            budget.tick = server_tick;
            budget.loaded = 0;
        }
        let previous = self.entity.block_position();
        let next = BlockPos::containing_vec3(&pos);
        let previous_chunk = ChunkPos::new(previous.get_x() >> 4, previous.get_z() >> 4);
        let next_chunk = ChunkPos::new(next.get_x() >> 4, next.get_z() >> 4);
        let loads_chunk = previous_chunk != next_chunk
            && self
                .entity
                .level
                .as_ref()
                .is_some_and(|level| !level.is_chunk_loaded(next_chunk));

        if loads_chunk {
            if limits.per_tick >= 0 && budget.loaded > limits.per_tick {
                return false;
            }
            if limits.per_projectile >= 0 && self.chunks_loaded >= limits.per_projectile {
                match limits.after_limit {
                    ChunkLoadLimitAction::Stop => {}
                    ChunkLoadLimitAction::Remove => self.entity.remove(RemovalReason::Discarded),
                    ChunkLoadLimitAction::ResetHorizontalMovement => {
                        self.entity
                            .set_delta_movement(0.0, self.entity.velocity.y, 0.0);
                    }
                }
                return false;
            }
            budget.loaded = budget.loaded.saturating_add(1);
            self.chunks_loaded = self.chunks_loaded.saturating_add(1);
        }

        self.entity.set_pos(pos.x, pos.y, pos.z);
        true
    }

    pub fn spawn(self, level: Arc<dyn LevelReader>, spawned_from: &ItemStack) -> bool {
        let mut projectile = self;
        projectile.entity.level = Some(level.clone());
        level.add_fresh_projectile(projectile, spawned_from)
    }

    pub fn effect_source(&self) -> Entity {
        self.get_owner().unwrap_or_else(|| self.entity.clone())
    }

    pub fn restore_owner_from(&mut self, other: &Self) {
        self.owner = other.owner.clone();
    }

    pub fn tick(&mut self) {
        if !self.has_been_shot {
            if let Some(level) = self.entity.level.as_ref() {
                level.game_event_with_context(
                    &game_event::projectile_shoot(),
                    self.entity.position(),
                    Context::of_entity(self.get_owner().map(Arc::new)),
                );
            }
            self.has_been_shot = true;
        }
        self.check_left_owner();
        self.entity.tick();
        self.left_owner_checked = false;
    }

    pub fn check_left_owner(&mut self) {
        if self.left_owner || self.left_owner_checked {
            return;
        }
        self.left_owner = self.is_outside_owner_collision_range();
        self.left_owner_checked = true;
    }

    fn is_outside_owner_collision_range(&self) -> bool {
        let Some(owner) = self.get_owner() else {
            return true;
        };
        let root = owner.get_root_vehicle();
        let bounds = self
            .entity
            .get_bounding_box()
            .expand_towards_vec(&self.entity.get_delta_movement())
            .inflate_all(1.0);
        let contains = |entity: &Entity| {
            crate::world::entity::entity_selector::can_be_picked(entity)
                && bounds.intersects(&entity.get_bounding_box())
        };
        if contains(&owner) || contains(&root) {
            return false;
        }
        self.entity.level.as_ref().is_none_or(|level| {
            !level
                .entities_in_box(&bounds)
                .iter()
                .any(|entity| contains(entity) && entity.get_root_vehicle().uuid == root.uuid)
        })
    }

    pub fn can_hit_entity(&self, target: &Entity) -> bool {
        target.is_alive()
            && !target.is_spectator()
            && target.is_pickable()
            && target.uuid != self.entity.uuid
            && (self.left_owner
                || self
                    .get_owner()
                    .is_none_or(|owner| !owner.is_passenger_of_same_vehicle(target)))
    }

    pub fn movement_to_shoot(&self, direction: Vec3, speed: f32, inaccuracy: f32) -> Vec3 {
        let deviation = 0.017_227_5 * f64::from(inaccuracy);
        let mut random = self.entity.random_mut();
        direction
            .normalize()
            .add(
                random.triangle_f64(0.0, deviation),
                random.triangle_f64(0.0, deviation),
                random.triangle_f64(0.0, deviation),
            )
            .scale(f64::from(speed))
    }

    pub fn shoot(&mut self, direction: Vec3, speed: f32, inaccuracy: f32) {
        let movement = self.movement_to_shoot(direction, speed, inaccuracy);
        self.entity
            .set_delta_movement(movement.x, movement.y, movement.z);
        let horizontal = (movement.x * movement.x + movement.z * movement.z).sqrt();
        let degrees = 180.0 / f64::from(std::f32::consts::PI);
        self.entity
            .set_y_rot((mth::atan2(movement.x, movement.z) * degrees) as f32);
        self.entity
            .set_x_rot((mth::atan2(movement.y, horizontal) * degrees) as f32);
        self.entity.old_rotation = self.entity.rotation;
        self.needs_sync = true;
    }

    pub fn shoot_from_rotation(
        &mut self,
        shooter: &Entity,
        pitch_offset: f32,
        speed: f32,
        inaccuracy: f32,
        inherit_movement: bool,
    ) {
        let pitch = shooter.get_x_rot();
        let yaw = shooter.get_y_rot().to_radians();
        let cos_pitch = mth::cos(f64::from(pitch.to_radians()));
        let direction = Vec3::new(
            f64::from(-mth::sin(f64::from(yaw)) * cos_pitch),
            f64::from(-mth::sin(f64::from((pitch + pitch_offset).to_radians()))),
            f64::from(mth::cos(f64::from(yaw)) * cos_pitch),
        );
        self.shoot(direction, speed, inaccuracy);

        if inherit_movement {
            let inherited = shooter.get_known_movement();
            if !inherited.x.is_nan() && !inherited.y.is_nan() && !inherited.z.is_nan() {
                let y = if shooter.is_on_ground() {
                    0.0
                } else {
                    inherited.y
                };
                let movement = self
                    .entity
                    .get_delta_movement()
                    .add(inherited.x, y, inherited.z);
                self.entity
                    .set_delta_movement(movement.x, movement.y, movement.z);
            }
        }
    }

    pub fn on_above_bubble_column(&mut self, downwards: bool) {
        self.entity.velocity.y += if downwards { -0.03 } else { 0.1 };
    }

    pub fn on_inside_bubble_column(&mut self, downwards: bool) {
        self.entity.velocity.y += if downwards { -0.03 } else { 0.06 };
        self.entity.fall_distance = 0.0;
    }

    pub fn hit_target_or_deflect_self(
        &mut self,
        hit: ProjectileHit<'_>,
        bounce_on_world_border: bool,
        on_entity_hit: impl FnOnce(&mut Projectile, &Entity, Vec3),
    ) -> ProjectileDeflection {
        let level = self.entity.level.clone();
        let owner = self.owner.clone();
        match hit {
            ProjectileHit::Entity { target, location } => {
                let deflection = level.as_ref().map_or_else(
                    || ProjectileDeflection::for_target(target),
                    |level| level.projectile_deflection(target, self),
                );
                if deflection != ProjectileDeflection::None {
                    if self.last_deflected_by != Some(target.uuid) {
                        self.deflect(deflection, Some(target), owner, false);
                        self.last_deflected_by = Some(target.uuid);
                    }
                    return deflection;
                }
                if let Some(level) = level.as_ref() {
                    if tagged(target, &keys::redirectable_projectile()) {
                        level.redirect_projectile(target.id, self.get_owner().as_ref(), owner);
                    }
                }
                on_entity_hit(self, target, location);
                if let Some(level) = level.as_ref() {
                    level.game_event_with_context(
                        &game_event::projectile_land(),
                        location,
                        Context::of_entity(Some(Arc::new(self.entity.clone()))),
                    );
                }
            }
            ProjectileHit::Block(hit) if !hit.miss => {
                if bounce_on_world_border && hit.world_border_hit {
                    self.deflect(ProjectileDeflection::Reverse, None, owner, false);
                    self.entity.velocity = self.entity.velocity.scale(0.2);
                    return ProjectileDeflection::Reverse;
                }
                if let Some(level) = level.as_ref() {
                    if let Some(state) = level.get_block_state(hit.block_pos) {
                        if let Some(on_hit) = &state.block.properties.on_projectile_hit {
                            on_hit(level.as_ref(), &state, hit, self);
                        }
                    }
                    level.game_event_with_context(
                        &game_event::projectile_land(),
                        crate::world::phys::vec3::at_center_of(&hit.block_pos.to_vec3i()),
                        Context::new(
                            Some(Arc::new(self.entity.clone())),
                            level.get_block_state(hit.block_pos),
                        ),
                    );
                }
            }
            ProjectileHit::Block(_) => {}
        }
        ProjectileDeflection::None
    }

    pub fn deflect(
        &mut self,
        kind: ProjectileDeflection,
        deflector: Option<&Entity>,
        owner: Option<EntityReference>,
        deflected_by_player: bool,
    ) {
        match kind {
            ProjectileDeflection::None => {}
            ProjectileDeflection::Reverse => {
                let rotation = 170.0 + self.entity.random_mut().next_float() * 20.0;
                self.entity.velocity = self.entity.velocity.scale(-0.5);
                self.entity.rotation.x += rotation;
                self.entity.old_rotation.x += rotation;
                self.needs_sync = true;
            }
            ProjectileDeflection::Aim | ProjectileDeflection::Momentum => {
                if let Some(deflector) = deflector {
                    self.entity.velocity = if kind == ProjectileDeflection::Aim {
                        deflector.get_look_angle()
                    } else {
                        deflector.get_delta_movement().normalize()
                    };
                    self.needs_sync = true;
                }
            }
        }
        let level = self.entity.level.clone();
        if level.as_ref().is_none_or(|level| !level.is_client_side()) {
            self.owner = owner;
            if let Some(level) = level {
                level.projectile_deflected(self, deflected_by_player);
            }
        }
    }

    pub fn update_rotation(&mut self) {
        let movement = self.entity.get_delta_movement();
        let horizontal = (movement.x * movement.x + movement.z * movement.z).sqrt();
        let degrees = 180.0 / f64::from(std::f32::consts::PI);
        self.entity.set_x_rot(lerp_rotation(
            self.entity.old_rotation.y,
            (mth::atan2(movement.y, horizontal) * degrees) as f32,
        ));
        self.entity.set_y_rot(lerp_rotation(
            self.entity.old_rotation.x,
            (mth::atan2(movement.x, movement.z) * degrees) as f32,
        ));
    }

    pub fn add_entity_packet(
        &self,
        wire_type: Holder<NetworkEntityType>,
    ) -> ClientboundAddEntityPacket {
        let pos = self.entity.position();
        ClientboundAddEntityPacket::new(
            self.entity.id,
            self.entity.uuid,
            wire_type,
            pos.x,
            pos.y,
            pos.z,
            self.entity.get_delta_movement(),
            self.entity.get_x_rot(),
            self.entity.get_y_rot(),
            f64::from(self.entity.y_head_rot),
            self.get_owner().map_or(0, |owner| owner.id),
        )
    }

    pub fn recreate_from_packet(&mut self, packet: &ClientboundAddEntityPacket) {
        self.entity.id = packet.id;
        self.entity.uuid = packet.uuid;
        self.entity.set_pos(packet.x, packet.y, packet.z);
        self.entity.old_pos = self.entity.pos;
        self.entity.set_x_rot(packet.get_x_rot());
        self.entity.set_y_rot(packet.get_y_rot());
        self.entity.y_head_rot = packet.get_y_head_rot();
        self.entity.old_rotation = self.entity.rotation;
        let movement = packet.movement;
        self.entity
            .set_delta_movement(movement.x, movement.y, movement.z);
        let owner = self
            .entity
            .level
            .as_ref()
            .and_then(|level| level.entity_by_id(packet.data));
        self.set_owner(owner.as_ref());
    }

    pub fn may_interact(&self, level: &dyn Level, pos: BlockPos) -> bool {
        self.get_owner().is_none_or(|owner| {
            if owner.is_player() {
                level.may_interact(&owner, pos)
            } else {
                level.mob_griefing()
            }
        })
    }

    pub fn may_break(&self, level: &dyn LevelReader) -> bool {
        tagged(&self.entity, &keys::impact_projectiles()) && level.projectile_can_break_blocks()
    }

    pub fn is_pickable(&self) -> bool {
        tagged(&self.entity, &keys::redirectable_projectile())
    }

    pub fn pick_radius(&self) -> f32 {
        if self.is_pickable() {
            1.0
        } else {
            0.0
        }
    }

    pub fn horizontal_hurt_knockback_direction(&self) -> (f64, f64) {
        (self.entity.velocity.x, self.entity.velocity.z)
    }

    pub fn dimension_changing_delay(&self) -> i32 {
        2
    }

    pub fn hurt_server(&mut self, source: &DamageSource) -> bool {
        if !self.entity.is_invulnerable() || source.bypasses_invulnerability() {
            self.needs_sync = true;
        }
        false
    }

    pub fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        if let Some(owner) = &self.owner {
            owner.store(tag, "Owner");
        } else {
            tag.remove("Owner");
        }
        if self.left_owner {
            tag.put_bool("LeftOwner", true);
        } else {
            tag.remove("LeftOwner");
        }
        tag.put_bool("HasBeenShot", self.has_been_shot);
    }

    pub fn read_additional_save_data(&mut self, tag: &CompoundTag) {
        self.owner = EntityReference::read(tag, "Owner");
        self.left_owner = tag.get_bool_or("LeftOwner", false);
        self.has_been_shot = tag.get_bool_or("HasBeenShot", false);
    }
}

impl ProjectileDeflection {
    pub fn for_target(target: &Entity) -> Self {
        if tagged(target, &keys::deflects_projectiles()) {
            Self::Reverse
        } else {
            Self::None
        }
    }
}

fn tagged(entity: &Entity, tag: &TagKey<EntityType>) -> bool {
    holder_type_id(&entity.entity_type).is_some_and(|id| is_entity_type_id_in_tag(&id, tag))
}

fn lerp_rotation(current: f32, target: f32) -> f32 {
    let spins = ((target - current) / 360.0 + 0.5).floor() as i32;
    mth::lerp_f32(0.2, current + spins as f32 * 360.0, target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::holder::Holder;
    use crate::util::uuid::Uuid;
    use crate::world::entity::entity_type::item_entity_type;

    #[test]
    fn applies_launch_rotation_and_paper_relative_velocity() {
        let mut shooter = Entity::new(Holder::Direct(item_entity_type()), None);
        shooter.set_delta_movement(0.5, 1.0, -0.5);
        shooter.on_ground = true;
        let mut projectile = Projectile::new(Entity::new(Holder::Direct(item_entity_type()), None));

        projectile.shoot(Vec3::new(3.0, 0.0, 0.0), 2.0, 0.0);
        assert_eq!(
            projectile.entity.get_delta_movement(),
            Vec3::new(2.0, 0.0, 0.0)
        );
        assert!((projectile.entity.get_y_rot() - 90.0).abs() < 0.001);
        assert!(projectile.needs_sync);

        projectile.shoot_from_rotation(&shooter, 0.0, 2.0, 0.0, true);
        assert_eq!(
            projectile.entity.get_delta_movement(),
            Vec3::new(0.5, 0.0, 1.5)
        );
    }

    #[test]
    fn ignores_its_owner_until_clear_and_breeze_deflection_applies_once() {
        let owner = Entity::new(Holder::Direct(item_entity_type()), None);
        let mut projectile = Projectile::new(Entity::new(Holder::Direct(item_entity_type()), None));
        projectile.set_owner(Some(&owner));
        projectile.tick();
        assert!(!projectile.left_owner);
        assert!(!projectile.can_hit_entity(&owner));

        projectile.entity.set_pos(5.0, 0.0, 0.0);
        projectile.tick();
        assert!(projectile.left_owner);
        assert!(projectile.can_hit_entity(&owner));

        let mut breeze_type = item_entity_type();
        breeze_type.id = crate::resources::Identifier::minecraft("breeze");
        let breeze = Entity::new(Holder::Direct(breeze_type), None);
        projectile.shoot(Vec3::new(0.0, 0.0, 1.0), 2.0, 0.0);
        for _ in 0..2 {
            let hit = ProjectileHit::Entity {
                target: &breeze,
                location: breeze.position(),
            };
            assert_eq!(
                projectile
                    .hit_target_or_deflect_self(hit, false, |_, _, _| panic!("should deflect")),
                ProjectileDeflection::Reverse
            );
            assert_eq!(
                projectile.entity.get_delta_movement(),
                Vec3::new(0.0, 0.0, -1.0)
            );
            assert!(projectile.owned_by(&owner));
        }
    }

    #[test]
    fn dispatches_shoot_and_block_hit_with_context() {
        use crate::world::level::block::state::block_state::BlockState;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Mutex;

        struct TestLevel {
            block: BlockState,
            events: Mutex<Vec<(String, Option<Uuid>)>>,
        }

        impl LevelReader for TestLevel {
            fn get_block_state(&self, _pos: BlockPos) -> Option<BlockState> {
                Some(self.block.clone())
            }

            fn has_chunk_at(&self, _pos: BlockPos) -> bool {
                true
            }

            fn game_event_with_context(
                &self,
                event: &game_event::GameEvent,
                _position: Vec3,
                context: Context,
            ) {
                let owner = context.source_entity.map(|entity| entity.uuid);
                self.events
                    .lock()
                    .unwrap()
                    .push((event.id.get_path().to_owned(), owner));
            }
        }

        let hit_count = Arc::new(AtomicUsize::new(0));
        let callback_count = hit_count.clone();
        let mut block = crate::world::level::block::Block::new(
            crate::resources::Identifier::minecraft("target"),
        );
        block.properties.on_projectile_hit = Some(Arc::new(move |_, _, _, _| {
            callback_count.fetch_add(1, Ordering::Relaxed);
        }));
        let level = Arc::new(TestLevel {
            block: BlockState::without_properties(Arc::new(block)),
            events: Mutex::new(Vec::new()),
        });
        let owner = Entity::new(Holder::Direct(item_entity_type()), Some(level.clone()));
        let mut projectile = Projectile::new(Entity::new(
            Holder::Direct(item_entity_type()),
            Some(level.clone()),
        ));
        projectile.set_owner(Some(&owner));
        projectile.tick();
        projectile.tick();

        let hit = BlockHitResult::new(
            Vec3::ZERO,
            crate::core::direction::Direction::UP,
            BlockPos::ZERO,
            false,
        );
        assert_eq!(
            projectile.hit_target_or_deflect_self(ProjectileHit::Block(&hit), false, |_, _, _| {}),
            ProjectileDeflection::None
        );
        assert_eq!(hit_count.load(Ordering::Relaxed), 1);
        assert_eq!(
            *level.events.lock().unwrap(),
            [
                ("projectile_shoot".to_owned(), Some(owner.uuid)),
                ("projectile_land".to_owned(), Some(projectile.entity.uuid)),
            ]
        );
    }

    #[test]
    fn projectile_chunk_limit_preserves_vertical_movement() {
        use crate::world::level::block::state::block_state::BlockState;

        struct UnloadedLevel;

        impl LevelReader for UnloadedLevel {
            fn get_block_state(&self, _pos: BlockPos) -> Option<BlockState> {
                None
            }

            fn has_chunk_at(&self, _pos: BlockPos) -> bool {
                false
            }
        }

        let level = Arc::new(UnloadedLevel);
        let mut projectile =
            Projectile::new(Entity::new(Holder::Direct(item_entity_type()), Some(level)));
        let mut budget = ProjectileChunkLoadBudget::default();
        let limits = ProjectileChunkLoadLimits {
            per_tick: -1,
            per_projectile: 1,
            after_limit: ChunkLoadLimitAction::ResetHorizontalMovement,
        };

        assert!(projectile.set_pos(Vec3::new(16.0, 0.0, 0.0), 1, &mut budget, limits));
        projectile.entity.set_delta_movement(1.0, -0.5, 0.5);
        assert!(!projectile.set_pos(Vec3::new(32.0, 0.0, 0.0), 2, &mut budget, limits));
        assert_eq!(projectile.entity.position(), Vec3::new(16.0, 0.0, 0.0));
        assert_eq!(
            projectile.entity.get_delta_movement(),
            Vec3::new(0.0, -0.5, 0.0)
        );
    }

    #[test]
    fn stores_owner_in_vanilla_uuid_format_and_reads_old_saves() {
        let mut shooter = Entity::new(Holder::Direct(item_entity_type()), None);
        shooter.uuid = Uuid::new(-1, i64::MIN + 4);
        let mut projectile = Projectile::new(Entity::new(Holder::Direct(item_entity_type()), None));
        projectile.set_owner(Some(&shooter));
        projectile.left_owner = true;

        let mut tag = CompoundTag::new();
        projectile.add_additional_save_data(&mut tag);
        assert_eq!(tag.get_int_array("Owner"), Some(&[-1, -1, i32::MIN, 4][..]));

        projectile.set_owner(None);
        projectile.read_additional_save_data(&tag);
        assert!(projectile.owned_by(&shooter));
        assert!(projectile.left_owner);

        tag.put_string("Owner", &shooter.uuid.to_string());
        projectile.read_additional_save_data(&tag);
        assert!(projectile.owned_by(&shooter));
    }
}
