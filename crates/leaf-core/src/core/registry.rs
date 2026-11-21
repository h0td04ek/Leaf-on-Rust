//! Перенос `net.minecraft.core.Registry`/`MappedRegistry` —
//! реальная реализация реестра: упорядоченные записи,
//! `ResourceKey`-индексы, holder'ы (`Holder::Reference`).

use crate::core::holder::Holder;
use crate::resources::resource_key::ResourceKey;
use crate::resources::Identifier;
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

/// `Registry<T>` — упорядоченный реестр значений по `ResourceKey`.
///
/// Порядок регистрации = wire-id (holderRegistry-кодек пишет
/// 0-based индекс). Значения T должны быть `Clone` (реестр
/// отдаёт ссылки через `Holder::Direct` при inline-использовании).
pub struct Registry<T> {
    entries: Vec<RegistryEntry<T>>,
    by_key: HashMap<ResourceKey<T>, usize>,
    frozen: bool,
}

#[derive(Clone)]
struct RegistryEntry<T> {
    key: ResourceKey<T>,
    value: T,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Registry<T> {
    /// `new MappedRegistry`.
    pub fn new() -> Self {
        Self { entries: Vec::new(), by_key: HashMap::new(), frozen: false }
    }

    /// `register(ResourceKey, T)` — возвращает holder записи.
    /// Паника при повторной регистрации (как `IllegalStateException`
    /// в Java при замене).
    pub fn register(&mut self, key: ResourceKey<T>, value: T) -> Holder<T> {
        assert!(!self.frozen, "registry is frozen");
        assert!(!self.by_key.contains_key(&key), "duplicate key {:?}", key.id());
        let id = self.entries.len() as i32;
        self.entries.push(RegistryEntry { key: key.clone(), value });
        self.by_key.insert(key, id as usize);
        Holder::Reference(id)
    }

    /// `freeze` — запрещает дальнейшую регистрацию.
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    /// `isFrozen`.
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// `size`.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `getId(ResourceKey)` — 0-based wire-id.
    pub fn get_id(&self, key: &ResourceKey<T>) -> Option<i32> {
        self.by_key.get(key).map(|&i| i as i32)
    }

    /// `getKey(T)` — `ResourceKey` по значению.
    pub fn get_key(&self, value: &T) -> Option<&ResourceKey<T>> {
        self.entries.iter().find(|e| &e.value == value).map(|e| &e.key)
    }

    /// `getValue(ResourceKey)`.
    pub fn get_value(&self, key: &ResourceKey<T>) -> Option<&T> {
        self.by_key.get(key).map(|&i| &self.entries[i].value)
    }

    /// `getValueOrThrow`.
    pub fn get_value_or_throw(&self, key: &ResourceKey<T>) -> &T {
        self.get_value(key).unwrap_or_else(|| panic!("missing registry entry {:?}", key.id()))
    }

    /// `getOptional(ResourceKey)`.
    pub fn get_optional(&self, key: &ResourceKey<T>) -> Option<&T> {
        self.get_value(key)
    }

    /// `getHolder(ResourceKey)` — `Holder::Reference` по ключу.
    pub fn get_holder(&self, key: &ResourceKey<T>) -> Option<Holder<T>> {
        self.get_id(key).map(Holder::Reference)
    }

    /// `getHolderOrThrow(ResourceKey)`.
    pub fn get_holder_or_throw(&self, key: &ResourceKey<T>) -> Holder<T> {
        self.get_holder(key).unwrap_or_else(|| panic!("missing holder {:?}", key.id()))
    }

    /// `getHolder(int)` — holder по registry-id.
    pub fn get_holder_by_id(&self, id: i32) -> Option<Holder<T>> {
        (id >= 0 && (id as usize) < self.entries.len()).then_some(Holder::Reference(id))
    }

    /// `getValue(int)` — значение по registry-id.
    pub fn get_value_by_id(&self, id: i32) -> Option<&T> {
        self.entries.get(id as usize).map(|e| &e.value)
    }

    /// `getKey(int)`.
    pub fn get_key_by_id(&self, id: i32) -> Option<&ResourceKey<T>> {
        self.entries.get(id as usize).map(|e| &e.key)
    }

    /// `entrySet` — `(ResourceKey, &T)` в порядке регистрации.
    pub fn entry_set(&self) -> impl Iterator<Item = (&ResourceKey<T>, &T)> {
        self.entries.iter().map(|e| (&e.key, &e.value))
    }

    /// `stream` / `keySet`.
    pub fn keys(&self) -> impl Iterator<Item = &ResourceKey<T>> {
        self.entries.iter().map(|e| &e.key)
    }

    /// `getId(T)` — id по значению.
    pub fn get_id_of_value(&self, value: &T) -> Option<i32> {
        self.entries.iter().position(|e| &e.value == value).map(|i| i as i32)
    }

    /// Поиск по `Identifier` (location).
    pub fn get_value_by_identifier(&self, id: &Identifier) -> Option<&T> {
        self.entries.iter().find(|e| e.key.id() == id).map(|e| &e.value)
    }

    /// `getHolderOrThrow(Identifier)`.
    pub fn get_holder_by_identifier(&self, id: &Identifier) -> Option<Holder<T>> {
        self.entries
            .iter()
            .position(|e| e.key.id() == id)
            .map(|i| Holder::Reference(i as i32))
    }

    /// Все значения по порядку.
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().map(|e| &e.value)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Registry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registry").field("size", &self.entries.len()).finish()
    }
}

impl<T: PartialEq> PartialEq for Registry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.entries.len() == other.entries.len()
            && self.entries.iter().zip(other.entries.iter()).all(|(a, b)| a.key == b.key && a.value == b.value)
    }
}
impl<T: PartialEq> Eq for Registry<T> {}

impl<T> Default for Registry<T>
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// `RegistryAccess` — типизированный доступ к реестрам уровня
/// (алиас над `RegistryAccessMap`).
pub type RegistryAccess = RegistryAccessMap;

/// Type-erased `Registry<T>` для хранения в `RegistryAccess`.
struct AnyRegistryEntry {
    name: Identifier,
    inner: Arc<dyn Any + Send + Sync>,
}

/// Расширенный `RegistryAccess` с type-erased хранением —
/// позволяет `lookup::<T>` без `AnyRegistry` bound-проблем.
#[derive(Default)]
pub struct RegistryAccessMap {
    registries: RwLock<HashMap<Identifier, Arc<dyn Any + Send + Sync>>>,
}

impl RegistryAccessMap {
    /// `new`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `register(name, registry)`.
    pub fn register<T: Clone + PartialEq + Send + Sync + 'static>(
        &self,
        name: Identifier,
        registry: Arc<Registry<T>>,
    ) {
        self.registries.write().unwrap().insert(name, registry);
    }

    /// `lookup(name)` → `Arc<Registry<T>>`.
    pub fn lookup<T: Clone + PartialEq + Send + Sync + 'static>(
        &self,
        name: &Identifier,
    ) -> Option<Arc<Registry<T>>> {
        self.registries
            .read()
            .unwrap()
            .get(name)?
            .clone()
            .downcast::<Registry<T>>()
            .ok()
    }

    /// `registryOrThrow`.
    pub fn registry_or_throw<T: Clone + PartialEq + Send + Sync + 'static>(
        &self,
        name: &Identifier,
    ) -> Arc<Registry<T>> {
        self.lookup(name).unwrap_or_else(|| panic!("missing registry {name}"))
    }

    /// Имена реестров.
    pub fn registry_names(&self) -> Vec<Identifier> {
        self.registries.read().unwrap().keys().cloned().collect()
    }
}

/// Глобальный доступ к `RegistryAccess` (`BuiltInRegistries` +
/// per-level `RegistryAccess`). Заполняется bootstrap'ом.
pub fn builtin_registry_access() -> &'static std::sync::OnceLock<Arc<RegistryAccessMap>> {
    static ACCESS: std::sync::OnceLock<Arc<RegistryAccessMap>> = std::sync::OnceLock::new();
    &ACCESS
}

pub mod registries {
    use crate::resources::Identifier;

    /// `Registries.ROOT_REGISTRY_NAME`
    pub fn root_registry_name() -> Identifier {
        Identifier::minecraft("minecraft")
    }

    pub fn block() -> Identifier {
        Identifier::minecraft("block")
    }
    pub fn item() -> Identifier {
        Identifier::minecraft("item")
    }
    pub fn entity_type() -> Identifier {
        Identifier::minecraft("entity_type")
    }
    pub fn menu() -> Identifier {
        Identifier::minecraft("menu")
    }
    pub fn sound_event() -> Identifier {
        Identifier::minecraft("sound_event")
    }
    pub fn mob_effect() -> Identifier {
        Identifier::minecraft("mob_effect")
    }
    pub fn dimension_type() -> Identifier {
        Identifier::minecraft("dimension_type")
    }
    pub fn dimension() -> Identifier {
        Identifier::minecraft("dimension")
    }
    pub fn timeline() -> Identifier {
        Identifier::minecraft("timeline")
    }
    pub fn biome() -> Identifier {
        Identifier::minecraft("worldgen/biome")
    }
    pub fn configured_carver() -> Identifier {
        Identifier::minecraft("worldgen/configured_carver")
    }
    pub fn placed_feature() -> Identifier {
        Identifier::minecraft("worldgen/placed_feature")
    }
    pub fn structure() -> Identifier {
        Identifier::minecraft("worldgen/structure")
    }
    pub fn structure_set() -> Identifier {
        Identifier::minecraft("worldgen/structure_set")
    }
    pub fn enchantment() -> Identifier {
        Identifier::minecraft("enchantment")
    }
    pub fn damage_type() -> Identifier {
        Identifier::minecraft("damage_type")
    }
    pub fn trim_pattern() -> Identifier {
        Identifier::minecraft("trim_pattern")
    }
    pub fn trim_material() -> Identifier {
        Identifier::minecraft("trim_material")
    }
    pub fn banner_pattern() -> Identifier {
        Identifier::minecraft("banner_pattern")
    }
    pub fn instrument() -> Identifier {
        Identifier::minecraft("instrument")
    }
    pub fn painting_variant() -> Identifier {
        Identifier::minecraft("painting_variant")
    }
    pub fn wolf_variant() -> Identifier {
        Identifier::minecraft("wolf_variant")
    }
    pub fn wolf_sound_variant() -> Identifier {
        Identifier::minecraft("wolf_sound_variant")
    }
    pub fn cat_variant() -> Identifier {
        Identifier::minecraft("cat_variant")
    }
    pub fn cat_sound_variant() -> Identifier {
        Identifier::minecraft("cat_sound_variant")
    }
    pub fn chicken_variant() -> Identifier {
        Identifier::minecraft("chicken_variant")
    }
    pub fn cow_variant() -> Identifier {
        Identifier::minecraft("cow_variant")
    }
    pub fn frog_variant() -> Identifier {
        Identifier::minecraft("frog_variant")
    }
    pub fn pig_variant() -> Identifier {
        Identifier::minecraft("pig_variant")
    }
    pub fn wolf_sound_variant_legacy() -> Identifier {
        Identifier::minecraft("wolf_sound_variant")
    }
    pub fn armadillo_sound_variant() -> Identifier {
        Identifier::minecraft("armadillo_sound_variant")
    }
    pub fn armadillo_variant() -> Identifier {
        Identifier::minecraft("armadillo_variant")
    }
    pub fn trial_spawner() -> Identifier {
        Identifier::minecraft("trial_spawner")
    }
    pub fn trial_chamber() -> Identifier {
        Identifier::minecraft("trial_chamber")
    }
    pub fn point_of_interest_type() -> Identifier {
        Identifier::minecraft("point_of_interest_type")
    }
    pub fn villager_type() -> Identifier {
        Identifier::minecraft("villager_type")
    }
    pub fn villager_profession() -> Identifier {
        Identifier::minecraft("villager_profession")
    }
    pub fn memory_module_type() -> Identifier {
        Identifier::minecraft("memory_module_type")
    }
    pub fn sensor_type() -> Identifier {
        Identifier::minecraft("sensor_type")
    }
    pub fn schedule() -> Identifier {
        Identifier::minecraft("schedule")
    }
    pub fn activity() -> Identifier {
        Identifier::minecraft("activity")
    }
    pub fn world_clock() -> Identifier {
        Identifier::minecraft("world_clock")
    }
    pub fn loot_pool_entry_type() -> Identifier {
        Identifier::minecraft("loot_pool_entry_type")
    }
    pub fn loot_function_type() -> Identifier {
        Identifier::minecraft("loot_function_type")
    }
    pub fn loot_condition_type() -> Identifier {
        Identifier::minecraft("loot_condition_type")
    }
    pub fn loot_number_provider_type() -> Identifier {
        Identifier::minecraft("loot_number_provider_type")
    }
    pub fn loot_score_provider_type() -> Identifier {
        Identifier::minecraft("loot_score_provider_type")
    }
    pub fn float_provider_type() -> Identifier {
        Identifier::minecraft("float_provider_type")
    }
    pub fn argument_type_info() -> Identifier {
        Identifier::minecraft("command_argument_type")
    }
    pub fn test_environment_definition() -> Identifier {
        Identifier::minecraft("test_environment")
    }
    pub fn test_instance_type() -> Identifier {
        Identifier::minecraft("test_instance_type")
    }
    pub fn game_event() -> Identifier {
        Identifier::minecraft("game_event")
    }
    pub fn recipe_type() -> Identifier {
        Identifier::minecraft("recipe_type")
    }
    pub fn recipe_serializer() -> Identifier {
        Identifier::minecraft("recipe_serializer")
    }
    pub fn attribute() -> Identifier {
        Identifier::minecraft("attribute")
    }
    pub fn stats() -> Identifier {
        Identifier::minecraft("stats")
    }
    pub fn custom_stat() -> Identifier {
        Identifier::minecraft("custom_stat")
    }
    pub fn chat_type() -> Identifier {
        Identifier::minecraft("chat_type")
    }
    pub fn dialog_type() -> Identifier {
        Identifier::minecraft("dialog_type")
    }
    pub fn position_source_type() -> Identifier {
        Identifier::minecraft("position_source_type")
    }
    pub fn particle_type() -> Identifier {
        Identifier::minecraft("particle_type")
    }
    pub fn block_state_predicate_type() -> Identifier {
        Identifier::minecraft("block_state_predicate_type")
    }
    pub fn rule_block_entity_modifier() -> Identifier {
        Identifier::minecraft("rule_block_entity_modifier")
    }
    pub fn rule_test() -> Identifier {
        Identifier::minecraft("rule_test")
    }
    pub fn decorator() -> Identifier {
        Identifier::minecraft("decorator")
    }
    pub fn material_condition() -> Identifier {
        Identifier::minecraft("material_condition")
    }
    pub fn material_rule() -> Identifier {
        Identifier::minecraft("material_rule")
    }
    pub fn map_decoration_type() -> Identifier {
        Identifier::minecraft("map_decoration_type")
    }
    pub fn chunk_status() -> Identifier {
        Identifier::minecraft("chunk_status")
    }
    pub fn worldgen_height_provider_type() -> Identifier {
        Identifier::minecraft("worldgen/height_provider_type")
    }
    pub fn worldgen_block_predicate_type() -> Identifier {
        Identifier::minecraft("worldgen/block_predicate_type")
    }
    pub fn worldgen_foliage_placer_type() -> Identifier {
        Identifier::minecraft("worldgen/foliage_placer_type")
    }
    pub fn worldgen_tree_decorator_type() -> Identifier {
        Identifier::minecraft("worldgen/tree_decorator_type")
    }
    pub fn worldgen_trunk_placer_type() -> Identifier {
        Identifier::minecraft("worldgen/trunk_placer_type")
    }
    pub fn worldgen_feature() -> Identifier {
        Identifier::minecraft("worldgen/feature")
    }
    pub fn worldgen_noise_settings() -> Identifier {
        Identifier::minecraft("worldgen/noise_settings")
    }
    pub fn worldgen_density_function_type() -> Identifier {
        Identifier::minecraft("worldgen/density_function_type")
    }
    pub fn worldgen_noise() -> Identifier {
        Identifier::minecraft("worldgen/noise")
    }
    pub fn worldgen_surface_rule() -> Identifier {
        Identifier::minecraft("worldgen/surface_rule")
    }
    pub fn worldgen_carver() -> Identifier {
        Identifier::minecraft("worldgen/carver")
    }
    pub fn worldgen_placement_modifier_type() -> Identifier {
        Identifier::minecraft("worldgen/placement_modifier_type")
    }
    pub fn worldgen_structure_placement_type() -> Identifier {
        Identifier::minecraft("worldgen/structure_placement")
    }
    pub fn worldgen_structure_piece() -> Identifier {
        Identifier::minecraft("worldgen/structure_piece")
    }
    pub fn worldgen_structure_type() -> Identifier {
        Identifier::minecraft("worldgen/structure")
    }
    pub fn worldgen_template_pool() -> Identifier {
        Identifier::minecraft("worldgen/template_pool")
    }
    pub fn banner_pattern_legacy() -> Identifier {
        Identifier::minecraft("banner_pattern")
    }
}

/// `Registries.ROOT_REGISTRY_NAME` — как функция (Identifier не const).
pub fn root_registry_name() -> Identifier {
    registries::root_registry_name()
}
