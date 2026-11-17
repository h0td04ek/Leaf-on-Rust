//! Перенос `net.minecraft.world.level.block.state.StateDefinition`
//! — упорядоченный набор свойств и все допустимые комбинации
//! состояний владельца (блок/жидкость).

use super::property::{PropertyRef, PropertyTrait};
use super::state_holder::StateHolder;
use crate::resources::identifier::Identifier;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// `StateDefinition<O, S>` — определение состояний.
///
/// `states` — все валидные комбинации (кэш `StateHolder`'ов);
/// `states_by_key` — ключ сериализованной карты имя→значение
/// для быстрого поиска.
pub struct StateDefinition {
    owner: Identifier,
    properties: Vec<PropertyRef>,
    states: Vec<StateHolder>,
    states_by_key: HashMap<Vec<(String, String)>, usize>,
}

impl StateDefinition {
    /// `StateDefinition.create(owner, factory, properties)` —
    /// перечисляет декартово произведение значений свойств.
    pub fn new(
        owner: Identifier,
        properties: Vec<PropertyRef>,
        states: Vec<StateHolder>,
    ) -> Self {
        let mut states_by_key = HashMap::new();
        for (i, s) in states.iter().enumerate() {
            states_by_key.insert(Self::state_key(s), i);
        }
        Self { owner, properties, states, states_by_key }
    }

    /// Владелец (`Block`/`Fluid` id).
    pub fn owner(&self) -> &Identifier {
        &self.owner
    }

    /// `getProperties` — упорядоченный список свойств.
    pub fn properties(&self) -> &[PropertyRef] {
        &self.properties
    }

    /// `getProperty(name)` — свойство по имени.
    pub fn get_property(&self, name: &str) -> Option<&PropertyRef> {
        self.properties.iter().find(|p| p.name() == name)
    }

    /// `getPossibleStates`.
    pub fn possible_states(&self) -> &[StateHolder] {
        &self.states
    }

    /// Число возможных состояний.
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    /// Поиск состояния по значениям (как `StateDefinition.any`).
    pub fn find_state(&self, values: &StateHolder) -> Option<&StateHolder> {
        let key = Self::state_key(values);
        self.states_by_key.get(&key).map(|&i| &self.states[i])
    }

    fn state_key(s: &StateHolder) -> Vec<(String, String)> {
        s.entries()
            .map(|(k, v)| {
                let val = match v {
                    super::property::StateValue::Int(i) => i.to_string(),
                    super::property::StateValue::Bool(b) => b.to_string(),
                    super::property::StateValue::Str(s) => s.clone(),
                };
                (k.clone(), val)
            })
            .collect()
    }
}

/// `StateDefinition.Builder`-эквивалент: построение всех
/// комбинаций значений свойств (декартово произведение).
pub fn enumerate_states(properties: &[PropertyRef]) -> Vec<StateHolder> {
    let mut states = vec![StateHolder::new()];
    for prop in properties {
        let mut next = Vec::with_capacity(states.len() * prop.possible_values().len().max(1));
        for state in &states {
            for value in prop.possible_values() {
                if let Some(sv) = prop.parse(&value) {
                    let mut s = state.clone();
                    s.set_erased(prop.name(), sv);
                    next.push(s);
                }
            }
        }
        states = next;
    }
    states
}
