//! Перенос `net.minecraft.world.level.storage.loot.LootContext`.

use crate::random::RandomSource;
use crate::util::context::context_key::ContextKey;
use crate::util::context::context_map::ContextMap;
use crate::world::entity::entity::Entity;
use crate::world::item::item_stack::ItemStack;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::storage::loot::parameters::loot_context_params;
use crate::world::phys::vec3::Vec3;
use std::collections::HashSet;
use std::sync::Arc;

/// `LootContext` — параметры + random + уровень для проверки
/// loot-условий и числовых провайдеров.
pub struct LootContext {
    /// `params` — `LootParams`.
    params: LootParams,
    /// `random`.
    random: Box<dyn RandomSource>,
    /// `level` — `ServerLevel` в Java; здесь `LevelReader`.
    level: Option<Arc<dyn LevelReader>>,
    /// `visitedElements` — защита от рекурсивных ссылок на условия.
    visited_elements: HashSet<String>,
}

impl LootContext {
    /// `LootContext(params, random, resolver)`.
    pub fn new(
        params: LootParams,
        random: Box<dyn RandomSource>,
        level: Option<Arc<dyn LevelReader>>,
    ) -> Self {
        Self { params, random, level, visited_elements: HashSet::new() }
    }

    /// `getLevel()` — `None` если контекст без уровня.
    pub fn level(&self) -> Option<&Arc<dyn LevelReader>> {
        self.level.as_ref()
    }

    /// `getRandom()`.
    pub fn random(&mut self) -> &mut dyn RandomSource {
        self.random.as_mut()
    }

    /// `hasParameter(key)`.
    pub fn has_parameter<T: 'static>(&self, key: &ContextKey<T>) -> bool {
        self.params.params.has(key)
    }

    /// `getParameter(key)` — паникует при отсутствии (как Java).
    pub fn get_parameter<T: 'static>(&self, key: &ContextKey<T>) -> &T {
        self.params.params.get(key).expect("missing loot context parameter")
    }

    /// `getOptionalParameter(key)`.
    pub fn get_optional_parameter<T: 'static>(&self, key: &ContextKey<T>) -> Option<&T> {
        self.params.params.get(key)
    }

    /// `addDynamicDrops(name, items)` — `LootParams.addDynamicDrops`.
    pub fn add_dynamic_drops(&mut self, name: crate::resources::identifier::Identifier, items: Vec<ItemStack>) {
        self.params.add_dynamic_drops(name, items);
    }

    /// `pushVisitedElement` — для `ConditionReference`.
    pub fn push_visited_element(&mut self, element: String) -> bool {
        self.visited_elements.insert(element)
    }

    /// `popVisitedElement`.
    pub fn pop_visited_element(&mut self, element: &str) {
        self.visited_elements.remove(element);
    }

    /// `hasVisitedElement`.
    pub fn has_visited_element(&self, element: &str) -> bool {
        self.visited_elements.contains(element)
    }
}

/// `LootParams` — карта параметров + динамические дропы.
pub struct LootParams {
    params: ContextMap,
    dynamic_drops: std::collections::HashMap<crate::resources::identifier::Identifier, Vec<ItemStack>>,
}

impl LootParams {
    /// `Builder`-эквивалент: пустые параметры.
    pub fn new() -> Self {
        Self { params: ContextMap::default(), dynamic_drops: Default::default() }
    }

    /// `withParameter(key, value)`.
    pub fn with_parameter<T: 'static>(mut self, key: &ContextKey<T>, value: T) -> Self {
        self.params.put(key, value);
        self
    }

    /// `withOptionalParameter(key, Option<value>)`.
    pub fn with_optional_parameter<T: 'static>(mut self, key: &ContextKey<T>, value: Option<T>) -> Self {
        if let Some(v) = value {
            self.params.put(key, v);
        }
        self
    }

    /// `params()` — доступ к карте.
    pub fn params(&self) -> &ContextMap {
        &self.params
    }

    /// `addDynamicDrops(name, consumer)` — складываем стопки.
    pub fn add_dynamic_drops(&mut self, name: crate::resources::identifier::Identifier, items: Vec<ItemStack>) {
        self.dynamic_drops.entry(name).or_default().extend(items);
    }

    /// `dynamicDrops(name)`.
    pub fn dynamic_drops(&self, name: &crate::resources::identifier::Identifier) -> &[ItemStack] {
        self.dynamic_drops.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

impl Default for LootParams {
    fn default() -> Self {
        Self::new()
    }
}

/// `LootContext.EntityTarget` — цель entity-параметра.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityTarget {
    /// `this`.
    This,
    /// `attacker`.
    Attacker,
    /// `direct_attacker`.
    DirectAttacker,
    /// `attacking_player`.
    AttackingPlayer,
    /// `target_entity`.
    TargetEntity,
    /// `interacting_entity`.
    InteractingEntity,
}

impl EntityTarget {
    /// `contextParam()`.
    pub fn context_param(&self) -> &'static ContextKey<Entity> {
        match self {
            EntityTarget::This => loot_context_params::this_entity(),
            EntityTarget::Attacker => loot_context_params::attacking_entity(),
            EntityTarget::DirectAttacker => loot_context_params::direct_attacking_entity(),
            EntityTarget::AttackingPlayer => loot_context_params::last_damage_player(),
            EntityTarget::TargetEntity => loot_context_params::target_entity(),
            EntityTarget::InteractingEntity => loot_context_params::interacting_entity(),
        }
    }

    /// serialized name (`getSerializedName`).
    pub fn serialized_name(&self) -> &'static str {
        match self {
            EntityTarget::This => "this",
            EntityTarget::Attacker => "attacker",
            EntityTarget::DirectAttacker => "direct_attacker",
            EntityTarget::AttackingPlayer => "attacking_player",
            EntityTarget::TargetEntity => "target_entity",
            EntityTarget::InteractingEntity => "interacting_entity",
        }
    }

    /// `getByName`.
    pub fn by_name(name: &str) -> Option<Self> {
        Some(match name {
            "this" => EntityTarget::This,
            "attacker" => EntityTarget::Attacker,
            "direct_attacker" => EntityTarget::DirectAttacker,
            "attacking_player" => EntityTarget::AttackingPlayer,
            "target_entity" => EntityTarget::TargetEntity,
            "interacting_entity" => EntityTarget::InteractingEntity,
            _ => return None,
        })
    }
}

/// `LootContext.ItemStackTarget`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemStackTarget {
    /// `tool`.
    Tool,
}

impl ItemStackTarget {
    /// `contextParam()`.
    pub fn context_param(&self) -> &'static ContextKey<ItemStack> {
        loot_context_params::tool()
    }

    /// `getSerializedName`.
    pub fn serialized_name(&self) -> &'static str {
        "tool"
    }
}
