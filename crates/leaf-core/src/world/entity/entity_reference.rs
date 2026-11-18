//! Перенос `net.minecraft.world.entity.EntityReference` —
//! ссылка на сущность по UUID с кэшем (Either<UUID, entity>).

use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::util::uuid::Uuid;
use crate::world::entity::entity::Entity;
use crate::world::level::level_reader::LevelReader;
use std::sync::Arc;

/// `EntityReference<StoredEntityType>` — Either<UUID, entity>:
/// кэширует сущность, пока она не удалена; после удаления
/// резолвит по UUID через `LevelReader::entity_by_uuid`.
#[derive(Clone)]
pub struct EntityReference {
    /// `uuid` — UUID сущности (вычисляется из кэша или задан).
    uuid: Uuid,
    /// `entity` — кэш `Either.right(entity)`; `None` = `Either.left(uuid)`.
    entity: Option<Entity>,
}

impl EntityReference {
    /// `new EntityReference(entity)` — `Either.right`.
    pub fn of(entity: &Entity) -> Self {
        Self {
            uuid: entity.uuid,
            entity: Some(entity.clone()),
        }
    }

    /// `new EntityReference(uuid)` — `Either.left`.
    pub fn of_uuid(uuid: Uuid) -> Self {
        Self { uuid, entity: None }
    }

    /// `getUUID()`.
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    /// `getEntity(uuidLookup, entityClass)` / `getEntity(level, class)` —
    /// кэш, если сущность не удалена; иначе резолв по UUID через уровень.
    /// `entity_class`-фильтр в Rust не нужен: уровень возвращает
    /// уже нужный тип (Entity — базовый для всех ссылок).
    pub fn get(&self, level: Option<&Arc<dyn LevelReader>>) -> Option<Entity> {
        if let Some(e) = &self.entity {
            if !e.removed {
                return Some(e.clone());
            }
        }
        // Кэш мёртв/отсутствует — резолв по UUID (Either.left ветка).
        let resolved = level.and_then(|l| l.entity_by_uuid(self.uuid));
        if let Some(e) = &resolved {
            if !e.removed {
                return resolved;
            }
        }
        None
    }

    /// `matches(entity)` — совпадение по UUID.
    pub fn matches(&self, entity: &Entity) -> bool {
        self.uuid == entity.uuid
    }

    /// `store(output, key)` — сохраняет UUID строкой (как
    /// `UUIDUtil.CODEC` в NBT).
    pub fn store(&self, tag: &mut CompoundTag, key: &str) {
        tag.put(key, Tag::String(self.uuid.to_string()));
    }

    /// `read(input, key)` — читает UUID-строку.
    pub fn read(tag: &CompoundTag, key: &str) -> Option<Self> {
        tag.get_string(key)
            .and_then(Uuid::from_string)
            .map(Self::of_uuid)
    }

    /// `EntityReference.getLivingEntity(reference, level)` —
    /// резолв + обёртка в LivingEntity (см. `LivingEntity::from_entity`).
    pub fn get_living_entity(
        reference: Option<&Self>,
        level: Option<&Arc<dyn LevelReader>>,
    ) -> Option<crate::world::entity::living_entity::LivingEntity> {
        reference?
            .get(level)
            .map(crate::world::entity::living_entity::LivingEntity::from_entity)
    }
}
