//! `net.minecraft.world.level.gamerules.GameRule<T>` — типизированный
//! дескриптор правила + реестр `BuiltInRegistries.GAME_RULE`.

use std::any::Any;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::resources::identifier::Identifier;
use crate::world::flag::feature_flag_set::FeatureFlagSet;

use super::game_rule_category::GameRuleCategory;
use super::game_rule_type::GameRuleType;

/// `GameRules.VisitorCaller<T>` — вызов visitor'а по типу.
pub type VisitorCaller<T> = fn(&mut dyn super::game_rules::GameRuleTypeVisitor, &GameRule<T>);

/// Brigadier `ArgumentType<T>`-эквивалент для gamerules
/// (только bool/int используются в ваниле).
#[derive(Debug, Clone)]
pub enum GameRuleArgument {
    /// `BoolArgumentType.bool()`.
    Bool,
    /// `IntegerArgumentType.integer(min, max)`.
    Int { min: i32, max: i32 },
}

impl GameRuleArgument {
    /// `parse(string)` — `Result<T-строка, ()>`; вызывающий код
    /// сам конвертирует в T через `parse_typed`.
    pub fn parse(&self, input: &str) -> Result<String, ()> {
        match self {
            GameRuleArgument::Bool => {
                if input.eq_ignore_ascii_case("true") || input.eq_ignore_ascii_case("false") {
                    Ok(input.to_string())
                } else {
                    Err(())
                }
            }
            GameRuleArgument::Int { min, max } => {
                let v: i32 = input.trim().parse().map_err(|_| ())?;
                if v >= *min && v <= *max {
                    Ok(input.to_string())
                } else {
                    Err(())
                }
            }
        }
    }
}

/// Внутренние данные правила (типизированные значения стёрты).
pub(crate) struct GameRuleInner {
    /// `id` — ключ в `BuiltInRegistries.GAME_RULE`.
    pub id: Identifier,
    pub category: GameRuleCategory,
    pub game_rule_type: GameRuleType,
    pub argument: GameRuleArgument,
    pub visitor_caller: VisitorCallerErased,
    pub default_value: Arc<dyn Any + Send + Sync>,
    pub command_result: std::sync::Arc<dyn Fn(&dyn Any) -> i32 + Send + Sync>,
    pub parse_value: std::sync::Arc<dyn Fn(&str) -> Option<Box<dyn Any + Send + Sync>> + Send + Sync>,
    pub required_features: FeatureFlagSet,
    /// Paper `gameRuleIndex`.
    pub game_rule_index: usize,
}

/// Type-erased visitor caller.
pub(crate) type VisitorCallerErased = fn(&mut dyn super::game_rules::GameRuleTypeVisitor, usize);

/// `GameRule<T>` — handle на запись реестра.
#[derive(Clone)]
pub struct GameRule<T> {
    pub(crate) inner: Arc<GameRuleInner>,
    pub(crate) _marker: PhantomData<fn() -> T>,
}

impl<T> GameRule<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// `id()`.
    pub fn id(&self) -> String {
        self.inner.id.to_short_string()
    }

    /// `getIdentifier()`.
    pub fn get_identifier(&self) -> &Identifier {
        &self.inner.id
    }

    /// `getDescriptionId()` — `gamerule.<namespace>.<path>`.
    pub fn get_description_id(&self) -> String {
        format!("gamerule.{}.{}", self.inner.id.namespace, self.inner.id.path)
    }

    /// `serialize(value)`.
    pub fn serialize(&self, value: &T) -> String {
        value_to_string_pub(value as &dyn Any)
    }

    /// `deserialize(string)` → `Result<T, ()>`.
    pub fn deserialize(&self, value: &str) -> Result<T, ()> {
        self.inner.argument.parse(value)?;
        (self.inner.parse_value)(value)
            .and_then(|b| b.downcast::<T>().ok().map(|b| *b))
            .ok_or(())
    }

    /// `valueClass()`.
    pub fn value_class_is_bool(&self) -> bool {
        self.inner.game_rule_type == GameRuleType::Bool
    }

    /// `callVisitor(visitor)`.
    pub fn call_visitor(&self, visitor: &mut dyn super::game_rules::GameRuleTypeVisitor) {
        (self.inner.visitor_caller)(visitor, self.inner.game_rule_index);
    }

    /// `getCommandResult(value)`.
    pub fn get_command_result(&self, value: &T) -> i32 {
        (self.inner.command_result)(value)
    }

    /// `category()`.
    pub fn category(&self) -> &GameRuleCategory {
        &self.inner.category
    }

    /// `gameRuleType()`.
    pub fn game_rule_type(&self) -> GameRuleType {
        self.inner.game_rule_type
    }

    /// `argument()`.
    pub fn argument(&self) -> &GameRuleArgument {
        &self.inner.argument
    }

    /// `defaultValue()`.
    pub fn default_value(&self) -> T {
        self.inner
            .default_value
            .downcast_ref::<T>()
            .expect("gamerule default type mismatch")
            .clone()
    }

    /// `requiredFeatures()`.
    pub fn required_features(&self) -> &FeatureFlagSet {
        &self.inner.required_features
    }

    /// Paper `gameRuleIndex`.
    pub fn game_rule_index(&self) -> usize {
        self.inner.game_rule_index
    }

    pub(crate) fn inner_arc(&self) -> Arc<GameRuleInner> {
        self.inner.clone()
    }

    /// Конструирование typed-правила из inner (для visitor'ов).
    pub(crate) fn from_inner(inner: Arc<GameRuleInner>) -> Self {
        Self { inner, _marker: PhantomData }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for GameRule<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameRule")
            .field("id", &self.inner.id)
            .field("index", &self.inner.game_rule_index)
            .finish()
    }
}

pub(crate) fn value_to_string_pub(value: &dyn Any) -> String {
    if let Some(b) = value.downcast_ref::<bool>() {
        b.to_string()
    } else if let Some(i) = value.downcast_ref::<i32>() {
        i.to_string()
    } else {
        String::new()
    }
}

/// `BuiltInRegistries.GAME_RULE` — реестр правил.
pub struct GameRuleRegistry {
    entries: Vec<Arc<GameRuleInner>>,
    by_name: std::collections::HashMap<Identifier, usize>,
}

impl GameRuleRegistry {
    fn new() -> Self {
        Self { entries: Vec::new(), by_name: std::collections::HashMap::new() }
    }

    /// Регистрация правила (аналог `Registry.register`).
    pub(crate) fn register<T>(
        &mut self,
        name: &str,
        category: GameRuleCategory,
        game_rule_type: GameRuleType,
        argument: GameRuleArgument,
        visitor_caller: VisitorCallerErased,
        default_value: T,
        command_result: fn(&T) -> i32,
        parse_value: Arc<dyn Fn(&str) -> Option<Box<dyn Any + Send + Sync>> + Send + Sync>,
        required_features: FeatureFlagSet,
    ) -> GameRule<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        static NEXT_INDEX: AtomicUsize = AtomicUsize::new(0);
        let index = NEXT_INDEX.fetch_add(1, Ordering::SeqCst);
        let id = Identifier::minecraft(name);
        let inner = Arc::new(GameRuleInner {
            id: id.clone(),
            category,
            game_rule_type,
            argument,
            visitor_caller,
            default_value: Arc::new(default_value),
            command_result: Arc::new(move |v: &dyn Any| {
                command_result(v.downcast_ref::<T>().expect("gamerule type mismatch"))
            }),
            parse_value,
            required_features,
            game_rule_index: index,
        });
        self.by_name.insert(id, self.entries.len());
        self.entries.push(inner.clone());
        GameRule { inner, _marker: PhantomData }
    }

    /// Поиск по имени.
    pub fn get(&self, name: &str) -> Option<&Arc<GameRuleInner>> {
        self.by_name
            .get(&Identifier::minecraft(name))
            .map(|&i| &self.entries[i])
    }

    /// Все записи.
    pub fn entries(&self) -> &[Arc<GameRuleInner>] {
        &self.entries
    }

    /// По индексу (Paper `gameRuleIndex`).
    pub fn by_index(&self, index: usize) -> Option<&Arc<GameRuleInner>> {
        self.entries.iter().find(|e| e.game_rule_index == index)
    }
}

/// Глобальный реестр (инициализируется `GameRules::bootstrap`).
pub fn game_rule_registry() -> &'static Mutex<GameRuleRegistry> {
    static REG: OnceLock<Mutex<GameRuleRegistry>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(GameRuleRegistry::new()))
}

/// Типизированный доступ к правилу по имени.
pub fn game_rule_bool(name: &str) -> GameRule<bool> {
    let reg = game_rule_registry().lock().unwrap();
    let inner = reg.get(name).unwrap_or_else(|| panic!("unknown gamerule {name}")).clone();
    GameRule { inner, _marker: PhantomData }
}

/// Типизированный доступ к int-правилу.
pub fn game_rule_int(name: &str) -> GameRule<i32> {
    let reg = game_rule_registry().lock().unwrap();
    let inner = reg.get(name).unwrap_or_else(|| panic!("unknown gamerule {name}")).clone();
    GameRule { inner, _marker: PhantomData }
}
