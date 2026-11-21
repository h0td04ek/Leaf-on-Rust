//! Перенос `net.minecraft.world.item.Item` — запись реестра ITEM.
//!
//! `Item.STREAM_CODEC` = `ByteBufCodecs.holderRegistry(Registries.ITEM)`:
//! на проводе — plain VarInt registry-id (0-based, БЕЗ direct-ветки
//! и БЕЗ +1 сдвига — в отличие от `ByteBufCodecs.holder`).
//!
//! Поля Item (descriptionId, craftRemainder, maxStackSize, ...) будут
//! расширяться по мере порта `Item`-иерархии; wire-критична только
//! registry-идентичность.

use crate::core::component::DataComponentMap;
use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use std::sync::{Arc, RwLock};

/// `Item` — базовый класс предмета (запись реестра).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Item {
    /// Registry-ключ (`minecraft:stone` и т.п.).
    pub id: Identifier,
}

impl Item {
    /// `new Item(id)`-эквивалент.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    /// `Item.getAttackDamageBonus(target, damageAmount, source)` —
    /// бонус урона предмета (база — 0; переопределяется предметами
    /// типа trident/spear).
    pub fn get_attack_damage_bonus(
        &self,
        _target: &crate::world::entity::entity::Entity,
        _damage_amount: f32,
        _damage_source: &crate::world::damagesource::damage_source::DamageSource,
    ) -> f32 {
        0.0
    }

    /// `Item.getItemDamageSource(entity)` (deprecated) —
    /// предмет-специфичный источник урона. Default: None.
    pub fn get_item_damage_source(
        &self,
        _entity: &crate::world::entity::living_entity::LivingEntity,
    ) -> Option<crate::world::damagesource::damage_source::DamageSource> {
        None
    }

    /// `Item.hurtEnemy(stack, target, attacker)` — пост-ударный
    /// хук предмета (износ/эффекты). Default: no-op.
    pub fn hurt_enemy(
        &self,
        _stack: &crate::world::item::item_stack::ItemStack,
        _target: &crate::world::entity::living_entity::LivingEntity,
        _attacker: &crate::world::entity::living_entity::LivingEntity,
    ) {
    }

    /// `Item.postHurtEnemy` — хук после урона. Default: no-op.
    pub fn post_hurt_enemy(
        &self,
        _stack: &crate::world::item::item_stack::ItemStack,
        _target: &crate::world::entity::living_entity::LivingEntity,
        _attacker: &crate::world::entity::living_entity::LivingEntity,
    ) {
    }

    /// `Item` для Direct-holder (не из реестра).
    pub fn direct(id: Identifier) -> Arc<Self> {
        Arc::new(Self::new(id))
    }
}

struct RegistryEntry {
    item: Arc<Item>,
    components: Option<Arc<DataComponentMap>>,
}

/// Глобальный реестр `Registries.ITEM` — порядок индексов совпадает с wire-id.
static ITEM_REGISTRY: std::sync::OnceLock<RwLock<Vec<RegistryEntry>>> =
    std::sync::OnceLock::new();

fn registry() -> &'static RwLock<Vec<RegistryEntry>> {
    ITEM_REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

/// Регистрация записи без загруженного прототипа компонентов.
pub fn register_item(item: Item) -> i32 {
    register(item, None)
}

/// Регистрация записи с компонентами, которые наследуют её стопки.
pub fn register_item_with_components(item: Item, components: DataComponentMap) -> i32 {
    register(item, Some(Arc::new(components)))
}

fn register(item: Item, components: Option<Arc<DataComponentMap>>) -> i32 {
    let mut reg = registry().write().unwrap();
    let id = reg.len() as i32;
    reg.push(RegistryEntry { item: Arc::new(item), components });
    id
}

/// `itemById`.
pub fn item_by_id(id: i32) -> Option<Arc<Item>> {
    registry().read().unwrap().get(id as usize).map(|entry| Arc::clone(&entry.item))
}

pub(crate) fn item_identifiers() -> Vec<Identifier> {
    registry().read().unwrap().iter().map(|entry| entry.item.id.clone()).collect()
}

fn find_entry<'a>(entries: &'a [RegistryEntry], id: &Identifier) -> Option<&'a RegistryEntry> {
    entries.iter().find(|entry| entry.item.id == *id)
}

/// Поиск по Identifier.
pub fn item_by_identifier(id: &Identifier) -> Option<Arc<Item>> {
    find_entry(&registry().read().unwrap(), id).map(|entry| Arc::clone(&entry.item))
}

pub fn item_holder_by_identifier(id: &Identifier) -> Option<Holder<Item>> {
    registry().read().unwrap().iter().position(|entry| entry.item.id == *id)
        .map(|wire_id| Holder::Reference(wire_id as i32))
}

pub(crate) fn component_prototype(holder: &Holder<Item>) -> Option<Arc<DataComponentMap>> {
    let registry = registry().read().unwrap();
    let entry = match holder {
        Holder::Reference(id) => registry.get(*id as usize),
        Holder::Direct(item) => find_entry(&registry, &item.id),
        Holder::Key(key) => find_entry(&registry, &key.identifier),
    }?;
    entry.components.clone()
}

/// Id предмета в реестре (None — не зарегистрирован).
pub fn item_registry_id(item: &Item) -> Option<i32> {
    registry().read().unwrap().iter().position(|entry| entry.item.id == item.id).map(|i| i as i32)
}

/// `Item.STREAM_CODEC` — `holderRegistry(ITEM)`:
/// VarInt id (0-based) ↔ `Holder::Reference(id)`.
pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<Item>> + Send + Sync + 'static {
    sync_fn(
        move |buf: &mut FriendlyByteBuf, holder: &Holder<Item>| {
            let id = match holder {
                Holder::Reference(id) => *id,
                Holder::Key(_) => {
                    panic!("holderRegistry codec cannot encode unbound key")
                }
                Holder::Direct(item) => item_registry_id(item)
                    .expect("direct item is not in registry"),
            };
            buf.write_var_int(id);
        },
        move |buf: &mut FriendlyByteBuf| {
            let id = buf.read_var_int();
            Holder::Reference(id)
        },
    )
}
