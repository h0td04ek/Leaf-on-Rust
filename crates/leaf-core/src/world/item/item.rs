//! Перенос `net.minecraft.world.item.Item` — запись реестра ITEM.
//!
//! `Item.STREAM_CODEC` = `ByteBufCodecs.holderRegistry(Registries.ITEM)`:
//! на проводе — plain VarInt registry-id (0-based, БЕЗ direct-ветки
//! и БЕЗ +1 сдвига — в отличие от `ByteBufCodecs.holder`).
//!
//! Поля Item (descriptionId, craftRemainder, maxStackSize, ...) будут
//! расширяться по мере порта `Item`-иерархии; wire-критична только
//! registry-идентичность.

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

    /// `Item` для Direct-holder (не из реестра).
    pub fn direct(id: Identifier) -> Arc<Self> {
        Arc::new(Self::new(id))
    }
}

/// Глобальный реестр `Registries.ITEM` — порядок = vanilla bootstrap
/// (wire-id = индекс). Наполняется `items::bootstrap` по мере порта.
static ITEM_REGISTRY: std::sync::OnceLock<RwLock<Vec<Arc<Item>>>> =
    std::sync::OnceLock::new();

fn registry() -> &'static RwLock<Vec<Arc<Item>>> {
    ITEM_REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

/// Регистрация предмета — вызывается из bootstrap предметов.
/// Возвращает registry-id.
pub fn register_item(item: Item) -> i32 {
    let mut reg = registry().write().unwrap();
    let id = reg.len() as i32;
    reg.push(Arc::new(item));
    id
}

/// `itemById`.
pub fn item_by_id(id: i32) -> Option<Arc<Item>> {
    registry().read().unwrap().get(id as usize).cloned()
}

/// Поиск по Identifier.
pub fn item_by_identifier(id: &Identifier) -> Option<Arc<Item>> {
    registry().read().unwrap().iter().find(|i| i.id == *id).cloned()
}

/// Id предмета в реестре (None — не зарегистрирован).
pub fn item_registry_id(item: &Item) -> Option<i32> {
    registry().read().unwrap().iter().position(|i| i.id == item.id).map(|i| i as i32)
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
