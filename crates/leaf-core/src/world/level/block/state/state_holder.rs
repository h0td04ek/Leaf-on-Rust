//! Перенос `net.minecraft.world.level.block.state.StateHolder` —
//! набор значений свойств состояния.

use super::property::{PropertyRef, PropertyTrait, StateValue};
use std::collections::BTreeMap;

/// `StateHolder<O, S>` — значения свойств одного состояния.
/// Ключ — имя свойства (`Property.getName`); порядок хранения
/// незначим для предикатов.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct StateHolder {
    values: BTreeMap<String, StateValue>,
}

impl StateHolder {
    /// `StateHolder.create` — пустой набор.
    pub fn new() -> Self {
        Self::default()
    }

    /// Установить значение свойства по типизированному свойству.
    pub fn set<T: super::property::PropertyValue>(&mut self, property: &super::property::Property<T>, value: T) {
        self.values.insert(property.name().to_string(), value.to_state_value());
    }

    /// Установить type-erased значение.
    pub fn set_erased(&mut self, name: &str, value: StateValue) {
        self.values.insert(name.to_string(), value);
    }

    /// `getValue(Property)` — `Option<StateValue>`.
    pub fn get(&self, property: &dyn PropertyTrait) -> Option<StateValue> {
        self.values.get(property.name()).cloned()
    }

    /// `getValue` по имени свойства.
    pub fn get_by_name(&self, name: &str) -> Option<StateValue> {
        self.values.get(name).cloned()
    }

    /// Типизированный доступ: `get::<i32>("level")`.
    pub fn get_typed<T: super::property::PropertyValue>(&self, property: &dyn PropertyTrait) -> Option<T> {
        let v = self.values.get(property.name())?;
        // Обратное преобразование StateValue → T через serialized name.
        let s = match v {
            StateValue::Int(i) => i.to_string(),
            StateValue::Bool(b) => b.to_string(),
            StateValue::Str(s) => s.clone(),
        };
        T::parse(&s)
    }

    /// Все пары (имя, значение).
    pub fn entries(&self) -> impl Iterator<Item = (&String, &StateValue)> {
        self.values.iter()
    }

    /// Число свойств.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Пуст ли набор.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl FromIterator<(String, StateValue)> for StateHolder {
    fn from_iter<I: IntoIterator<Item = (String, StateValue)>>(iter: I) -> Self {
        Self { values: iter.into_iter().collect() }
    }
}
