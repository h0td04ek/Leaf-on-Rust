//! Перенос `net.minecraft.world.entity.EntityType` —
//! реестровая запись типа сущности.
//!
//! `EntityType.STREAM_CODEC = ByteBufCodecs.holderRegistry(ENTITY_TYPE)`.
//! Полная модель (фабрика сущностей, размеры,
//! serialization codec) — с портом системы сущностей.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::world::entity::mob_category::MobCategory;
use std::sync::{Arc, RwLock};

/// Глобальный реестр типов сущностей (`BuiltInRegistries.ENTITY_TYPE`).
static ENTITY_TYPE_REGISTRY: std::sync::OnceLock<RwLock<Vec<Arc<EntityType>>>> = std::sync::OnceLock::new();

fn entity_type_registry() -> &'static RwLock<Vec<Arc<EntityType>>> {
    ENTITY_TYPE_REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

/// Зарегистрировать тип сущности в реестре (порядок = registry id).
pub fn register_entity_type(entity_type: EntityType) -> i32 {
    let mut reg = entity_type_registry().write().unwrap();
    reg.push(Arc::new(entity_type));
    (reg.len() - 1) as i32
}

/// `BuiltInRegistries.ENTITY_TYPE.byNameLookup`-подобный поиск.
pub fn entity_type_by_identifier(id: &Identifier) -> Option<Arc<EntityType>> {
    entity_type_registry().read().unwrap().iter().find(|e| &e.id == id).cloned()
}

/// `EntityType.PIG` — поиск по ключу `minecraft:pig`.
pub fn pig() -> Option<Arc<EntityType>> {
    entity_type_by_identifier(&Identifier::with_default_namespace("pig").unwrap())
}

/// `EntityType` — запись реестра `Registries.ENTITY_TYPE`.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityType {
    /// Registry-ключ типа сущности (`minecraft:zombie` и т.п.).
    pub id: Identifier,
    /// `category` — категория спавна.
    pub category: MobCategory,
}

impl EntityType {
    /// `new`.
    pub fn new(id: Identifier) -> Self {
        Self { id, category: MobCategory::Misc }
    }

    /// `withCategory`-стиль: задать категорию.
    pub fn with_category(mut self, category: MobCategory) -> Self {
        self.category = category;
        self
    }

    /// `getCategory`.
    pub fn get_category(&self) -> MobCategory {
        self.category
    }

    /// `getDescriptionId`-базовый id (`entity.minecraft.zombie`).
    pub fn id_string(&self) -> String {
        format!("entity.{}", self.id)
    }

    /// `EntityType.getKey(type)` — registry-ключ.
    pub fn get_key(&self) -> Identifier {
        self.id.clone()
    }

    /// `EntityType.STREAM_CODEC` — `holderRegistry(ENTITY_TYPE)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<EntityType>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<EntityType>()
    }
}

impl Eq for EntityType {}
impl std::hash::Hash for EntityType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
