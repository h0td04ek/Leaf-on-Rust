//! `net.minecraft.world.level.ServerLevelAccessor` —
//! `LevelAccessor` + доступ к `ServerLevel`.
//!
//! Отличие от Java: `getLevel()` возвращает `&dyn LevelAccessor`
//! (ServerLevel ещё не портирован — ближайший общий уровень);
//! `getMinecraftWorld()` — Paper/CB-алиас.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::world::difficulty_instance::DifficultyInstance;
use crate::world::entity::entity::Entity;
use crate::world::level::level_accessor::LevelAccessor;
use crate::world::level::level_writer::LevelWriter;

/// `ServerLevelAccessor extends LevelAccessor`.
pub trait ServerLevelAccessor: LevelAccessor {
    /// `getLevel()` — `ServerLevel` (у нас `&dyn LevelAccessor`).
    fn get_level(&self) -> &dyn LevelAccessor;

    /// `getCurrentDifficultyAt(pos)` — `DifficultyInstance`.
    /// Default: уровеньная сложность с нулевым временем
    /// (переопределяется в `ServerLevel`).
    fn get_current_difficulty_instance_at(&self, pos: BlockPos) -> DifficultyInstance {
        DifficultyInstance::new(self.current_difficulty_at(pos), 0, 0, 0.0)
    }

    /// `addFreshEntityWithPassengers(entity)`.
    /// Java итерирует `getSelfAndPassengers`; у нас пассажиры
    /// сущностей ещё не хранят Arc — добавляем корневую сущность
    /// (пассажиры пусты до порта vehicle-системы).
    fn add_fresh_entity_with_passengers(&self, entity: Arc<Entity>) {
        self.add_fresh_entity_with_passengers_reason(entity);
    }

    /// `addFreshEntityWithPassengers(entity, reason)` — CB-вариант.
    fn add_fresh_entity_with_passengers_reason(&self, entity: Arc<Entity>) {
        LevelWriter::add_fresh_entity(self, entity);
    }

    /// CraftBukkit `getMinecraftWorld()` — алиас `getLevel()`.
    fn get_minecraft_world(&self) -> &dyn LevelAccessor {
        self.get_level()
    }
}
