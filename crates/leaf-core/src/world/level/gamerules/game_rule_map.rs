//! `net.minecraft.world.level.gamerules.GameRuleMap` — хранилище
//! значений правил (Paper array-backed `idAccess`).

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::nbt::{CompoundTag, Tag};

use super::game_rule::{value_to_string_pub, GameRule, GameRuleInner, GameRuleRegistry};
use super::game_rule_type::GameRuleType;

/// Нетипизированная ссылка на правило (для `keySet`).
#[derive(Clone)]
pub struct GameRuleRef {
    pub(crate) inner: Arc<GameRuleInner>,
}

impl GameRuleRef {
    pub(crate) fn from_inner(inner: Arc<GameRuleInner>) -> Self {
        Self { inner }
    }

    /// `id()`.
    pub fn id(&self) -> &crate::resources::identifier::Identifier {
        &self.inner.id
    }

    /// Тип значения.
    pub fn game_rule_type(&self) -> GameRuleType {
        self.inner.game_rule_type
    }

    /// Дефолтное значение как строка.
    pub fn default_value_string(&self) -> String {
        value_to_string_pub(&*self.inner.default_value)
    }
}

/// `GameRuleMap` — значения по `game_rule_index`.
#[derive(Default, Clone)]
pub struct GameRuleMap {
    map: HashMap<usize, Arc<dyn Any + Send + Sync>>,
    rules: HashMap<usize, Arc<GameRuleInner>>,
}

impl GameRuleMap {
    /// `GameRuleMap.of()` — пустая.
    pub fn new() -> Self {
        Self::default()
    }

    /// `GameRuleMap.of(rules)` — все дефолтные значения.
    pub fn of_defaults(registry: &GameRuleRegistry) -> Self {
        let mut m = Self::new();
        for inner in registry.entries() {
            m.rules.insert(inner.game_rule_index, inner.clone());
            m.map.insert(inner.game_rule_index, inner.default_value.clone());
        }
        m
    }

    /// `setDefault(inner)` — вставить дефолтное значение правила.
    pub(crate) fn set_default(&mut self, inner: Arc<GameRuleInner>) {
        let index = inner.game_rule_index;
        self.rules.insert(index, inner.clone());
        self.map.insert(index, inner.default_value.clone());
    }

    /// `has(rule)` — Paper: `idAccess[index] != null`.
    pub fn has<T>(&self, rule: &GameRule<T>) -> bool {
        self.map.contains_key(&rule.inner.game_rule_index)
    }

    /// `has` для нетипизированной ссылки.
    pub(crate) fn has_ref(&self, rule: &GameRuleRef) -> bool {
        self.map.contains_key(&rule.inner.game_rule_index)
    }

    /// `get(rule)` → `Option<T>`.
    pub fn get<T: Clone + Send + Sync + 'static>(&self, rule: &GameRule<T>) -> Option<T> {
        self.map
            .get(&rule.inner.game_rule_index)?
            .downcast_ref::<T>()
            .cloned()
    }

    /// `get(rule)` по нетипизированной ссылке → строка.
    pub fn get_string(&self, rule: &GameRuleRef) -> Option<String> {
        self.map
            .get(&rule.inner.game_rule_index)
            .map(|v| value_to_string_pub(&**v))
    }

    /// `set(rule, value)`.
    pub fn set<T: Clone + Send + Sync + 'static>(&mut self, rule: &GameRule<T>, value: T) {
        let index = rule.inner.game_rule_index;
        self.map.insert(index, Arc::new(value));
        self.rules.insert(index, rule.inner.clone());
    }

    /// `remove(rule)`.
    pub fn remove<T>(&mut self, rule: &GameRule<T>) -> Option<Arc<dyn Any + Send + Sync>> {
        self.map.remove(&rule.inner.game_rule_index)
    }

    /// `keySet()`.
    pub fn key_set(&self) -> Vec<GameRuleRef> {
        self.rules
            .values()
            .map(|inner| GameRuleRef { inner: inner.clone() })
            .collect()
    }

    /// `size()`.
    pub fn size(&self) -> usize {
        self.map.len()
    }

    /// `withOther(rules)` — merge, other имеет приоритет.
    pub fn with_other(&self, other: &GameRuleMap) -> Self {
        let mut m = self.copy_of();
        for (k, v) in &other.map {
            m.map.insert(*k, v.clone());
        }
        for (k, r) in &other.rules {
            m.rules.insert(*k, r.clone());
        }
        m
    }

    /// `setFromIf(rules, predicate)`.
    pub fn set_from_if(&mut self, rules: &GameRuleMap, predicate: impl Fn(&GameRuleRef) -> bool) {
        for (k, inner) in &rules.rules {
            if predicate(&GameRuleRef { inner: inner.clone() }) {
                if let Some(v) = rules.map.get(k) {
                    self.map.insert(*k, v.clone());
                    self.rules.insert(*k, inner.clone());
                }
            }
        }
    }

    /// `copyOf(rules)`.
    pub fn copy_of(&self) -> Self {
        Self {
            map: self.map.clone(),
            rules: self.rules.clone(),
        }
    }

    /// Доступ к inner по индексу (для `GameRules`).
    pub(crate) fn inner_by_index(&self, index: usize) -> Option<&Arc<GameRuleInner>> {
        self.rules.get(&index)
    }

    /// Значение по индексу как `&dyn Any`.
    pub(crate) fn value_by_index(&self, index: usize) -> Option<&dyn Any> {
        self.map.get(&index).map(|a| &**a as &dyn Any)
    }
}

/// `GameRuleMap.CODEC` → NBT compound (`name → value-string`),
/// как в level.dat секция `GameRules`.
impl GameRuleMap {
    /// Сериализация в `Tag::Compound`.
    pub fn to_nbt(&self) -> Tag {
        let mut compound = CompoundTag::new();
        for (index, inner) in &self.rules {
            if let Some(v) = self.map.get(index) {
                let s = value_to_string_pub(&**v);
                compound.put(inner.id.to_short_string(), Tag::String(s));
            }
        }
        Tag::Compound(compound)
    }

    /// Десериализация из NBT (`ofTrusted`).
    pub fn from_nbt(tag: &Tag, registry: &GameRuleRegistry) -> Self {
        let mut m = Self::new();
        if let Tag::Compound(c) = tag {
            for (name, value_tag) in c.iter() {
                if let (Some(inner), Tag::String(s)) = (registry.get(name), value_tag) {
                    let index = inner.game_rule_index;
                    let boxed: Arc<dyn Any + Send + Sync> = match inner.game_rule_type {
                        GameRuleType::Bool => Arc::new(s.parse::<bool>().unwrap_or(false)),
                        GameRuleType::Int => Arc::new(s.parse::<i32>().unwrap_or(0)),
                    };
                    m.rules.insert(index, inner.clone());
                    m.map.insert(index, boxed);
                }
            }
        }
        m
    }
}
