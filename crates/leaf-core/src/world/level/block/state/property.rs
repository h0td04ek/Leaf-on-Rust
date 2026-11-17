//! Перенос `net.minecraft.world.level.block.state.properties.Property`
//! и связанных типов (`Property.Value` иерархии).
//!
//! В Java `Property<T extends Comparable<T>>` — абстрактный класс с
//! тремя наследниками (`BooleanProperty`, `IntegerProperty`,
//! `EnumProperty`). Здесь — дженерик `Property<T: PropertyValue>` +
//! type-erased `PropertyTrait` для `StateDefinition`.

use std::fmt;
use std::sync::Arc;

/// Сериализуемое значение свойства — аналог `Comparable<T>` в Java.
///
/// Реализуется для `i32`, `bool`, `String` и для enum'ов состояний
/// через `impl_property_value!`/ручную реализацию (`parse` +
/// `to_state_value`).
pub trait PropertyValue: Clone + PartialEq + Send + Sync + 'static {
    /// `Property.getValue(String)` — парсинг сериализованного имени.
    fn parse(s: &str) -> Option<Self>;

    /// Преобразование в type-erased `StateValue` для хранения.
    fn to_state_value(&self) -> StateValue;
}

/// Type-erased значение свойства — то, что хранится в `StateHolder`.
///
/// Порядок вариантов повторяет порядок `Comparable` внутри одного
/// типа: Int сравнивается численно, Bool — false<true, Str —
/// лексикографически (enum'ы в vanilla сериализуются именами).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateValue {
    /// `IntegerProperty`.
    Int(i32),
    /// `BooleanProperty`.
    Bool(bool),
    /// `EnumProperty`/строковые значения (serialized name).
    Str(String),
}

impl PropertyValue for i32 {
    fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }
    fn to_state_value(&self) -> StateValue {
        StateValue::Int(*self)
    }
}

impl PropertyValue for bool {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }
    fn to_state_value(&self) -> StateValue {
        StateValue::Bool(*self)
    }
}

impl PropertyValue for String {
    fn parse(s: &str) -> Option<Self> {
        Some(s.to_string())
    }
    fn to_state_value(&self) -> StateValue {
        StateValue::Str(self.clone())
    }
}

/// Реализация `PropertyValue` для enum'ов состояний:
/// `parse` через `serialized_name`, сравнение — по имени.
///
/// ```ignore
/// impl_property_value!(DoorHinge, "left" => Left, "right" => Right);
/// ```
#[macro_export]
macro_rules! impl_property_value {
    ($ty:ty, $($name:literal => $variant:ident),+ $(,)?) => {
        impl $crate::world::level::block::state::property::PropertyValue for $ty {
            fn parse(s: &str) -> Option<Self> {
                $(if s == $name { return Some(<$ty>::$variant); })+
                None
            }
            fn to_state_value(&self) -> $crate::world::level::block::state::property::StateValue {
                $crate::world::level::block::state::property::StateValue::Str(self.serialized_name().to_string())
            }
        }
    };
}

/// Type-erased интерфейс свойства — аналог `Property<?>` wildcard.
pub trait PropertyTrait: Send + Sync {
    /// `getName`.
    fn name(&self) -> &str;

    /// `getPossibleValues` — сериализованные имена всех значений.
    fn possible_values(&self) -> Vec<String>;

    /// `getValue(String)` — `Option<StateValue>` (None если имя
    /// неизвестно или не из множества значений).
    fn parse(&self, s: &str) -> Option<StateValue>;

}

impl dyn PropertyTrait {
    /// `getValue(String)` → типизированное значение `T`
    /// (с проверкой вхождения в множество значений свойства).
    pub fn get_typed<T: PropertyValue>(&self, s: &str) -> Option<T> {
        T::parse(s).filter(|v| {
            let sv = v.to_state_value();
            self.parse(&state_value_to_string(&sv)).as_ref() == Some(&sv)
        })
    }
}

/// `Property<T>` — именованное свойство с набором допустимых
/// значений. `values` — все возможные значения по порядку
/// (порядок значим для `IntegerProperty`/RangedMatcher).
#[derive(Clone)]
pub struct Property<T: PropertyValue> {
    name: String,
    values: Vec<T>,
}

impl<T: PropertyValue> Property<T> {
    /// `Property.create(name, values)`.
    pub fn new(name: impl Into<String>, values: Vec<T>) -> Self {
        Self { name: name.into(), values }
    }

    /// `getName`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// `getPossibleValues`.
    pub fn values(&self) -> &[T] {
        &self.values
    }

    /// `getPossibleValues` (сериализованные имена).
    pub fn possible_values_serialized(&self) -> Vec<String> {
        self.values.iter().map(|v| state_value_to_string(&v.to_state_value())).collect()
    }

    /// `getValue(String)` — парсит и проверяет вхождение в `values`.
    pub fn get_value(&self, s: &str) -> Option<T> {
        let parsed = T::parse(s)?;
        self.values.iter().find(|v| **v == parsed).cloned()
    }
}

impl<T: PropertyValue> PropertyTrait for Property<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn possible_values(&self) -> Vec<String> {
        self.possible_values_serialized()
    }

    fn parse(&self, s: &str) -> Option<StateValue> {
        self.get_value(s).map(|v| v.to_state_value())
    }
}

/// Сериализованное имя `StateValue` (для Str — само значение).
pub fn state_value_to_string(v: &StateValue) -> String {
    match v {
        StateValue::Int(i) => i.to_string(),
        StateValue::Bool(b) => b.to_string(),
        StateValue::Str(s) => s.clone(),
    }
}

/// Удобный алиас для type-erased ссылки на свойство.
pub type PropertyRef = Arc<dyn PropertyTrait>;

impl fmt::Debug for dyn PropertyTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Property({})", self.name())
    }
}
