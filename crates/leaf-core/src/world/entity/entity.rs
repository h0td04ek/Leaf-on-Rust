//! Перенос `net.minecraft.world.entity.Entity` — базовый класс сущности.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::section_pos::SectionPos;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::network::chat::component::Component;
use crate::util::uuid::Uuid;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::vec3::Vec3;

/// `Vec2` — 2D вектор для вращения (yaw, pitch).
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2 {
    /// `x` — yaw.
    pub x: f32,
    /// `y` — pitch.
    pub y: f32,
}

impl Vec2 {
    /// `new(x, y)`.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// `RemovalReason` — причина удаления сущности.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemovalReason {
    /// `KILLED`.
    Killed,
    /// `DISCARDED`.
    Discarded,
    /// `UNLOADED_TO_CHUNK`.
    UnloadedToChunk,
    /// `UNLOADED_WITH_PLAYER`.
    UnloadedWithPlayer,
    /// `CHANGED_DIMENSION`.
    ChangedDimension,
}

/// `Entity` — базовый класс сущности.
pub struct Entity {
    /// `id` — entity ID.
    pub id: i32,
    /// `uuid` — UUID сущности.
    pub uuid: Uuid,
    /// `type` — тип сущности.
    pub entity_type: crate::core::holder::Holder<crate::world::entity::entity_type::EntityType>,
    /// `level` — уровень.
    pub level: Option<Arc<dyn LevelReader>>,
    /// `pos` — позиция.
    pub pos: Vec3,
    /// `oldPos` — предыдущая позиция.
    pub old_pos: Vec3,
    /// `rotation` — вращение (yaw, pitch).
    pub rotation: Vec2,
    /// `oldRotation` — предыдущее вращение.
    pub old_rotation: Vec2,
    /// `velocity` — скорость.
    pub velocity: Vec3,
    /// `onGround` — на земле.
    pub on_ground: bool,
    /// `horizontalCollision` — горизонтальное столкновение.
    pub horizontal_collision: bool,
    /// `verticalCollision` — вертикальное столкновение.
    pub vertical_collision: bool,
    /// `hurtTime` — время урона.
    pub hurt_time: i32,
    /// `hurtDuration` — длительность урона.
    pub hurt_duration: i32,
    /// `deathTime` — время смерти.
    pub death_time: i32,
    /// `tickCount` — счётчик тиков.
    pub tick_count: i32,
    /// `firstTick` — первый тик.
    pub first_tick: bool,
    /// `removed` — удалён ли.
    pub removed: bool,
    /// `removalReason` — причина удаления.
    pub removal_reason: Option<RemovalReason>,
    /// `name` — имя.
    pub name: Option<Component>,
    /// `customName` — кастомное имя.
    pub custom_name: Option<Component>,
    /// `boundingBox` — bounding box.
    pub bounding_box: Aabb,
    /// `dimensions` — размеры.
    pub dimensions: crate::world::entity::entity_type::EntityDimensions,
    /// `eyeHeight` — высота глаз.
    pub eye_height: f32,
    /// `eyeHeightOld` — предыдущая высота глаз.
    pub eye_height_old: f32,
    /// `invisible` — невидим.
    pub invisible: bool,
    /// `invulnerable` — неуязвимость (босс/команды).
    pub invulnerable: bool,
    /// `glowing` — светящийся.
    pub glowing: bool,
    /// `noPhysics` — без физики.
    pub no_physics: bool,
    /// `sharedFlags` — shared flags.
    pub shared_flags: u8,
    /// `data` — synced entity data.
    pub data: crate::network::syncher::synched_entity_data::SynchedEntityData,
    /// `tags` — теги.
    pub tags: Vec<crate::resources::identifier::Identifier>,
    /// `persistenceRequired` — требуется сохранение.
    pub persistence_required: bool,
    /// `fallDistance` — дистанция падения.
    pub fall_distance: f32,
    /// `wasTouchingWater` — касался ли воды в прошлом тике.
    pub was_touching_water: bool,
    /// `wasEyeInWater` — глаз был в воде.
    pub was_eye_in_water: bool,
    /// `random` — `Entity.random` (LegacyRandomSource).
    /// `Arc<Mutex>` — `Entity` должен оставаться Send+Sync
    /// (цели AI и сущности разделяются между потоками).
    pub random: std::sync::Arc<std::sync::Mutex<Box<dyn crate::random::RandomSource>>>,
    /// `noGravity` — без гравитации.
    pub no_gravity: bool,
    /// Признак «сущность — Mob» (в Java — `instanceof Mob`;
    /// выставляется конструктором `Mob`, т.к. у нас композиция,
    /// а не наследование).
    pub is_mob: bool,
}

impl Entity {
    /// `new(type, level)`.
    pub fn new(
        entity_type: crate::core::holder::Holder<crate::world::entity::entity_type::EntityType>,
        level: Option<Arc<dyn LevelReader>>,
    ) -> Self {
        let half = 0.3_f64;
        Self {
            id: 0,
            uuid: Uuid::random(),
            entity_type,
            level,
            pos: Vec3::ZERO,
            old_pos: Vec3::ZERO,
            rotation: Vec2::new(0.0, 0.0),
            old_rotation: Vec2::new(0.0, 0.0),
            velocity: Vec3::ZERO,
            on_ground: false,
            horizontal_collision: false,
            vertical_collision: false,
            hurt_time: 0,
            hurt_duration: 0,
            death_time: 0,
            tick_count: 0,
            first_tick: true,
            removed: false,
            removal_reason: None,
            name: None,
            custom_name: None,
            bounding_box: Aabb::new(-half, 0.0, -half, half, 1.8, half),
            dimensions: crate::world::entity::entity_type::EntityDimensions::fixed(0.6, 1.8),
            eye_height: 1.62,
            eye_height_old: 1.62,
            invisible: false,
            invulnerable: false,
            glowing: false,
            no_physics: false,
            shared_flags: 0,
            data: crate::network::syncher::synched_entity_data::SynchedEntityData::new(Vec::new()),
            tags: Vec::new(),
            persistence_required: false,
            fall_distance: 0.0,
            was_touching_water: false,
            was_eye_in_water: false,
            random: std::sync::Arc::new(std::sync::Mutex::new(
                crate::random::legacy::create_unique(),
            )),
            no_gravity: false,
            is_mob: false,
        }
    }

    /// `getId()`.
    pub fn get_id(&self) -> i32 { self.id }

    /// `setId(id)`.
    pub fn set_id(&mut self, id: i32) { self.id = id; }

    /// `getUUID()`.
    pub fn get_uuid(&self) -> Uuid { self.uuid }

    /// `getType()`.
    pub fn get_type(&self) -> &crate::core::holder::Holder<crate::world::entity::entity_type::EntityType> {
        &self.entity_type
    }

    /// `getLevel()`.
    pub fn get_level(&self) -> Option<&Arc<dyn LevelReader>> {
        self.level.as_ref()
    }

    /// `getPos()`.
    pub fn get_pos(&self) -> Vec3 { self.pos }

    /// `setPos(x, y, z)`.
    pub fn set_pos(&mut self, x: f64, y: f64, z: f64) {
        self.pos = Vec3::new(x, y, z);
        self.update_bounding_box();
    }

    /// `getX()`, `getY()`, `getZ()`.
    /// `getBlockX/Y/Z`.
    pub fn get_block_x(&self) -> i32 { crate::util::mth::floor(self.pos.x) as i32 }
    pub fn get_block_y(&self) -> i32 { crate::util::mth::floor(self.pos.y) as i32 }
    pub fn get_block_z(&self) -> i32 { crate::util::mth::floor(self.pos.z) as i32 }

    /// `position()`.
    pub fn position(&self) -> Vec3 { self.pos }

    /// `blockPosition()`.
    pub fn block_position(&self) -> BlockPos {
        BlockPos::containing(self.pos.x, self.pos.y, self.pos.z)
    }

    pub fn get_x(&self) -> f64 { self.pos.x }
    pub fn get_y(&self) -> f64 { self.pos.y }
    pub fn get_z(&self) -> f64 { self.pos.z }

    /// `getRotation()`.
    pub fn get_rotation(&self) -> Vec2 { self.rotation }

    /// `getYRot()`.
    pub fn get_y_rot(&self) -> f32 { self.rotation.x }

    /// `getXRot()`.
    pub fn get_x_rot(&self) -> f32 { self.rotation.y }

    /// `setRot(yaw, pitch)`.
    pub fn set_rot(&mut self, yaw: f32, pitch: f32) {
        self.rotation = Vec2::new(yaw, pitch);
    }

    /// `setYRot(yaw)`.
    pub fn set_y_rot(&mut self, yaw: f32) { self.rotation.x = yaw; }

    /// `setXRot(pitch)`.
    pub fn set_x_rot(&mut self, pitch: f32) { self.rotation.y = pitch; }

    /// `getVelocity()`.
    pub fn get_velocity(&self) -> Vec3 { self.velocity }

    /// `setDeltaMovement(x, y, z)`.
    pub fn set_delta_movement(&mut self, x: f64, y: f64, z: f64) {
        self.velocity = Vec3::new(x, y, z);
    }

    /// `getDeltaMovement()`.
    pub fn get_delta_movement(&self) -> Vec3 { self.velocity }

    /// `isOnGround()`.
    pub fn is_on_ground(&self) -> bool { self.on_ground }

    /// `isAlive()`.
    pub fn is_alive(&self) -> bool {
        !self.removed && self.death_time == 0
    }

    /// `isRemoved()`.
    pub fn is_removed(&self) -> bool { self.removed }

    /// `remove(reason)`.
    pub fn remove(&mut self, reason: RemovalReason) {
        self.removed = true;
        self.removal_reason = Some(reason);
    }

    /// `getRemovalReason()`.
    pub fn get_removal_reason(&self) -> Option<RemovalReason> {
        self.removal_reason
    }

    /// `tick()` — вызывается каждый тик.
    pub fn tick(&mut self) {
        self.tick_count += 1;
        self.old_pos = self.pos;
        self.old_rotation = self.rotation;
        self.eye_height_old = self.eye_height;
        self.update_eye_height();
        if self.first_tick {
            self.first_tick = false;
        }
    }

    /// `updateEyeHeight()`.
    fn update_eye_height(&mut self) {
        self.eye_height = self.dimensions.eye_height;
    }

    /// `updateBoundingBox()`.
    fn update_bounding_box(&mut self) {
        let half_width = self.dimensions.width / 2.0;
        self.bounding_box = Aabb::new(
            self.pos.x - half_width as f64,
            self.pos.y,
            self.pos.z - half_width as f64,
            self.pos.x + half_width as f64,
            self.pos.y + self.dimensions.height as f64,
            self.pos.z + half_width as f64,
        );
    }

    /// `getBoundingBox()`.
    pub fn get_bounding_box(&self) -> Aabb { self.bounding_box }

    /// `getDimensions()`.
    pub fn get_dimensions(&self) -> crate::world::entity::entity_type::EntityDimensions {
        self.dimensions
    }

    /// `getEyeHeight()`.
    pub fn get_eye_height(&self) -> f32 { self.eye_height }

    /// `getEyeY` — y + eyeHeight.
    pub fn get_eye_y(&self) -> f64 {
        self.pos.y + self.eye_height as f64
    }

    /// `getEyePosition()`.
    pub fn get_eye_position(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.pos.y + self.eye_height as f64, self.pos.z)
    }

    /// `getName()`.
    pub fn get_name(&self) -> Option<&Component> {
        self.name.as_ref().or(self.custom_name.as_ref())
    }

    /// `setCustomName(name)`.
    pub fn set_custom_name(&mut self, name: Option<Component>) {
        self.custom_name = name;
    }

    /// `isInvisible()`.
    pub fn is_invisible(&self) -> bool { self.invisible }

    /// `setInvisible(invisible)`.
    pub fn set_invisible(&mut self, invisible: bool) { self.invisible = invisible; }

    /// `isGlowing()`.
    pub fn is_glowing(&self) -> bool { self.glowing }

    /// `setGlowing(glowing)`.
    pub fn set_glowing(&mut self, glowing: bool) { self.glowing = glowing; }

    /// `isNoPhysics()`.
    pub fn is_no_physics(&self) -> bool { self.no_physics }

    /// `setNoPhysics(noPhysics)`.
    pub fn set_no_physics(&mut self, no_physics: bool) { self.no_physics = no_physics; }

    /// `getSharedFlag(flag)`.
    pub fn get_shared_flag(&self, flag: u8) -> bool {
        (self.shared_flags & flag) != 0
    }

    /// `setSharedFlag(flag, value)`.
    pub fn set_shared_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.shared_flags |= flag;
        } else {
            self.shared_flags &= !flag;
        }
    }

    /// `getData()`.
    pub fn get_data(&self) -> &crate::network::syncher::synched_entity_data::SynchedEntityData {
        &self.data
    }

    /// `getDataMut()`.
    pub fn get_data_mut(&mut self) -> &mut crate::network::syncher::synched_entity_data::SynchedEntityData {
        &mut self.data
    }

    /// `getTags()`.
    pub fn get_tags(&self) -> &[crate::resources::identifier::Identifier] {
        &self.tags
    }

    /// `addTag(tag)`.
    pub fn add_tag(&mut self, tag: crate::resources::identifier::Identifier) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// `removeTag(tag)`.
    pub fn remove_tag(&mut self, tag: &crate::resources::identifier::Identifier) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }

    /// `hasTag(tag)`.
    pub fn has_tag(&self, tag: &crate::resources::identifier::Identifier) -> bool {
        self.tags.contains(tag)
    }

    /// `isPersistenceRequired()`.
    pub fn is_persistence_required(&self) -> bool { self.persistence_required }

    /// `setPersistenceRequired(required)`.
    pub fn set_persistence_required(&mut self, required: bool) { self.persistence_required = required; }

    /// `save(tag)` — сохранить в NBT.
    pub fn save(&self, tag: &mut CompoundTag) {
        tag.put("id".to_string(), Tag::String(self.entity_type.value().map(|t| t.to_string()).unwrap_or_default()));
        tag.put("UUID".to_string(), Tag::String(self.uuid.to_string()));
        tag.put("Pos".to_string(), Tag::List(vec![
            Tag::Double(self.pos.x),
            Tag::Double(self.pos.y),
            Tag::Double(self.pos.z),
        ]));
        tag.put("Rotation".to_string(), Tag::List(vec![
            Tag::Float(self.rotation.x),
            Tag::Float(self.rotation.y),
        ]));
        tag.put("Motion".to_string(), Tag::List(vec![
            Tag::Double(self.velocity.x),
            Tag::Double(self.velocity.y),
            Tag::Double(self.velocity.z),
        ]));
        tag.put("OnGround".to_string(), Tag::Byte(if self.on_ground { 1 } else { 0 }));
        tag.put("Glowing".to_string(), Tag::Byte(if self.glowing { 1 } else { 0 }));
        tag.put("PersistenceRequired".to_string(), Tag::Byte(if self.persistence_required { 1 } else { 0 }));
    }

    /// `load(tag)` — загрузить из NBT.
    pub fn load(&mut self, tag: &CompoundTag) {
        if let Some(pos) = tag.get_list("Pos") {
            if pos.len() == 3 {
                if let (Tag::Double(x), Tag::Double(y), Tag::Double(z)) =
                    (&pos[0], &pos[1], &pos[2])
                {
                    self.pos = Vec3::new(*x, *y, *z);
                }
            }
        }
        if let Some(rot) = tag.get_list("Rotation") {
            if rot.len() == 2 {
                if let (Tag::Float(x), Tag::Float(y)) = (&rot[0], &rot[1]) {
                    self.rotation = Vec2::new(*x, *y);
                }
            }
        }
        if let Some(motion) = tag.get_list("Motion") {
            if motion.len() == 3 {
                if let (Tag::Double(x), Tag::Double(y), Tag::Double(z)) =
                    (&motion[0], &motion[1], &motion[2])
                {
                    self.velocity = Vec3::new(*x, *y, *z);
                }
            }
        }
        if let Some(Tag::Byte(v)) = tag.get("OnGround") {
            self.on_ground = *v != 0;
        }
        if let Some(Tag::Byte(v)) = tag.get("Glowing") {
            self.glowing = *v != 0;
        }
        if let Some(Tag::Byte(v)) = tag.get("PersistenceRequired") {
            self.persistence_required = *v != 0;
        }
        if let Some(Tag::String(s)) = tag.get("UUID") {
            if let Some(u) = Uuid::from_string(s) {
                self.uuid = u;
            }
        }
    }

    /// `getBlockPos()` — позиция блока.
    pub fn get_block_pos(&self) -> BlockPos {
        BlockPos::new(self.pos.x as i32, self.pos.y as i32, self.pos.z as i32)
    }

    /// `getChunkPos()` — позиция чанка.
    pub fn get_chunk_pos(&self) -> crate::world::level::chunk_pos::ChunkPos {
        crate::world::level::chunk_pos::ChunkPos::new(
            (self.pos.x as i32) >> 4,
            (self.pos.z as i32) >> 4,
        )
    }

    /// `getSectionPos()` — позиция секции.
    pub fn get_section_pos(&self) -> SectionPos {
        SectionPos::of(&self.get_block_pos())
    }

    /// `distanceToSqr(other)` — расстояние до другой сущности.
    pub fn distance_to_sqr(&self, other: &Entity) -> f64 {
        let dx = self.pos.x - other.pos.x;
        let dy = self.pos.y - other.pos.y;
        let dz = self.pos.z - other.pos.z;
        dx * dx + dy * dy + dz * dz
    }

    /// `isAliveAndWell()`.
        /// `spawnAtLocation(stack)` — создание `ItemEntity` в позиции
    /// сущности и `addFreshEntity` в уровень (vanilla `Entity`).
    /// Возвращает созданную сущность-drope (аналог `ItemEntity`).
    pub fn spawn_at_location(
        &mut self,
        stack: crate::world::item::item_stack::ItemStack,
    ) -> Option<crate::world::entity::item_entity::ItemEntity> {
        self.spawn_at_location_with_offset(stack, 0.0)
    }

    /// `spawnAtLocation(stack, yOffset)`.
    pub fn spawn_at_location_with_offset(
        &mut self,
        stack: crate::world::item::item_stack::ItemStack,
        y_offset: f64,
    ) -> Option<crate::world::entity::item_entity::ItemEntity> {
        if stack.is_empty() {
            return None;
        }
        let level = self.level.clone()?;
        let mut item_entity = crate::world::entity::item_entity::ItemEntity::new(
            Some(level.clone()),
            self.pos.x,
            self.pos.y + y_offset,
            self.pos.z,
            stack,
        );
        item_entity.set_default_pick_up_delay();
        level.add_fresh_item_entity(item_entity.clone());
        Some(item_entity)
    }

    /// `killedEntity(level, killedEntity, damageSource)` — база
    /// возвращает `true` (Player переопределяет для kill credit).
    pub fn killed_entity(
        &self,
        _level: &dyn crate::world::level::level_reader::LevelReader,
        _killed_entity: &crate::world::entity::entity::Entity,
        _damage_source: &crate::world::damagesource::damage_source::DamageSource,
    ) -> bool {
        true
    }

    /// `awardKillScore(killedEntity, damageSource)` — база no-op
    /// (диспатч kill-credit; `LivingEntity`/`Player` уточняют).
    pub fn award_kill_score(
        &mut self,
        _killed_entity: &crate::world::entity::entity::Entity,
        _damage_source: &crate::world::damagesource::damage_source::DamageSource,
    ) {
    }

    /// `gameEvent(event, pos)` — делегат в `LevelReader.game_event`.
    pub fn game_event(
        &mut self,
        event: &crate::world::level::gameevent::game_event::GameEvent,
    ) {
        if let Some(level) = self.level.clone() {
            level.game_event(event, crate::core::block_pos::BlockPos::containing_vec3(&self.pos));
        }
    }

pub fn is_alive_and_well(&self) -> bool {
        self.is_alive() && !self.is_removed()
    }

    /// `isAttackable()`.
    pub fn is_attackable(&self) -> bool {
        self.is_alive() && !self.is_removed()
    }

    /// `isPushable()`.
    pub fn is_pushable(&self) -> bool { true }

    /// `isPickable()`.
    pub fn is_pickable(&self) -> bool { true }

    /// `isLivingEntity()` — `true` для LivingEntity-наследников.
    pub fn is_living_entity(&self) -> bool { false }

    /// `getMainHandItem()` — предмет в основной руке (`None` у базовой Entity).
    pub fn get_main_hand_item(&self) -> Option<crate::world::item::item_stack::ItemStack> { None }

    /// `isSpectator()`.
    pub fn is_spectator(&self) -> bool { false }

    /// `isCreative()` — creative-режим (Player; default false).
    pub fn is_creative(&self) -> bool { false }

    /// `getLookAngle()` — единичный вектор взгляда.
    pub fn get_look_angle(&self) -> Vec3 {
        self.calculate_view_vector(self.get_x_rot(), self.get_y_rot())
    }

    /// `getKnownMovement()` — deltaMovement (известное движение).
    pub fn get_known_movement(&self) -> Vec3 {
        self.velocity
    }

    /// `getRandom()` — мутабельный доступ к `Entity.random`.
    pub fn random_mut(
        &self,
    ) -> std::sync::MutexGuard<'_, Box<dyn crate::random::RandomSource>> {
        self.random.lock().expect("entity random poisoned")
    }

    /// `entityRandom()` — shared-доступ к `Entity.random`.
    pub fn entity_random(
        &self,
    ) -> std::sync::Arc<std::sync::Mutex<Box<dyn crate::random::RandomSource>>> {
        self.random.clone()
    }

    /// `isCrouching()`.
    pub fn is_crouching(&self) -> bool { false }

    /// `isSprinting()`.
    pub fn is_sprinting(&self) -> bool { false }

    /// `isSwimming()`.
    pub fn is_swimming(&self) -> bool { false }

    /// `isFallFlying()`.
    pub fn is_fall_flying(&self) -> bool { false }

    /// `isVisuallyCrawling()`.
    pub fn is_visually_crawling(&self) -> bool { false }

    /// `isSleeping()`.
    pub fn is_sleeping(&self) -> bool { false }

    /// `isPassenger()`.
    pub fn is_passenger(&self) -> bool { false }

    /// `isVehicle()`.
    pub fn is_vehicle(&self) -> bool { false }

    /// `getPassengers()`.
    pub fn get_passengers(&self) -> Vec<&Entity> { Vec::new() }

    /// `getVehicle()`.
    pub fn get_vehicle(&self) -> Option<&Entity> { None }

    /// `onGround()`.
    pub fn on_ground(&self) -> bool { self.on_ground }

    /// `setOnGround(onGround)`.
    pub fn set_on_ground(&mut self, on_ground: bool) { self.on_ground = on_ground; }

    /// `isHorizontalCollision()`.
    pub fn is_horizontal_collision(&self) -> bool { self.horizontal_collision }

    /// `isVerticalCollision()`.
    pub fn is_vertical_collision(&self) -> bool { self.vertical_collision }

    /// `hurtTime()`.
    pub fn hurt_time(&self) -> i32 { self.hurt_time }

    /// `hurtDuration()`.
    pub fn hurt_duration(&self) -> i32 { self.hurt_duration }

    /// `deathTime()`.
    pub fn death_time(&self) -> i32 { self.death_time }

    /// `tickCount()`.
    pub fn tick_count(&self) -> i32 { self.tick_count }

    /// `isFirstTick()`.
    pub fn is_first_tick(&self) -> bool { self.first_tick }

    /// `getNameString()`.
    pub fn get_name_string(&self) -> String {
        self.name.as_ref().map(|_| "entity".to_string()).unwrap_or_else(|| "entity".to_string())
    }

    /// `getScoreboardName()`.
    pub fn get_scoreboard_name(&self) -> String { self.get_name_string() }

    /// `getDisplayName()`.
    pub fn get_display_name(&self) -> Component {
        self.name.clone().unwrap_or_else(|| Component::literal(&self.get_scoreboard_name()))
    }

    /// `getTeam()`.
    pub fn get_team(&self) -> Option<&crate::world::scores::team::PlayerTeam> { None }

    /// `isAlliedTo(other)`.
    pub fn is_allied_to(&self, _other: &Entity) -> bool { false }

    /// `canBeCollidedWith(box)`.
    pub fn can_be_collided_with(&self, _box: Aabb) -> bool {
        self.is_alive() && !self.is_removed()
    }

    /// `canCollideWith(box)`.
    pub fn can_collide_with(&self, _box: Aabb) -> bool {
        self.is_alive() && !self.is_removed()
    }

    /// `canBeSeenAsEnemy()`.
    pub fn can_be_seen_as_enemy(&self) -> bool {
        self.is_alive() && !self.is_removed()
    }

    /// `isFree()`.
    pub fn is_free(&self) -> bool {
        self.is_alive() && !self.is_removed()
    }

    /// `isEffectiveAi()`.
    pub fn is_effective_ai(&self) -> bool { true }

    /// `maxUpStep()`.
    pub fn max_up_step(&self) -> f32 { 0.0 }

    /// `getWalkTargetValue(blockPos, direction)`.
    pub fn get_walk_target_value(&self, _block_pos: &BlockPos, _direction: Direction) -> f32 { 0.0 }

    /// `getBoundingBoxForCulling()`.
    pub fn get_bounding_box_for_culling(&self) -> Aabb { self.bounding_box }

    /// `getPose()`.
    pub fn get_pose(&self) -> crate::world::entity::pose::Pose {
        crate::world::entity::pose::Pose::Standing
    }

    /// `isInside(blockPos, blockState)`.
    pub fn is_inside(&self, _pos: &BlockPos, _state: &BlockState) -> bool { false }

    /// `isInWall()`.
    pub fn is_in_wall(&self) -> bool { false }

    /// `isSilent()`.
    pub fn is_silent(&self) -> bool { false }

    /// `setSilent(silent)`.
    pub fn set_silent(&mut self, _silent: bool) {}

    /// `isNoGravity()`.
    pub fn is_no_gravity(&self) -> bool { self.no_gravity }

    /// `setNoGravity(noGravity)`.
    pub fn set_no_gravity(&mut self, no_gravity: bool) { self.no_gravity = no_gravity; }

    /// `isOnFire()`.
    pub fn is_on_fire(&self) -> bool { false }

    /// `isInWater()`.
    pub fn is_in_water(&self) -> bool { false }

    /// `isBaby()` — false для базовой entity (как в Java).
    pub fn is_baby(&self) -> bool { false }

    /// `isFlying()` — полёт (для Player переопределяется).
    pub fn is_flying(&self) -> bool { false }

    /// `isInLava()`.
    pub fn is_in_lava(&self) -> bool { false }

    /// `getFluidJumpThreshold` — 0.4 (как в Entity).
    pub fn get_fluid_jump_threshold(&self) -> f32 {
        0.4
    }

    /// `getFluidHeight(tag)` — высота жидкости внутри bb
    /// (скан блоков в shrunk bb; source=1.0, flowing≈0.889).
    pub fn get_fluid_height(&self, water: bool) -> f32 {
        let Some(level) = &self.level else { return 0.0 };
        let bb = self.get_bounding_box();
        let (min_x, min_y, min_z) = (bb.min_x + 0.001, bb.min_y + 0.001, bb.min_z + 0.001);
        let (max_x, max_y, max_z) = (bb.max_x - 0.001, bb.max_y - 0.001, bb.max_z - 0.001);
        let mut height = 0.0f32;
        for x in crate::util::mth::floor(min_x)..=crate::util::mth::floor(max_x) {
            for y in crate::util::mth::floor(min_y)..=crate::util::mth::floor(max_y) {
                for z in crate::util::mth::floor(min_z)..=crate::util::mth::floor(max_z) {
                    let fs = level
                        .get_fluid_state(BlockPos::new(x, y, z))
                        .unwrap_or_else(crate::world::level::block::state::fluid_state::FluidState::empty);
                    if fs.is_empty() {
                        continue;
                    }
                    let matches = if water {
                        fs.is_water()
                            || fs.is_in_named_tag(&crate::tags::fluid_tags::FluidTags::water())
                    } else {
                        fs.is_lava()
                    };
                    if matches {
                        height = height.max(if fs.fluid.is_source { 1.0 } else { 0.888_888_9 });
                    }
                }
            }
        }
        height
    }

    /// `isEyeInFluid(FluidTags.WATER/LAVA)` — жидкость в точке глаз.
    pub fn is_eye_in_fluid(&self, water: bool) -> bool {
        let Some(level) = &self.level else { return false };
        let eye = self.get_eye_position();
        let bp = BlockPos::containing(eye.x, eye.y, eye.z);
        level
            .get_fluid_state(bp)
            .map(|f| {
                if water {
                    f.is_water() || f.is_in_named_tag(&crate::tags::fluid_tags::FluidTags::water())
                } else {
                    f.is_lava()
                }
            })
            .unwrap_or(false)
    }

    /// `isUnderWater()`.
    pub fn is_under_water(&self) -> bool { false }

    /// `getHealth()`.
    pub fn get_health(&self) -> f32 { 0.0 }

    /// `getMaxHealth()`.
    pub fn get_max_health(&self) -> f32 { 0.0 }

    /// `getAbsorptionAmount()`.
    pub fn get_absorption_amount(&self) -> f32 { 0.0 }

    /// `getArmorValue()`.
    pub fn get_armor_value(&self) -> i32 { 0 }

    /// `hurt(source, amount)`.
    pub fn hurt(&mut self, _source: &crate::world::damagesource::damage_type::DamageType, _amount: f32) -> bool { false }

    /// `heal(amount)`.
    pub fn heal(&mut self, _amount: f32) {}

    /// `isInvulnerable()`.
    pub fn is_invulnerable(&self) -> bool { self.invulnerable }

    /// `calculateViewVector(xRot, yRot)` — единичный вектор взгляда.
    /// `getViewVector(partialTicks)`.
    pub fn get_view_vector(&self, _partial_ticks: f32) -> Vec3 {
        self.calculate_view_vector(self.get_x_rot(), self.get_y_rot())
    }

    pub fn calculate_view_vector(&self, x_rot: f32, y_rot: f32) -> Vec3 {
        let pitch = x_rot.to_radians();
        let yaw = (-y_rot).to_radians();
        let cp = pitch.cos();
        Vec3::new(
            (-yaw.sin() * cp) as f64,
            (-pitch.sin()) as f64,
            (yaw.cos() * cp) as f64,
        )
    }

    /// `isPlayer()` — переопределяется в `Player`.
    pub fn is_player(&self) -> bool { false }

    /// `playSound(sound, volume, pitch)` — через level.
    pub fn play_sound(
        &mut self,
        sound: Option<&crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>,
        volume: f32,
        pitch: f32,
    ) {
        let Some(sound) = sound else { return; };
        let Some(id) = sound.value().map(|e| e.location.clone()).or_else(|| {
            sound.key().map(|k| k.identifier.clone())
        }) else { return; };
        if let Some(level) = self.level.clone() {
            level.play_sound(self.pos, &id, volume, pitch);
        }
    }

    /// `isInvulnerableTo(source)`.
    pub fn is_invulnerable_to(&self, _source: &crate::world::damagesource::damage_type::DamageType) -> bool { false }

    /// `isFreezing()`.
    pub fn is_freezing(&self) -> bool { false }

    /// `isInClouds()`.
    pub fn is_in_clouds(&self) -> bool { false }

    /// `isInPowderSnow()`.
    pub fn is_in_powder_snow(&self) -> bool { false }

    /// `isPassengerOfSameVehicle(other)`.
    pub fn is_passenger_of_same_vehicle(&self, _other: &Entity) -> bool { false }

    /// `isMultipartEntity()`.
    pub fn is_multipart_entity(&self) -> bool { false }

    /// `getParts()`.
    pub fn get_parts(&self) -> Vec<&Entity> { Vec::new() }

    /// `getSelfAndPassengers()`.
    pub fn get_self_and_passengers(&self) -> Vec<&Entity> { vec![self] }

    /// `hasPassenger(entity)`.
    pub fn has_passenger(&self, _entity: &Entity) -> bool { false }

    /// `getFirstPassenger()`.
    pub fn get_first_passenger(&self) -> Option<&Entity> { None }

    /// `getControllingPassenger()`.
    pub fn get_controlling_passenger(&self) -> Option<&Entity> { None }
}
