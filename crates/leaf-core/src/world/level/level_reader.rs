//! Перенос `net.minecraft.world.level.LevelReader` — read-only
//! доступ к уровню для предикатов и паттернов.

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::resources::identifier::Identifier;
use crate::world::level::block::entity::block_entity::BlockEntityAccess;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use std::rc::Rc;
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

    /// `BiomeSource.getNoiseBiome(x, y, z, sampler)` — шумовой биом
    /// в quart-координатах (x/y/z — quart pos). Default: `biome_at`
    /// по block-позиции (`QuartPos::to_block`).
    fn noise_biome_at(
        &self,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<crate::core::holder::Holder<crate::world::level::biome::biome::Biome>> {
        let pos = BlockPos::new(
            crate::core::quart_pos::to_block(x),
            crate::core::quart_pos::to_block(y),
            crate::core::quart_pos::to_block(z),
        );
        self.biome_at(pos).cloned()
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

    /// `getDayTime()` — day time (0..24000 цикл) для
    /// schedule/behavior'ов. По умолчанию = game_time.
    fn day_time(&self) -> i64 {
        self.game_time()
    }

    /// `getPoiManager()` — POI-менеджер уровня.
    /// `None` до порта `PoiManager` на ServerLevel.
    fn poi_manager(
        &self,
    ) -> Option<Rc<dyn crate::world::entity::ai::village::poi::poi_manager::PoiManager>> {
        None
    }

    /// `spawnGolemIfNeeded`-уровень: спавн IronGolem рядом с
    /// villager'ом при выполнении условий. Default no-op до
    /// порта IronGolem entity.
    fn spawn_golem_if_needed(
        &self,
        _villager: &crate::world::entity::entity::Entity,
        _game_time: i64,
        _min_villager_amount: i32,
    ) {
    }

    /// `isVillage(pos)` — алиас is_village_at для behavior-классов.
    fn is_village(&self, pos: BlockPos) -> bool {
        self.is_village_at(pos)
    }

    /// `getWorldBorder().isWithinBounds(pos)` — позиция внутри
    /// границы мира (GoToWantedItem). Default: true.
    fn world_border_is_within_bounds(&self, _pos: BlockPos) -> bool {
        true
    }

    /// `level.random` — источник случайности уровня.
    /// Default: новый LegacyRandom (thread_local не нужен —
    /// вызывается редко).
    fn level_random_next_int(&self, bound: i32) -> i32 {
        use std::cell::RefCell;
        thread_local! {
            static RNG: RefCell<Box<dyn crate::random::RandomSource>> =
                RefCell::new(crate::random::legacy::create_unique());
        }
        RNG.with(|r| r.borrow_mut().next_int_bound(bound))
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

    /// Purpur `purpurConfig.villagerClericsFarmWarts` —
    /// разрешить ли клерикам обрабатывать новь в нижнем мире.
    /// Default false (vanilla-поведение).
    fn purpur_villager_clerics_farm_warts(&self) -> bool {
        false
    }

    /// Purpur `purpurConfig.villagerDisplayTradeItem` —
    /// показывать ли предмет трейда в руке жителя.
    /// Default true (vanilla-поведение).
    fn purpur_villager_display_trade_item(&self) -> bool {
        true
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


    /// `addFreshEntity(FireworkRocketEntity)` — спавн
    /// фейерверк-ракеты (CelebrateVillagersSurvivedRaid).
    /// Default no-op.
    fn add_fresh_firework_rocket(
        &self,
        _entity: crate::world::entity::firework_rocket_entity::FireworkRocketEntity,
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

    /// `PoiManager.getRandom(HOME, ..., radius, random)` — случайный
    /// HOME-POI в radius (LocateHidingPlace). Default: ближайший.
    fn random_home_poi_near(&self, center: BlockPos, radius: i32) -> Option<BlockPos> {
        self.home_poi_near(center, radius)
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

    /// `shootProjectile`-эквивалент — спавн снаряда от сущности
    /// (crossbow/bow/trident). Default no-op до порта
    /// projectile-сущностей.
    fn shoot_projectile(
        &self,
        _shooter: &crate::world::entity::entity::Entity,
        _stack: &crate::world::item::item_stack::ItemStack,
        _target: &crate::world::entity::entity::Entity,
        _charge: f32,
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


    /// `level.getBlockEntity(pos)` как `Container` —
    /// контейнеры блок-сущностей (chest/barrel/shulker/...).
    /// Default — нет доступа к block entity (пусто).
    fn container_at(
        &self,
        _pos: BlockPos,
    ) -> Option<std::rc::Rc<std::cell::RefCell<dyn crate::world::simple_container::Container>>> {
        None
    }

    /// Итерация block-entity контейнеров в области — аналог
    /// `chunk.getBlockEntities().values()` фильтра по
    /// `BaseContainerBlockEntity` в `TransportItemsBetweenContainers`.
    /// Default — пусто.
    fn containers_in_area(
        &self,
        _min: BlockPos,
        _max: BlockPos,
    ) -> Vec<(
        BlockPos,
        std::rc::Rc<std::cell::RefCell<dyn crate::world::simple_container::Container>>,
    )> {
        Vec::new()
    }

    /// `level.clip(ClipContext(pos, vec, COLLIDER, NONE, entity))` —
    /// первый блок по лучу (block-hit pos). Default — нет попадания.
    fn clip_block_hit(&self, _from: crate::world::phys::vec3::Vec3, _to: crate::world::phys::vec3::Vec3) -> Option<BlockPos> {
        None
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

    /// `ChunkGenerator.getBaseHeight(x, z, heightmap, level,
    /// randomState)` — высота сгенерированной колонны
    /// (без пост-обработки). Default: `get_height`.
    fn get_base_height(
        &self,
        ty: crate::world::level::levelgen::heightmap::HeightmapType,
        x: i32,
        z: i32,
    ) -> i32 {
        self.get_height(ty, x, z)
    }

    /// `ChunkGenerator.getBaseColumn(x, z, level, randomState)` —
    /// `NoiseColumn` сгенерированного ландшафта. Default: колонна
    /// из `get_block_state`.
    fn get_base_column(
        &self,
        x: i32,
        z: i32,
    ) -> crate::world::level::noise_column::NoiseColumn {
        crate::world::level::noise_column::NoiseColumn::build_from_level(self, x, z)
    }

    /// `BiomeSource.getBiomesWithin(x, y, z, radius, sampler)` —
    /// биомы в радиусе (к genuinely quart-координатах вызывающий
    /// код уже поделил). Default: один биом в центре.
    fn biomes_within(
        &self,
        x: i32,
        y: i32,
        z: i32,
        _radius: i32,
    ) -> Vec<crate::core::holder::Holder<crate::world::level::biome::biome::Biome>> {
        self.biome_at(BlockPos::new(x, y, z))
            .cloned()
            .into_iter()
            .collect()
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

    /// `forEachEntity(LivingEntity.class, aabb, noSpectators, fn)` —
    /// mutable-итерация по живым сущностям (ServerExplosion
    /// hurtEntities). Default: нет (ServerLevel переопределит).
    fn for_each_living_in_box(
        &self,
        _aabb: &crate::world::phys::aabb::Aabb,
        _f: &mut dyn FnMut(&mut crate::world::entity::living_entity::LivingEntity),
    ) {
    }

    /// `level.random.nextFloat()` — float [0,1) для взрывов
    /// (`calculateExplodedPositions` ray-power). Default — через
    /// `level_random_next_int`.
    fn level_random_next_float(&self) -> f32 {
        self.level_random_next_int(i32::MAX) as f32 / i32::MAX as f32
    }

    /// `GameRules.TNT_EXPLODES` — геймерул для TNT-prime при взрыве.
    /// Default: true.
    fn tnt_explodes(&self) -> bool {
        true
    }

    /// `state.getBlock() instanceof TntBlock` — проверка TNT-блока
    /// для prime-ветки `interactWithBlocks`. Default: false.
    fn is_tnt_block_at(&self, _pos: BlockPos) -> bool {
        false
    }

    /// `blockState.onExplosionHit` → дропы предметов при взрыве
    /// (`getDrops`/`popResource`). Default: пусто (ServerLevel
    /// подключит loot tables).
    fn block_explosion_drops(
        &self,
        _pos: BlockPos,
        _state: &crate::world::level::block::state::block_state::BlockState,
        _yield_fraction: f32,
    ) -> Vec<crate::world::item::item_stack::ItemStack> {
        Vec::new()
    }

    /// `CraftEventFactory.callBlockIgniteEvent` (IgnitionByExplosion)
    /// — разрешение поджога блока взрывом. Default: true.
    fn allow_block_ignition(&self, _pos: BlockPos) -> bool {
        true
    }

    /// `BaseFireBlock.getState(level, pos)` — fire block-state
    /// для `createFire`. Default: None.
    fn fire_block_state_at(&self, _pos: BlockPos) -> Option<crate::world::level::block::state::block_state::BlockState> {
        None
    }

    /// PreExplodeEvents (Purpur/CraftBukkit) — отмена взрыва до
    /// `explode()`. Default: false.
    fn explosion_pre_cancelled(
        &self,
        _explosion: &dyn crate::world::level::explosion::Explosion,
    ) -> bool {
        false
    }


    /// `level.random.nextDouble()` — double [0,1) для спавнеров
    /// и клиентской анимации. Default — через `next_int`.
    fn level_random_next_double(&self) -> f64 {
        self.level_random_next_int(i32::MAX) as f64 / i32::MAX as f64
    }

    /// `level.random.nextLong()` — long для LevelRandom-адаптера.
    /// Default — два int'а.
    fn level_random_next_long(&self) -> i64 {
        (((self.level_random_next_int(i32::MAX) as u64) << 32)
            | self.level_random_next_int(i32::MAX) as u64) as i64
    }

    // ---- BaseSpawner hooks ----

    /// `hasNearbyAlivePlayerThatAffectsSpawningForSpawner(x,y,z,range)`
    /// — игрок, влияющий на спавн, в радиусе. Default: false.
    fn spawner_near_player(&self, _center: crate::world::phys::vec3::Vec3, _range: f64) -> bool {
        false
    }

    /// `hasNeighborSignal(pos)` — редстоун-сигнал (Purpur
    /// spawnerDeactivateByRedstone). Default: false.
    fn has_neighbor_signal(&self, _pos: BlockPos) -> bool {
        false
    }

    /// `isSpawnerBlockEnabled()` — спавнер-блок включён
    /// (gamerule doMobSpawning-подобный). Default: true.
    fn spawner_block_enabled(&self) -> bool {
        true
    }

    /// `tickRates.mobSpawner` — Paper tick rate (тиков между
    /// серверными тиками спавнера). Default: 1.
    fn spawner_tick_rate(&self) -> i32 {
        1
    }

    /// `SpawnPlacements.checkSpawnRules(type, level, reason, pos, random)`
    /// — проверка правил спавна (свет/блок/биом). Default: true
    /// (полная проверка — после порта SpawnPlacements).
    fn spawn_rules_valid(
        &self,
        _entity_type: &crate::world::entity::entity_type::EntityType,
        _pos: BlockPos,
    ) -> bool {
        true
    }

    /// `getMaxLocalRawBrightness(pos)` — max(block, sky) raw light.
    /// Default — через `get_brightness`.
    fn get_max_local_raw_brightness(&self, pos: BlockPos) -> i32 {
        self.get_brightness(crate::world::level::light_layer::LightLayer::Block, pos)
            .max(self.get_brightness(crate::world::level::light_layer::LightLayer::Sky, pos))
    }

    /// `EntityTypeTest.forExactClass + NO_SPECTATORS` — подсчёт
    /// сущностей точного типа в AABB (maxNearbyEntities спавнера).
    /// Default: через `entities_in_box` + фильтр по type-id.
    fn count_nearby_entities_in_box(
        &self,
        type_id: &crate::resources::identifier::Identifier,
        aabb: &crate::world::phys::aabb::Aabb,
    ) -> i32 {
        self.entities_in_box(aabb)
            .iter()
            .filter(|e| {
                !e.is_spectator()
                    && crate::world::entity::entity_type::holder_type_id(&e.entity_type)
                        .map(|id| &id == type_id)
                        .unwrap_or(false)
            })
            .count() as i32
    }

    /// `EntityType.loadEntityRecursive(tag, level, SPAWNER, NOP)` —
    /// создание сущности из NBT (detached, до добавления в мир).
    /// Default: None — ServerLevel реализует с entity-реестром.
    fn spawn_entity_from_nbt(
        &self,
        _entity_tag: &crate::nbt::compound::CompoundTag,
        _pos_hint: crate::world::phys::vec3::Vec3,
    ) -> Option<crate::world::entity::entity::Entity> {
        None
    }

    /// `mob.checkSpawnRules(level, SPAWNER)` — mob-правила спавна
    /// (для non-mob сущностей — true). Default: true.
    fn mob_check_spawn_rules(&self, _entity: &crate::world::entity::entity::Entity) -> bool {
        true
    }

    /// `mob.checkSpawnObstruction(level)` — проверка преград
    /// (для non-mob — true). Default: true.
    fn mob_check_spawn_obstruction(&self, _entity: &crate::world::entity::entity::Entity) -> bool {
        true
    }

    /// `mob.finalizeSpawn(level, difficulty, SPAWNER, null)` —
    /// финализация спавна (экипировка/модификаторы). Default no-op.
    fn finalize_spawner_spawn(
        &self,
        _entity: &mut crate::world::entity::entity::Entity,
        _difficulty: crate::world::difficulty::Difficulty,
    ) {
    }

    /// `mob.equip(equipmentTable)` — применение EquipmentTable.
    /// Default no-op.
    fn equip_spawner_entity(
        &self,
        _entity: &mut crate::world::entity::entity::Entity,
        _equipment: &crate::entity::equipment_table::EquipmentTable,
    ) {
    }

    /// `tryAddFreshEntityWithPassengers(entity, SPAWNER)` — добавить
    /// сущность с пассажирами; возвращает false при неудаче.
    /// Default: `add_fresh_entity` + true.
    fn try_add_fresh_entity_with_passengers(
        &self,
        entity: crate::world::entity::entity::Entity,
    ) -> bool {
        self.add_fresh_entity(entity);
        true
    }

    /// `mob.spawnAnim()` после добавления — по uuid (entity уже
    /// в мире). Default no-op.
    fn mob_spawn_anim(&self, _entity_uuid: crate::util::uuid::Uuid) {}

    /// `level.getCurrentDifficultyAt(pos)` — сложность в позиции.
    /// Default: уровеньная `difficulty()`.
    fn current_difficulty_at(&self, _pos: BlockPos) -> crate::world::difficulty::Difficulty {
        self.difficulty()
    }

    /// `addParticle(SMOKE/FLAME)` для clientTick спавнера —
    /// пара частиц в точке. Default: `send_particles`-хуки нет —
    /// no-op.
    fn add_spawner_particles(&self, _x: f64, _y: f64, _z: f64) {}

    // === NaturalSpawner / SpawnPlacements hooks ===

    /// `natureSpawnChunkMap.nearest(level, x, y, z)` — квадрат
    /// расстояния до ближайшего игрока для natural spawn
    /// (Leaf-оптимизация). Default: +∞ (нет игроков рядом —
    /// спавн не ограничен).
    fn nearest_player_distance_sqr(&self, _x: f64, _y: f64, _z: f64) -> f64 {
        f64::INFINITY
    }

    /// `level.canSpawnEntitiesInChunk(chunkPos)` — можно ли
    /// спавнить сущности в чанке (keep-spawn-area правила).
    /// Default: true.
    fn can_spawn_entities_in_chunk(&self, _chunk_pos: crate::world::level::chunk_pos::ChunkPos) -> bool {
        true
    }

    /// `level.getRespawnData()` — `(dimension, pos)` точки
    /// возрождения (RespawnData). Default: None.
    fn respawn_data(&self) -> Option<(Identifier, crate::world::phys::vec3::Vec3)> {
        None
    }

    /// `level.dimensionType().hasCeiling()` — потолок
    /// (nether/end). Default: false (overworld).
    fn dimension_has_ceiling(&self) -> bool {
        false
    }

    /// `CollisionGetter.containsAnyLiquid(aabb)` — есть ли
    /// жидкость в AABB. Default: скан по `get_fluid_state`.
    fn contains_any_liquid(&self, bb: &crate::world::phys::aabb::Aabb) -> bool {
        let (x0, y0, z0) = (bb.min_x.floor() as i32, bb.min_y.floor() as i32, bb.min_z.floor() as i32);
        let (x1, y1, z1) = (bb.max_x.floor() as i32, bb.max_y.floor() as i32, bb.max_z.floor() as i32);
        for x in x0..=x1 {
            for y in y0..=y1 {
                for z in z0..=z1 {
                    if let Some(fs) = self.get_fluid_state(BlockPos::new(x, y, z)) {
                        if !fs.is_empty() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// `level.isUnobstructed(entity)` — нет ли коллизий у
    /// сущности в её bounding box. Default: `no_collision(bb)`.
    fn is_unobstructed_entity(&self, entity: &crate::world::entity::entity::Entity) -> bool {
        self.no_collision(&entity.bounding_box)
    }

    /// `GameRules.SPAWN_MOBS` — геймерул спавна мобов при
    /// генерации чанков. Default: true.
    fn spawn_mobs_game_rule(&self) -> bool {
        true
    }

    /// `level.getRawBrightness(pos, amountToSubtract)` — raw
    /// яркость с вычетом. Default: max(block, sky - subtract).
    fn get_raw_brightness(&self, pos: BlockPos, amount_to_subtract: i32) -> i32 {
        let block = self.get_brightness(crate::world::level::light_layer::LightLayer::Block, pos);
        let sky = (self.get_brightness(crate::world::level::light_layer::LightLayer::Sky, pos) - amount_to_subtract).max(0);
        block.max(sky)
    }

    /// `level.getMaxLocalRawBrightness(pos, radius)` — max raw
    /// яркость в радиусе. Default: как 1-аргументная версия
    /// (полный скан — в ServerLevel).
    fn get_max_local_raw_brightness_radius(&self, pos: BlockPos, _radius: i32) -> i32 {
        self.get_max_local_raw_brightness(pos)
    }

    /// `dimensionType.monsterSpawnBlockLightLimit()` — лимит
    /// block light для спавна монстров (<15 = лимит).
    /// Default: 15 (без лимита).
    fn monster_spawn_block_light_limit(&self) -> i32 {
        15
    }

    /// `dimensionType.monsterSpawnLightTest().sample(random)` —
    /// сэмпл light-теста монстров. Default: `UniformInt(0,7)`
    /// (overworld).
    fn monster_spawn_light_test_sample(&self, random: &mut dyn crate::random::RandomSource) -> i32 {
        random.next_int_bound(8)
    }

    /// `purpurConfig.entityLeftHandedChance` — шанс леворукости
    /// при `Mob.finalizeSpawn`. Default: 0.05.
    fn entity_left_handed_chance(&self) -> f32 {
        0.05
    }

    /// `biome.is(BiomeTags.X)` — принадлежность биома тегу
    /// (например `REDUCED_WATER_AMBIENT_SPAWNS`). Default: false.
    fn biome_has_tag(&self, _pos: BlockPos, _tag: &Identifier) -> bool {
        false
    }

    /// `isInNetherFortressBounds` — monster на nether_bricks
    /// внутри структуры fortress. Default: false.
    fn nether_fortress_at(&self, _pos: BlockPos) -> bool {
        false
    }

    /// `PreCreatureSpawnEvent` — результат события до спавна
    /// (Paper). `None` — нет слушателей/событие не отменено.
    /// Default: None.
    fn pre_spawn_status(
        &self,
        _entity_type: &crate::entity::entity_type::EntityType,
        _pos: BlockPos,
    ) -> Option<crate::world::level::natural_spawner::PreSpawnStatus> {
        None
    }

    /// `updateFailurePlayerMobTypeMap`/`updatePlayerMobTypeMap` —
    /// per-player mob cap трекинг (Paper). Default: no-op.
    fn track_spawned_mob(&self, _entity: &crate::world::entity::entity::Entity) {}

    /// `NaturalSpawner.getMobForSpawn` — создание `Mob` по типу
    /// (entity registry + `EntityType.create`). Default: None.
    fn create_mob_for_spawn(
        &self,
        _entity_type: &crate::entity::entity_type::EntityType,
    ) -> Option<crate::world::entity::mob::Mob> {
        None
    }
    /// `level.scheduleTick(pos, type, delay)` — планирование тика
    /// блока/жидкости. Default: no-op (подключится с Level-tick).
    fn schedule_tick(&self, _pos: BlockPos, _type: &crate::resources::identifier::Identifier, _delay: i32) {}

    /// `level.markPosForPostprocessing(pos)` — отложенная проверка
    /// формы после пост-обработки структур. Default: no-op.
    fn mark_pos_for_postprocessing(&self, _pos: BlockPos) {}

    /// `getStructureTemplateManager()` — менеджер шаблонов
    /// структур (для structure-фреймворка); `None` у уровней
    /// без шаблонов.
    fn structure_template_manager(
        &self,
    ) -> Option<Arc<crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager>>
    {
        None
    }

    /// Реестр `worldgen/template_pool` для JigsawPlacement.
    /// Default: `None` (Jigsaw-структуры без реестра не генерируются).
    fn pool_registry(
        &self,
    ) -> Option<
        Arc<dyn crate::world::level::levelgen::structure::pools::jigsaw_placement::PoolRegistryAccess>,
    > {
        None
    }

    /// `ChunkGenerator`-доступ для pool-элементов
    /// (`get_first_free_height`, sea level). Default: `None`.
    fn pool_generator(
        &self,
    ) -> Option<
        Arc<dyn crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess>,
    > {
        None
    }

    /// `level.placeLootContainer`/`BlockEntity.setLootTable` —
    /// наполнение контейнера лутом при спавне. Default: no-op
    /// (подключится с BlockEntity/Container wiring).
    fn place_loot_container(
        &self,
        _pos: BlockPos,
        _loot_table: &crate::resources::identifier::Identifier,
        _seed: i64,
    ) {
    }
    /// `biome.is(BiomeTags.MINESHAFT_BLOCKING)` — биом блокирует
    /// генерацию шахты (пустыни/плохие земли и т.п.). Default:
    /// false (реализуется с biome-tag wiring).
    fn is_mineshaft_blocking_biome(&self, _pos: crate::core::block_pos::BlockPos) -> bool {
        false
    }

    /// `level.addFreshEntity(MinecartChest)` — спавн шахтной
    /// тележки с сундуком при генерации структуры. `pos` —
    /// центр блока (x+0.5, y+0.5, z+0.5). Default: no-op.
    fn spawn_minecart_chest(
        &self,
        _pos: crate::world::phys::vec3::Vec3,
        _loot_table: &crate::resources::identifier::Identifier,
        _seed: i64,
    ) {
    }

    /// `SpawnerBlockEntity.setEntityId` при генерации —
    /// установка entity-type спавнера в `pos`. Default: no-op.
    fn place_structure_spawner(&self, _pos: BlockPos, _entity_type: &str) {}

    /// `Level.isInSpawnableBounds(pos)` — позиция в пределах
    /// спавнябельной высоты (vanilla: -64 <= y < 320).
    /// Default: жёсткие ванильные границы.
    fn is_in_spawnable_bounds(&self, pos: crate::core::block_pos::BlockPos) -> bool {
        pos.get_y() >= -64 && pos.get_y() < 320
    }

    /// `level.isChunkLoaded`-эквивалент — загружен ли чанк
    /// (для StructureManager.checkStructurePresence). Default:
    /// `has_chunk_at` по центру чанка.
    fn is_chunk_loaded(&self, chunk_pos: crate::world::level::chunk_pos::ChunkPos) -> bool {
        let center = crate::core::block_pos::BlockPos::new(
            chunk_pos.get_min_block_x() + 8,
            self.min_build_height(),
            chunk_pos.get_min_block_z() + 8,
        );
        self.has_chunk_at(center)
    }
    /// `level.getSeed()` — сид мира (для Structure.generate).
    /// Default: 0.
    fn world_seed(&self) -> i64 {
        0
    }
    /// `level.setBlockEntityNbt`/`BlockEntity.loadWithComponents` —
    /// применение NBT к block entity после установки блока
    /// (StructureTemplate.placeInWorld). Default: no-op.
    fn set_block_entity_nbt(
        &self,
        _pos: BlockPos,
        _nbt: &crate::nbt::compound::CompoundTag,
    ) {
    }

    /// `EntityType.create(nbt, level, spawnReason)` — создание
    /// сущности из NBT (StructureTemplate.placeEntities).
    /// Default: None.
    fn create_entity_from_nbt(
        &self,
        _nbt: &crate::nbt::compound::CompoundTag,
    ) -> Option<crate::world::entity::entity::Entity> {
        None
    }
    /// `level.updateShapeAtEdge`/`updateNeighbourShapes` —
    /// обновление формы соседних блоков после размещения
    /// шаблона. Default: no-op.
    fn update_neighbor_shape(&self, _pos: BlockPos) {}

    /// `StructureTemplate.fillFromWorld` — заполнение шаблона
    /// из области мира. Default: пустой шаблон.
    fn fill_template_from_world(
        &self,
        _pos: BlockPos,
        _size: crate::core::vec3i::Vec3i,
        _include_entities: bool,
        _ignored_blocks: &[crate::resources::identifier::Identifier],
    ) -> crate::world::level::levelgen::structure::templatesystem::structure_template::StructureTemplate {
        crate::world::level::levelgen::structure::templatesystem::structure_template::StructureTemplate::new()
    }

}
