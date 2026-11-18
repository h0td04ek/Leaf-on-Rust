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
pub trait LevelReader {
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

    /// `setBlock(pos, blockId)` — установка блока по registry-id
    /// (cobweb spawn WeavingMobEffect). Default no-op.
    fn set_block_by_id(&self, _pos: BlockPos, _block_id: &Identifier) -> bool {
        false
    }

    /// `levelEvent(eventId, pos, data)` — level event
    /// (ANIMATION_SPAWN_COBWEB=3018 и т.п.). Default no-op.
    fn level_event(&self, _event_id: i32, _pos: BlockPos, _data: i32) {}

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
}
