//! `net.minecraft.world.level.LevelAccessor` —
//! `CommonLevelAccessor + LevelReader + ScheduledTickAccess` +
//! звук/частицы/level-event/game-event хуки.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::holder::Holder;
use crate::core::particles::particle_options::ParticleOptions;
use crate::random::RandomSource;
use crate::sounds::sound_event::SoundEvent;
use crate::sounds::sound_source::SoundSource;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_source::ChunkSource;
use crate::world::level::common_level_accessor::CommonLevelAccessor;
use crate::world::level::gameevent::game_event::{Context as GameEventContext, GameEvent};
use crate::world::level::redstone::neighbor_updater;
use crate::world::level::scheduled_tick_access::{
    BlockTickType, FluidTickType, ScheduledTickAccess,
};
use crate::world::level::storage::level_data::LevelData;
use crate::world::level::RespawnData;
use crate::world::phys::vec3::Vec3;
use crate::world::ticks::{ScheduledTick, TickPriority};

/// `LevelAccessor`.
pub trait LevelAccessor: CommonLevelAccessor + ScheduledTickAccess {
    /// `nextSubTickCount()`.
    fn next_sub_tick_count(&self) -> u64;

    /// `getLevelData()`.
    fn get_level_data(&self) -> &dyn LevelData;

    /// `getServer()` — `MinecraftServer` ещё не портирован;
    /// возвращает type-erased ссылку на серверный объект.
    fn get_server(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        None
    }

    /// `getDifficulty()` — default → `levelData.getDifficulty()`.
    fn get_difficulty(&self) -> crate::world::difficulty::Difficulty {
        self.get_level_data().get_difficulty()
    }

    /// `getChunkSource()`.
    fn get_chunk_source(&self) -> &dyn ChunkSource;

    /// `hasChunk(chunkX, chunkZ)` — default → `chunkSource.hasChunk`.
    fn has_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.get_chunk_source().has_chunk(chunk_x, chunk_z)
    }

    /// `getRandom()`.
    fn get_random(&self) -> Arc<dyn RandomSource>;

    /// `self` как `&dyn LevelAccessor` — нужен default-методам:
    /// `Self: ?Sized` в трейтах не коерсится в trait-object
    /// автоматически. Реализация: `fn as_level_accessor(&self)
    /// -> &dyn LevelAccessor { self }`.
    fn as_level_accessor(&self) -> &dyn LevelAccessor;

    /// `getRespawnData()` — Paper/Leaf: точка возрождения.
    fn get_respawn_data(&self) -> RespawnData {
        self.get_level_data().get_respawn_data()
    }

    /// `updateNeighborsAt(pos, block)` — default пусто (Java).
    /// Переопределяется в `Level` для редстоун-обновлений.
    fn update_neighbors_at(&self, _pos: BlockPos, _block: &BlockState) {}

    /// `neighborShapeChanged(direction, pos, neighborPos,
    /// neighborState, flags, recursionLeft)` — default →
    /// `NeighborUpdater.executeShapeUpdate`.
    #[allow(clippy::too_many_arguments)]
    fn neighbor_shape_changed(
        &self,
        direction: Direction,
        pos: BlockPos,
        neighbor_pos: BlockPos,
        neighbor_state: &BlockState,
        flags: i32,
        recursion_left: i32,
    ) {
        neighbor_updater::execute_shape_update(
            self.as_level_accessor(),
            direction,
            pos,
            neighbor_pos,
            neighbor_state,
            flags,
            recursion_left - 1,
        );
    }

    /// `playSound(entity, pos, sound, source)` — default →
    /// 1.0F/1.0F.
    #[allow(clippy::too_many_arguments)]
    fn play_sound_entity(
        &self,
        _entity: Option<Arc<crate::world::entity::entity::Entity>>,
        _pos: BlockPos,
        _sound: &SoundEvent,
        _source: SoundSource,
    ) {
        // default: playSound(entity, pos, sound, source, 1, 1)
        // — вызов 6-аргументной версии невозможен без типа Holder;
        // конкретные уровни переопределяют.
    }

    /// `playSound(entity, pos, sound, source, volume, pitch)`.
    fn play_sound_full(
        &self,
        _entity: Option<Arc<crate::world::entity::entity::Entity>>,
        _pos: BlockPos,
        _sound: &SoundEvent,
        _source: SoundSource,
        _volume: f32,
        _pitch: f32,
    ) {
    }

    /// `addParticle(options, x, y, z, xSpeed, ySpeed, zSpeed)`.
    #[allow(clippy::too_many_arguments)]
    fn add_particle(
        &self,
        _options: &ParticleOptions,
        _x: f64,
        _y: f64,
        _z: f64,
        _x_speed: f64,
        _y_speed: f64,
        _z_speed: f64,
    ) {
    }

    /// `levelEvent(entity, type, pos, data)`.
    fn level_event_entity(
        &self,
        _entity: Option<Arc<crate::world::entity::entity::Entity>>,
        _event_type: i32,
        _pos: BlockPos,
        _data: i32,
    ) {
    }

    /// `levelEvent(type, pos, data)` — default → `levelEvent(null,...)`.
    fn level_event(&self, event_type: i32, pos: BlockPos, data: i32) {
        self.level_event_entity(None, event_type, pos, data);
    }

    /// `gameEvent(gameEvent, pos, context)` — Holder-вариант.
    fn game_event_holder(
        &self,
        _game_event: &Holder<GameEvent>,
        _pos: Vec3,
        _context: GameEventContext,
    ) {
    }

    /// `gameEvent(entity, gameEvent, pos)` — default → Context(entity,null).
    fn game_event_entity(
        &self,
        entity: Option<Arc<crate::world::entity::entity::Entity>>,
        game_event: &Holder<GameEvent>,
        pos: Vec3,
    ) {
        self.game_event_holder(game_event, pos, GameEventContext::of_entity(entity));
    }

    /// `gameEvent(entity, gameEvent, pos: BlockPos)` — Vec3.atCenterOf.
    fn game_event_entity_block_pos(
        &self,
        entity: Option<Arc<crate::world::entity::entity::Entity>>,
        game_event: &Holder<GameEvent>,
        pos: BlockPos,
    ) {
        self.game_event_entity(entity, game_event, crate::world::phys::vec3::at_center_of(&pos.to_vec3i()));
    }

    /// `gameEvent(gameEvent, pos, context)` — BlockPos-вариант.
    fn game_event_holder_block_pos(
        &self,
        game_event: &Holder<GameEvent>,
        pos: BlockPos,
        context: GameEventContext,
    ) {
        self.game_event_holder(game_event, crate::world::phys::vec3::at_center_of(&pos.to_vec3i()), context);
    }

    /// `gameEvent(ResourceKey, pos, context)` — lookup по реестру.
    /// Требует доступа к реестру GAME_EVENT; default no-op до
    /// порта RegistryAccess на уровне.
    fn game_event_key(
        &self,
        _game_event: &crate::resources::ResourceKey<GameEvent>,
        _pos: BlockPos,
        _context: GameEventContext,
    ) {
    }

    // ---- ScheduledTickAccess переопределения Java ----
    // `createTick` использует `getGameTime() + delay` и
    // `nextSubTickCount()`. Наш ScheduledTickAccess уже имеет
    // `create_tick_*` — здесь добавляем Java-точные
    // `createTick(pos, type, delay[, priority])`.

    /// `createTick(pos, block, delay, priority)`.
    fn create_tick_block_at(
        &self,
        pos: BlockPos,
        type_: BlockTickType,
        delay: i32,
        priority: TickPriority,
    ) -> ScheduledTick<BlockTickType> {
        ScheduledTick::new(
            type_,
            pos,
            self.game_time() + delay as i64,
            priority,
            self.next_sub_tick_count() as i64,
        )
    }

    /// `createTick(pos, block, delay)`.
    fn create_tick_block_at_default(
        &self,
        pos: BlockPos,
        type_: BlockTickType,
        delay: i32,
    ) -> ScheduledTick<BlockTickType> {
        ScheduledTick::new(
            type_,
            pos,
            self.game_time() + delay as i64,
            TickPriority::Normal,
            self.next_sub_tick_count() as i64,
        )
    }

    /// `createTick(pos, fluid, delay, priority)`.
    fn create_tick_fluid_at(
        &self,
        pos: BlockPos,
        type_: FluidTickType,
        delay: i32,
        priority: TickPriority,
    ) -> ScheduledTick<FluidTickType> {
        ScheduledTick::new(
            type_,
            pos,
            self.game_time() + delay as i64,
            priority,
            self.next_sub_tick_count() as i64,
        )
    }

    /// `createTick(pos, fluid, delay)`.
    fn create_tick_fluid_at_default(
        &self,
        pos: BlockPos,
        type_: FluidTickType,
        delay: i32,
    ) -> ScheduledTick<FluidTickType> {
        ScheduledTick::new(
            type_,
            pos,
            self.game_time() + delay as i64,
            TickPriority::Normal,
            self.next_sub_tick_count() as i64,
        )
    }
}

