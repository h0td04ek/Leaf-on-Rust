//! Перенос `net.minecraft.world.level.LevelReader` — read-only
//! доступ к уровню для предикатов и паттернов.

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::resources::identifier::Identifier;
use crate::world::level::block::entity::block_entity::BlockEntityAccess;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use std::sync::Arc;

/// `LevelReader` — минимальный read-интерфейс уровня.
///
/// Реализуется `Level`/`ServerLevel`/`WorldGenLevel` по мере порта.
pub trait LevelReader: Send + Sync {
    /// `getBlockState(pos)` — `None` если чанк не загружен
    /// (в Java возвращает air — здесь Option честнее).
    fn get_block_state(&self, pos: BlockPos) -> Option<BlockState>;

    /// `getFluidState(pos)`.
    fn get_fluid_state(&self, pos: BlockPos) -> Option<FluidState> {
        None
    }

    /// `getBlockEntity(pos)`.
    fn get_block_entity(&self, pos: BlockPos) -> Option<Arc<dyn BlockEntityAccess>> {
        None
    }

    /// `hasChunkAt(pos)`.
    fn has_chunk_at(&self, pos: BlockPos) -> bool;

    /// `isLoaded(pos)` — `hasChunk` + позиция внутри build height.
    fn is_loaded(&self, pos: BlockPos) -> bool {
        self.has_chunk_at(pos)
    }

    /// `getMinBuildHeight`.
    fn min_build_height(&self) -> i32 {
        -64
    }

    /// `getHeight`.
    fn height(&self) -> i32 {
        384
    }

    /// `getSeaLevel`.
    fn sea_level(&self) -> i32 {
        self.min_build_height() + 63
    }

    /// `isInsideBuildHeight(y)`.
    fn is_inside_build_height(&self, y: i32) -> bool {
        y >= self.min_build_height() && y < self.min_build_height() + self.height()
    }

    /// `getBiome(pos)` — биом в позиции (`None` — не загружен).
    fn biome_at(
        &self,
        _pos: BlockPos,
    ) -> Option<&Holder<crate::world::level::biome::biome::Biome>> {
        None
    }

    /// `getStructureAt(pos, holderSet)` — структура в holder-set.
    fn structure_at(
        &self,
        _pos: BlockPos,
        _structures: &HolderSet<crate::world::level::levelgen::structure::Structure>,
    ) -> Option<bool> {
        None
    }

    /// `dimension()` — identifier измерения.
    fn dimension_id(&self) -> Option<&Identifier> {
        None
    }

    /// `isSkyDim`/`isSmokeyAt(pos)` — дымность (незер).
    fn is_smokey_at(&self, _pos: BlockPos) -> bool {
        false
    }

    /// `canSeeSky(pos)`.
    fn can_see_sky(&self, _pos: BlockPos) -> bool {
        true
    }

    /// `getGameTime()` — world time in ticks.
    fn game_time(&self) -> i64 {
        0
    }

    /// `loadedAndEntityCanStandOn(pos, entity)` — позиция
    /// загружена и сущность может стоять (InsideBrownianWalk).
    /// Default: true.
    fn loaded_and_entity_can_stand_on(&self, _pos: BlockPos, _mob: &crate::world::entity::mob::Mob) -> bool {
        true
    }

    /// `GameRules.FORGIVE_DEAD_PLAYERS` — прощать мёртвых
    /// игроков (StopBeingAngryIfTargetDead). Default: true.
    fn forgive_dead_players(&self) -> bool {
        true
    }

    /// `isBrightOutside()` — `dimensionType.hasSkyLight() &&
    /// isDay()`. Default: false (переопределяется уровнем).
    fn is_bright_outside(&self) -> bool {
        false
    }

    /// Purpur/Leaf `purpurConfig.creeperEncircleTarget` —
    /// обход цели во время вздутия крипера (SwellGoal).
    /// Default: false.
    fn purpur_creeper_encircle_target(&self) -> bool {
        false
    }

    /// `getScoreboard()` — scoreboard уровня (`None` пока нет
    /// реализации уровня).
    fn scoreboard(&self) -> Option<&crate::world::scores::scoreboard::Scoreboard> {
        None
    }

    /// `isRaining()` / `isThundering()` — погода (default false).
    fn is_raining(&self) -> bool {
        false
    }
    fn is_thundering(&self) -> bool {
        false
    }

    /// `getBrightness(LightLayer, pos)` — до порта lighting
    /// возвращает максимум (как в fully-lit мире).
    fn get_brightness(&self, _layer: crate::world::level::light_layer::LightLayer, _pos: BlockPos) -> i32 {
        15
    }

    /// `isWaterAt(pos)` — `getFluidState(pos).is(Fluids.WATER)`.
    /// `playSound` — проиграть звук в позиции (default no-op;
    /// серверная реализация отправляет пакет клиентам).
    fn play_sound(
        &self,
        _pos: crate::world::phys::vec3::Vec3,
        _sound: &crate::resources::identifier::Identifier,
        _volume: f32,
        _pitch: f32,
    ) {
    }

    fn is_water_at(&self, pos: BlockPos) -> bool {
        self.get_fluid_state(pos).map(|f| f.is_water()).unwrap_or(false)
    }

    /// `addFreshEntity(entity)` — спавн сущности в мир
    /// (default no-op; `ServerLevel` реализует через entity manager).
    fn add_fresh_entity(&self, _entity: crate::world::entity::entity::Entity) {}

    /// `spawnLightningBolt`-эквивалент — призыв молнии
    /// (WindChargedMobEffect). Default no-op до порта LightningBolt.
    fn spawn_lightning_bolt(&self, _pos: crate::world::phys::vec3::Vec3) {}

    /// `addFreshEntity(ItemEntity)` — спавн дропа предмета.
    fn add_fresh_item_entity(
        &self,
        _entity: crate::world::entity::item_entity::ItemEntity,
    ) {
    }

    /// `addFreshEntity(ExperienceOrb)` — спавн сферы опыта.
    fn add_fresh_experience_orb(
        &self,
        _entity: crate::world::entity::experience_orb::ExperienceOrb,
    ) {
    }

    /// `broadcastEntityEvent(entity, eventId)` — broadcast entity event
    /// (EntityEvent.DEATH и т.п.). Default no-op до порта сервера.
    fn broadcast_entity_event(
        &self,
        _entity_id: i32,
        _event_id: u8,
    ) {
    }

    /// `getGameRules().get(GameRules.MOB_GRIEFING)` — default true.
    fn mob_griefing(&self) -> bool {
        true
    }

    /// `level.purpurConfig.llamaJoinCaravans` — Purpur-конфиг
    /// караванов лам. Default: true.
    fn llama_caravans_enabled(&self) -> bool {
        true
    }

    // ---- Raid (`Raid.tick`/`Raids`) --------------------------

    /// `ServerLevel.isVillage(pos)` — pos в пределах деревни.
    fn is_village_at(&self, _pos: BlockPos) -> bool {
        false
    }

    /// `ServerLevel.isVillage(SectionPos)` — секция деревни.
    fn is_village_section(
        &self,
        _section: crate::core::section_pos::SectionPos,
    ) -> bool {
        false
    }

    /// `ServerLevel.hasChunksAt(x1,z1,x2,z2)` — все чанки есть.
    fn has_chunks_at_range(&self, _x1: i32, _z1: i32, _x2: i32, _z2: i32) -> bool {
        true
    }

    /// `ServerLevel.isPositionEntityTicking(pos)`.
    fn is_position_entity_ticking(&self, _pos: BlockPos) -> bool {
        true
    }

    /// `SpawnPlacements` проверка для RAVAGER (EVENT-спавн).
    fn ravager_spawn_position_ok(&self, _pos: BlockPos) -> bool {
        true
    }

    /// `EntityType.create(level, EVENT)` → LivingEntity-view
    /// свежесозданной сущности нужного типа. Default: нет спавна.
    fn spawn_entity_of_type(
        &self,
        _type_id: &Identifier,
        _pos: BlockPos,
    ) -> Option<crate::world::entity::living_entity::LivingEntity> {
        None
    }

    /// `raider.snapTo/finalizeSpawn/applyRaidBuffs/setOnGround/
    /// addFreshEntityWithPassengers` — размещение рейдера в мире.
    fn place_raider_at(
        &self,
        _raider: &mut crate::world::entity::living_entity::LivingEntity,
        _pos: BlockPos,
        _wave: i32,
    ) {
    }

    /// `raider.startRiding(vehicle)` — посадить рейдера.
    fn make_passenger(
        &self,
        _rider: &mut crate::world::entity::living_entity::LivingEntity,
        _vehicle_uuid: crate::util::uuid::Uuid,
    ) {
    }

    /// `Raid.playSound` — RAID_HORN звуковой пакет игрокам рейда.
    fn play_raid_horn(&self, _pos: BlockPos, _random_seed: i64) {}

    /// `Raid` victory: `addEffect(HERO_OF_THE_VILLAGE)` +
    /// stats/criteria для heroesOfTheVillage.
    fn grant_hero_of_the_village(
        &self,
        _entity: &crate::world::entity::entity::Entity,
        _raid_omen_level: i32,
    ) {
    }

    /// `ServerLevel.getRaids()` — доступ к `Raids` менеджеру.
    /// Default: нет сохранённых рейдов.
    fn raids(&self) -> Option<std::rc::Rc<std::cell::RefCell<crate::world::entity::raid::Raids>>> {
        None
    }

    /// `level.getRaidAt(pos)` — рейд, чей center ближе всех.
    fn raid_at(&self, _pos: BlockPos) -> Option<std::rc::Rc<std::cell::RefCell<crate::world::entity::raid::Raid>>> {
        None
    }

    /// `GameRules.RAIDS` — рейды включены. Default: true.
    fn raids_enabled(&self) -> bool {
        true
    }

    /// `EnvironmentAttributes.CAN_START_RAID` — можно ли начать
    /// рейд в pos. Default: true.
    fn environment_attribute_can_start_raid(&self, _pos: BlockPos) -> bool {
        true
    }

    /// `getPoiManager` village-POI центризация для
    /// `createOrExtendRaid` (POI VILLAGE radius 64 → centroid).
    /// Default: None → используется pos.
    fn village_center_near(&self, _pos: BlockPos) -> Option<BlockPos> {
        None
    }

    // ---- Village goals (`MoveThroughVillageGoal` и др.) ----

    /// `ServerLevel.isCloseToVillage(pos, distance)`.
    fn is_close_to_village(&self, _pos: BlockPos, _distance: i32) -> bool {
        false
    }

    /// `ServerLevel.sectionsToVillage(sectionPos)` — расстояние
    /// до ближайшей деревни в секциях (StrollThroughVillageGoal).
    fn sections_to_village(
        &self,
        _section: crate::core::section_pos::SectionPos,
    ) -> i32 {
        i32::MAX
    }

    /// `BehaviorUtils.findSectionClosestToVillage(level, section,
    /// range)` — ближайшая village-секция (MoveBackToVillageGoal).
    fn find_section_closest_to_village(
        &self,
        _section: crate::core::section_pos::SectionPos,
        _range: i32,
    ) -> Option<crate::core::section_pos::SectionPos> {
        None
    }

    /// `level.getHeightmapPos(HEIGHTMAP_TYPE, pos)` — pos с y
    /// из heightmap (MOTION_BLOCKING_NO_LEAVES для stroll).
    fn get_heightmap_pos(
        &self,
        ty: crate::world::level::levelgen::heightmap::HeightmapType,
        pos: BlockPos,
    ) -> BlockPos {
        let h = self.get_height(ty, pos.x, pos.z);
        BlockPos::new(pos.x, h, pos.z)
    }

    /// `getPoiManager().find(VILLAGE, predicate, center, range,
    /// IS_OCCUPIED)` — ближайший village-POI, не входящий в
    /// `exclude` (MoveThroughVillageGoal `hasNotVisited`).
    fn village_poi_near(
        &self,
        _center: BlockPos,
        _max_dist: i32,
        _exclude: &[BlockPos],
    ) -> Option<BlockPos> {
        None
    }

    /// `getPoiManager().find(..., center=section.center(), range,
    /// IS_OCCUPIED)` — POI внутри секции (radius `max_dist` от
    /// центра), для `GolemRandomStrollInVillageGoal`.
    fn poi_in_section(
        &self,
        _center: BlockPos,
        _max_dist: i32,
    ) -> Option<Vec<BlockPos>> {
        None
    }

    /// `PoiManager.find(HOME, ..., IS_OCCUPIED)` — ближайший
    /// HOME-POI (NearestBedSensor; AcquirePoi-путь — упрощённо
    /// до ближайшего POI без path-проверки до порта PoiManager).
    fn home_poi_near(&self, _center: BlockPos, _max_dist: i32) -> Option<BlockPos> {
        None
    }

    /// `CampfireBlock.isLitCampfire(state)` — soul_campfire горит
    /// (PiglinSpecificSensor.isValidRepellent).
    fn is_lit_campfire(&self, pos: BlockPos) -> bool {
        self.get_block_state(pos)
            .map(|s| {
                let id = &s.block.id;
                if !id.path.contains("campfire") {
                    return false;
                }
                // property "lit" ещё не смоделирована — campfire
                // id сам по себе lit-состояние не различает;
                // до порта BlockState properties считаем lit.
                true
            })
            .unwrap_or(false)
    }

    /// `DoorBlock.isWoodenDoor(level, pos)` — `state.is(
    /// WOODEN_DOORS)` (MoveThroughVillageGoal door-scan пути).
    fn is_wooden_door(&self, pos: BlockPos) -> bool {
        self.get_block_state(pos)
            .map(|s| {
                crate::tags::is_block_in_named_tag(
                    &s.block.id,
                    &crate::tags::block_tags::BlockTags::wooden_doors(),
                )
            })
            .unwrap_or(false)
    }

    /// `gameEvent(event, pos, context)` — гейм-эвент
    /// (sculk sensors, allays и т.п.). Default no-op до порта
    /// game-event диспетчера.
    fn game_event(
        &self,
        _event: &crate::world::level::gameevent::game_event::GameEvent,
        _pos: crate::core::block_pos::BlockPos,
    ) {
    }

    /// `getGameRules().get(GameRules.MOB_DROPS)` — default true.
    fn mob_drops(&self) -> bool {
        true
    }

    /// `getGameRules().get(GameRules.MAX_ENTITY_CRAMMING)` — default 24.
    fn max_entity_cramming(&self) -> i32 {
        24
    }

    /// `getEntities(type, aabb, predicate, output, limit)`-эквивалент —
    /// подсчёт сущностей типа рядом (для OozingMobEffect cramming).
    /// Default 0 до порта entity manager.
    fn count_nearby_entities(
        &self,
        _entity_type: &Identifier,
        _center: crate::world::phys::vec3::Vec3,
        _radius: f64,
    ) -> i32 {
        0
    }

    /// `canBeReplaced`-проверка для установки блока (cobweb и т.п.).
    fn can_replace_at(&self, _pos: BlockPos) -> bool {
        false
    }

    /// `isFaceSturdy(pos, direction)` — грань блока прочная.
    fn is_face_sturdy(
        &self,
        _pos: BlockPos,
        _direction: crate::core::direction::Direction,
    ) -> bool {
        false
    }

    /// `setBlock(pos, BlockState, flags)` — установка состояния
    /// блока (enchantment effects: ReplaceBlock/ReplaceDisk/
    /// SetBlockProperties). Default: через `set_block_by_id`.
    fn set_block_state(
        &self,
        pos: BlockPos,
        state: &crate::world::level::block::state::block_state::BlockState,
    ) -> bool {
        self.set_block_by_id(pos, &state.block.id)
    }

    /// `sendParticles(particle, x, y, z, count, dx, dy, dz, speed)`
    /// (enchantment SpawnParticlesEffect). Default no-op.
    fn send_particles(
        &self,
        _particle: &crate::core::particles::particle_options::ParticleOptions,
        _x: f64,
        _y: f64,
        _z: f64,
        _count: i32,
        _dx: f64,
        _dy: f64,
        _dz: f64,
        _speed: f64,
    ) {
    }

    /// `explode(entity, damageSource, x, y, z, radius, fire,
    /// interaction)` (ExplodeEffect). Default no-op.
    #[allow(clippy::too_many_arguments)]
    fn explode(
        &self,
        _entity: Option<crate::world::entity::entity::Entity>,
        _damage_source: Option<crate::world::damagesource::damage_source::DamageSource>,
        _pos: crate::world::phys::vec3::Vec3,
        _radius: f32,
        _create_fire: bool,
        _interaction: crate::world::item::enchantment::effects::enchantment_entity_effect::ExplosionInteraction,
        _knockback_multiplier: Option<f32>,
    ) {
    }

    /// `gameEvent(holder, pos)` по holder'у (enchantment
    /// triggerGameEvent). Default: резолвит value через `holder()`.
    fn game_event_value(
        &self,
        _event: &crate::world::level::gameevent::game_event::GameEvent,
        pos: BlockPos,
    ) {
    }

    /// `runFunction(function, entity, pos)` — RunFunction effect.
    /// Default no-op до порта command dispatcher.
    fn run_function(
        &self,
        _function: Identifier,
        _entity: &crate::world::entity::living_entity::LivingEntity,
        _origin: crate::world::phys::vec3::Vec3,
    ) {
    }

    /// `summonEntity(entityType holder, pos, joinTeam)` —
    /// SummonEntityEffect. Default no-op до порта EntityType.create.
    fn summon_entity(
        &self,
        _entity_type: &crate::core::holder::Holder<crate::world::entity::entity_type::EntityType>,
        _pos: BlockPos,
        _join_team: bool,
    ) {
    }

    /// `setBlock(pos, blockId)` — установка блока по registry-id
    /// (cobweb spawn WeavingMobEffect). Default no-op.
    fn set_block_by_id(&self, _pos: BlockPos, _block_id: &Identifier) -> bool {
        false
    }

    /// `levelEvent(eventId, pos, data)` — level event
    /// (ANIMATION_SPAWN_COBWEB=3018 и т.п.). Default no-op.
    fn level_event(&self, _event_id: i32, _pos: BlockPos, _data: i32) {}

    /// `destroyBlockProgress(entityId, pos, progress)` —
    /// прогресс разрушения блока (BreakDoorGoal). Default no-op.
    fn destroy_block_progress(&self, _entity_id: i32, _pos: BlockPos, _progress: i32) {}

    /// `ChestBlockEntity.getOpenCount(level, pos)` — число
    /// открытых игроков у сундука (CatSitOnBlockGoal).
    /// Default: 0 (закрыт).
    fn container_open_count(&self, _pos: BlockPos) -> i32 {
        0
    }

    /// `destroyBlock(pos, drop)` — разрушение блока (EatBlockGoal
    /// и др.). Default: установить air.
    fn destroy_block(&self, pos: BlockPos, _drop: bool) -> bool {
        self.set_block_state(
            pos,
            &crate::world::level::block::state::block_state::BlockState::air(),
        )
    }

    /// `BlockCollisions` — collision-AABB блоков в box
    /// (`CollisionGetter.getBlockCollisions`). Default-реализация
    /// итерирует `get_block_state` + `get_collision_shape`.
    fn block_collisions_in_box(
        &self,
        bb: &crate::world::phys::aabb::Aabb,
    ) -> Vec<crate::world::phys::aabb::Aabb> {
        use crate::core::block_pos::BlockPos;
        let mut out = Vec::new();
        let min_x = bb.min_x.floor() as i32;
        let min_y = bb.min_y.floor() as i32;
        let min_z = bb.min_z.floor() as i32;
        let max_x = bb.max_x.floor() as i32;
        let max_y = bb.max_y.floor() as i32;
        let max_z = bb.max_z.floor() as i32;
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let pos = BlockPos::new(x, y, z);
                    let Some(state) = self.get_block_state(pos) else {
                        continue;
                    };
                    if !state.block.properties.has_collision {
                        continue;
                    }
                    let shape = state.collision_shape_unleveled();
                    if shape.is_empty() {
                        continue;
                    }
                    let (fx, fy, fz) = (x as f64, y as f64, z as f64);
                    for a in shape.to_aabbs() {
                        out.push(crate::world::phys::aabb::Aabb::new(
                            fx + a.min_x,
                            fy + a.min_y,
                            fz + a.min_z,
                            fx + a.max_x,
                            fy + a.max_y,
                            fz + a.max_z,
                        ));
                    }
                }
            }
        }
        out
    }

    /// `EntityCollisions` — collision-AABB других сущностей
    /// (`CollisionGetter.getEntityCollisions`). Default: нет
    /// (ServerLevel переопределит по entity-списку).
    fn entity_collisions(
        &self,
        _bb: &crate::world::phys::aabb::Aabb,
        _ignore: &crate::world::entity::entity::Entity,
    ) -> Vec<crate::world::phys::aabb::Aabb> {
        Vec::new()
    }

    /// `generate_loot`-эквивалент — наполнение `LootTable` предметами
    /// для `dropFromLootTable`. Default — пустой список до порта
    /// `LootTable`/`LootPool`.
    fn generate_loot(
        &self,
        _table: &crate::resources::identifier::Identifier,
        _params: &crate::world::level::storage::loot::loot_context::LootParams,
    ) -> Vec<crate::world::item::item_stack::ItemStack> {
        Vec::new()
    }

    /// `ServerLevel.getPathTypeCache` — кэш `PathType` уровня.
    /// Default — нет кэша (контекст будет считать напрямую).
    fn path_type_cache(&self) -> Option<&crate::world::level::pathfinder::path_type_cache::PathTypeCache> {
        None
    }

    /// `CollisionGetter.noCollision(entity, aabb)` — нет ли
    /// коллизий у сущности с блоками/сущностями в AABB.
    /// Default — true (коллизий нет; конкретный Level переопределит).
    fn no_collision(&self, _aabb: &crate::world::phys::aabb::Aabb) -> bool {
        true
    }

    /// `getHeight(Heightmap.Types, x, z)` — высота по heightmap.
    /// Default: скан сверху вниз по `is_block_counted`.
    fn get_height(
        &self,
        ty: crate::world::level::levelgen::heightmap::HeightmapType,
        x: i32,
        z: i32,
    ) -> i32 {
        let max_y = self.min_build_height() + self.height();
        for y in (self.min_build_height()..max_y).rev() {
            if let Some(state) = self.get_block_state(BlockPos::new(x, y, z)) {
                if ty.is_block_counted(&state) {
                    return y + 1;
                }
            }
        }
        self.min_build_height()
    }

    /// `getNearestPlayer(targeting, mob, x, y, z, aabb)` —
    /// ближайший игрок (для LookAtPlayerGoal и др.).
    /// Default: нет сущностей (ServerLevel переопределит).
    fn nearest_player(
        &self,
        _pos: crate::world::phys::vec3::Vec3,
        _range: f64,
    ) -> Option<crate::world::entity::entity::Entity> {
        None
    }

    /// `getNearestEntity(targeting, entity, x, y, z)` —
    /// ближайшая сущность заданного типа (LookAtPlayerGoal
    /// non-Player ветка). Default: нет сущностей.
    fn nearest_entity(
        &self,
        _pos: crate::world::phys::vec3::Vec3,
        _range: f64,
    ) -> Option<crate::world::entity::entity::Entity> {
        None
    }

    /// `getNearestEntity` с фильтром по entity-type id —
    /// `NearestAttackableTargetGoal.findTarget` non-player ветка.
    /// Default: фильтрует `nearest_entity` (реализации с полным
    /// spatial query должны переопределять).
    fn nearest_entity_of_type(
        &self,
        pos: crate::world::phys::vec3::Vec3,
        range: f64,
        type_id: &Identifier,
    ) -> Option<crate::world::entity::entity::Entity> {
        self.nearest_entity(pos, range).filter(|e| {
            crate::world::entity::entity_type::holder_type_id(&e.entity_type)
                .map(|id| &id == type_id)
                .unwrap_or(false)
        })
    }

    /// `getEntity(uuid)` / `getEntityInAnyDimension` — резолв
    /// сущности по UUID (EntityReference). Default: нет сущностей.
    fn entity_by_uuid(
        &self,
        _uuid: crate::util::uuid::Uuid,
    ) -> Option<crate::world::entity::entity::Entity> {
        None
    }

    /// `getDifficulty()` — сложность уровня (TargetingConditions,
    /// LivingEntity::canAttack). Default: Normal.
    fn difficulty(&self) -> crate::world::difficulty::Difficulty {
        crate::world::difficulty::Difficulty::Normal
    }

    /// `getGameRules().get(GameRules.UNIVERSAL_ANGER)` —
    /// universal anger для HurtByTargetGoal. Default: false.
    fn universal_anger(&self) -> bool {
        false
    }

    /// `getEntitiesOfClass(Mob, aabb, NO_SPECTATORS)` —
    /// итерация по мобам в AABB для `HurtByTargetGoal.alertOthers`.
    /// Default: нет мобов (ServerLevel переопределит).
    fn for_each_mob_in_box(
        &self,
        _aabb: &crate::world::phys::aabb::Aabb,
        _f: &mut dyn FnMut(&mut crate::world::entity::mob::Mob),
    ) {
    }

    /// `getEntitiesOfClass(Entity, aabb)` / `getNearbyEntities` —
    /// снимок сущностей в AABB (DefendVillageTargetGoal и др.).
    /// Default: пусто.
    fn entities_in_box(
        &self,
        _aabb: &crate::world::phys::aabb::Aabb,
    ) -> Vec<crate::world::entity::entity::Entity> {
        Vec::new()
    }

    // `CollisionGetter.getWorldBorder`-стиль проверек не нужен —
    // `isOutsideBuildHeight` уже покрыт `min_build_height`/`height`.
}


