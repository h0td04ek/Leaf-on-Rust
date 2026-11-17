//! Перенос `net.minecraft.core.component.DataComponentType`.
//!
//! Типизированный кодек компонента хранится как type-erased
//! `ComponentStreamCodec` (encode/decode через `dyn Any`), реестр —
//! глобальный упорядоченный список (wire-id = индекс, как в
//! `BuiltInRegistries.DATA_COMPONENT_TYPE`).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// Type-erased значение компонента — `PartialEq`/`Debug` через
/// `value_eq`/`value_debug` (Java `Object.equals`/`toString`).
pub trait ComponentValueData: Send + Sync {
    /// `as_any` — downcast к конкретному типу.
    fn as_any(&self) -> &dyn Any;
    /// `type_id`.
    fn type_id(&self) -> TypeId;
    /// `equals` — сравнение с другим type-erased значением.
    fn value_eq(&self, other: &dyn ComponentValueData) -> bool;
    /// `toString`-эквивалент.
    fn value_debug(&self) -> String;
    /// Клонирование type-erased значения.
    fn clone_box(&self) -> ComponentValue;
}

impl<T> ComponentValueData for T
where
    T: Any + Send + Sync + PartialEq + Debug + Clone + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn value_eq(&self, other: &dyn ComponentValueData) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .map(|o| self == o)
            .unwrap_or(false)
    }
    fn value_debug(&self) -> String {
        format!("{:?}", self)
    }
    fn clone_box(&self) -> ComponentValue {
        Box::new(self.clone())
    }
}

/// Type-erased stream-кодек значения компонента.
#[derive(Clone)]
pub struct ComponentStreamCodec {
    encode: Arc<dyn Fn(&mut FriendlyByteBuf, &dyn Any) + Send + Sync>,
    decode: Arc<dyn Fn(&mut FriendlyByteBuf) -> ComponentValue + Send + Sync>,
    /// `TypeId` значения для downcast при чтении.
    pub value_type_id: TypeId,
}

impl ComponentStreamCodec {
    /// `networkSynchronized(streamCodec)` — типизированная обёртка.
    pub fn new<T, E, D>(encode: E, decode: D, value_type_id: TypeId) -> Self
    where
        T: Any + Send + Sync + PartialEq + Debug + Clone + 'static,
        E: Fn(&mut FriendlyByteBuf, &T) + Send + Sync + 'static,
        D: Fn(&mut FriendlyByteBuf) -> T + Send + Sync + 'static,
    {
        Self {
            encode: Arc::new(move |buf, value| {
                let typed = value.downcast_ref::<T>().expect("component value type mismatch");
                encode(buf, typed);
            }),
            decode: Arc::new(move |buf| Box::new(decode(buf)) as ComponentValue),
            value_type_id,
        }
    }

    /// `encode` — сериализация значения.
    pub fn encode(&self, buf: &mut FriendlyByteBuf, value: &dyn Any) {
        (self.encode)(buf, value);
    }

    /// `decode` — десериализация значения.
    pub fn decode(&self, buf: &mut FriendlyByteBuf) -> ComponentValue {
        (self.decode)(buf)
    }
}

/// `DataComponentType<T>` — запись реестра `DATA_COMPONENT_TYPE`.
#[derive(Clone)]
pub struct DataComponentType {
    /// Registry-ключ (`minecraft:max_stack_size` и т.п.).
    pub id: Identifier,
    /// `codec` — persistent codec (NBT); `None` = transient.
    /// Хранится как маркер: NBT-кодеки портируются с `nbt`-интеграцией.
    pub transient: bool,
    /// `ignoreSwapAnimation`.
    pub ignore_swap_animation: bool,
    /// `streamCodec` — network-кодек значения (`@Nullable`:
    /// transient-only типы не имеют network-кодека).
    pub stream_codec: Option<ComponentStreamCodec>,
}

impl DataComponentType {
    /// `isTransient`.
    pub fn is_transient(&self) -> bool {
        self.transient
    }

    /// `ignoreSwapAnimation`.
    pub fn ignore_swap_animation(&self) -> bool {
        self.ignore_swap_animation
    }
}

impl std::fmt::Debug for DataComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataComponentType")
            .field("id", &self.id)
            .field("transient", &self.transient)
            .finish()
    }
}

/// Глобальный реестр типов компонентов — порядок = vanilla
/// `DataComponents.bootstrap` (wire-id = индекс).
static DATA_COMPONENT_TYPE_REGISTRY: std::sync::OnceLock<RwLock<Vec<Arc<DataComponentType>>>> =
    std::sync::OnceLock::new();

fn registry() -> &'static RwLock<Vec<Arc<DataComponentType>>> {
    DATA_COMPONENT_TYPE_REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

/// `DataComponentType.STREAM_CODEC` —
/// `ByteBufCodecs.registry(DATA_COMPONENT_TYPE)` (VarInt id+1).
pub fn data_component_type_stream_codec(
) -> impl StreamCodec<FriendlyByteBuf, Arc<DataComponentType>> {
    crate::network::codec::stream_codec::sync_fn(
        |buf: &mut FriendlyByteBuf, t: &Arc<DataComponentType>| {
            let id = data_component_type_id(t).expect("unregistered component type");
            buf.write_var_int(id + 1);
        },
        |buf: &mut FriendlyByteBuf| {
            let i = buf.read_var_int();
            assert!(i != 0, "invalid data component type id");
            data_component_type_arc_by_id(i - 1).expect("unknown component type id")
        },
    )
}

/// Wire-id типа (индекс в реестре).
pub fn data_component_type_id(ty: &DataComponentType) -> Option<i32> {
    crate::core::component::data_components::ensure_bootstrapped();
    let reg = registry().read().unwrap();
    reg.iter().position(|t| t.id == ty.id).map(|i| i as i32)
}

/// `dataComponentTypeById`.
pub fn data_component_type_by_id(id: i32) -> Option<DataComponentType> {
    crate::core::component::data_components::ensure_bootstrapped();
    registry()
        .read()
        .unwrap()
        .get(id as usize)
        .map(|t| (**t).clone())
}

/// `dataComponentTypeByIdentifier` — `BuiltInRegistries.DATA_COMPONENT_TYPE.getValue`.
pub fn data_component_type_by_identifier(id: &Identifier) -> Option<DataComponentType> {
    crate::core::component::data_components::ensure_bootstrapped();
    registry()
        .read()
        .unwrap()
        .iter()
        .find(|t| t.id == *id)
        .map(|t| (**t).clone())
}

/// Регистрация типа — вызывается из `data_components::bootstrap` в
/// точном порядке vanilla. Возвращает wire-id.
pub fn register_data_component_type(ty: DataComponentType) -> i32 {
    let mut reg = registry().write().unwrap();
    let id = reg.len() as i32;
    reg.push(Arc::new(ty));
    id
}

/// Снимок реестра (для отладки/тестов).
pub fn data_component_type_count() -> usize {
    crate::core::component::data_components::ensure_bootstrapped();
    registry().read().unwrap().len()
}

/// Все зарегистрированные типы по id.
pub fn all_data_component_types() -> Vec<DataComponentType> {
    crate::core::component::data_components::ensure_bootstrapped();
    registry().read().unwrap().iter().map(|t| (**t).clone()).collect()
}

/// Индекс типов по Identifier — для patch codec.
pub fn data_component_type_id_by_identifier(id: &Identifier) -> Option<i32> {
    crate::core::component::data_components::ensure_bootstrapped();
    let reg = registry().read().unwrap();
    reg.iter().position(|t| t.id == *id).map(|i| i as i32)
}

/// `DataComponentType` по wire-id (Arc).
pub fn data_component_type_arc_by_id(id: i32) -> Option<Arc<DataComponentType>> {
    crate::core::component::data_components::ensure_bootstrapped();
    registry().read().unwrap().get(id as usize).cloned()
}

/// Type-erased значение компонента.
pub type ComponentValue = Box<dyn ComponentValueData>;

impl Clone for ComponentValue {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Удобный билдер `ComponentStreamCodec` из StreamCodec-подобной пары.
pub fn typed_stream_codec<T, E, D>(encode: E, decode: D) -> ComponentStreamCodec
where
    T: Any + Send + Sync + PartialEq + Debug + Clone + 'static,
    E: Fn(&mut FriendlyByteBuf, &T) + Send + Sync + 'static,
    D: Fn(&mut FriendlyByteBuf) -> T + Send + Sync + 'static,
{
    ComponentStreamCodec::new(encode, decode, TypeId::of::<T>())
}

/// Кодек на основе `StreamCodec` объекта.
pub fn stream_codec_of<T, C>(codec: C) -> ComponentStreamCodec
where
    T: Any + Send + Sync + PartialEq + Debug + Clone + 'static,
    C: StreamCodec<FriendlyByteBuf, T> + Send + Sync + 'static,
{
    let codec = std::sync::Arc::new(codec);
    typed_stream_codec::<T, _, _>(
        {
            let codec = codec.clone();
            move |buf, v| codec.encode(buf, v)
        },
        move |buf| codec.decode(buf),
    )
}

/// Downcast helper для значений.
pub fn component_value_downcast<T: 'static>(value: &dyn Any) -> Option<&T> {
    value.downcast_ref::<T>()
}
