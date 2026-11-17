//! Перенос `net.minecraft.network.chat.contents.*` — типы содержимого
//! компонентов (ComponentContents и реализации).
//!
//! `resolve()`-методы, требующие `CommandSourceStack`/сервера, будут
//! подключены при порте command/entity-слоёв; visit-обход для
//! неразрешённых типов не даёт вывода — как в Java.

use crate::locale::language::{language_generation, Language};
use crate::network::chat::component::Component;
use crate::network::chat::formatted_text::FormattedText;
use crate::network::chat::style::Style;
use crate::resources::identifier::Identifier;
use crate::util::either::Either;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

// ------------------------------------------------------------ SelectorPattern

/// `SelectorPattern` — паттерн селектора сущностей.
/// `resolved` (EntitySelector) появится с портом команд; хранится pattern.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SelectorPattern {
    pub pattern: String,
}

impl SelectorPattern {
    pub fn new(pattern: impl Into<String>) -> SelectorPattern {
        SelectorPattern { pattern: pattern.into() }
    }

    /// `parse` — валидация через EntitySelectorParser (портируется с commands).
    /// Пока создаём из строки без компиляции селектора.
    pub fn parse(pattern: &str) -> Result<SelectorPattern, String> {
        Ok(SelectorPattern::new(pattern))
    }
}

impl std::fmt::Display for SelectorPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pattern)
    }
}

// ------------------------------------------------------------ DataSource

/// `BlockDataSource` — `compiledPos` появится с портом Coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockDataSource {
    pub pos_pattern: String,
    /// Распарсенные координаты (x, y, z) — после порта BlockPosArgument.
    pub compiled_pos: Option<(f64, f64, f64)>,
}

impl BlockDataSource {
    pub fn new(pos_pattern: impl Into<String>) -> BlockDataSource {
        BlockDataSource { pos_pattern: pos_pattern.into(), compiled_pos: None }
    }
}

/// `EntityDataSource` — `compiledSelector` после порта селекторов.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityDataSource {
    pub selector_pattern: String,
}

/// `StorageDataSource`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StorageDataSource {
    pub id: Identifier,
}

/// `DataSource` — источник NBT-данных для `nbt`-компонента.
#[derive(Clone, Debug, PartialEq)]
pub enum DataSource {
    Entity(EntityDataSource),
    Block(BlockDataSource),
    Storage(StorageDataSource),
}

// ------------------------------------------------------------ ObjectInfo

/// `AtlasSprite`/`PlayerSprite` — объект для `object`-компонента.
#[derive(Clone, Debug, PartialEq)]
pub enum ObjectInfo {
    AtlasSprite { atlas: Identifier, sprite: Identifier },
    PlayerSprite { profile_name: String, profile_id: Option<crate::util::uuid::Uuid>, hat: bool },
}

// ------------------------------------------------------------ TranslatableArg

/// Аргумент `translatable`-компонента (`Object` в Java).
/// Непримитивные объекты нормализуются в строку при создании
/// (`translatableEscape`), компоненты сохраняются как Component.
#[derive(Clone, Debug, PartialEq)]
pub enum TranslatableArg {
    /// `Component`-аргумент.
    Component(Box<Component>),
    /// Примитивный аргумент (Number/Boolean/String/прочее → toString).
    Text(String),
}

impl TranslatableArg {
    /// Аргумент из значения, реализующего Display (аналог `translationArg`).
    pub fn from_display<T: std::fmt::Display>(value: &T) -> TranslatableArg {
        TranslatableArg::Text(value.to_string())
    }
}

// ------------------------------------------------------------ TranslatableContents

/// `TranslatableFormatException`.
#[derive(Clone, Debug)]
pub struct TranslatableFormatException {
    pub key: String,
    pub message: String,
}

impl std::fmt::Display for TranslatableFormatException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.key, self.message)
    }
}

impl std::error::Error for TranslatableFormatException {}

/// `TranslatableContents` — `%s`-шаблон из Language.
#[derive(Clone)]
pub struct TranslatableContents {
    pub key: String,
    pub fallback: Option<String>,
    pub args: Vec<TranslatableArg>,
    /// Кэш декомпозиции: (поколение языка, части).
    decomposed: Arc<Mutex<Option<(u64, Vec<FormattedText>)>>>,
}

impl TranslatableContents {
    /// `new(key, fallback, args)`.
    pub fn new(key: impl Into<String>, fallback: Option<String>, args: Vec<TranslatableArg>) -> Self {
        Self {
            key: key.into(),
            fallback,
            args,
            decomposed: Arc::new(Mutex::new(None)),
        }
    }

    /// `isAllowedPrimitiveArgument` — Number/Boolean/String.
    /// В Rust-модели примитивы уже нормализованы в Text.
    pub fn is_allowed_primitive_argument(_arg: &TranslatableArg) -> bool {
        matches!(_arg, TranslatableArg::Text(_))
    }

    fn decompose(&self) -> Vec<FormattedText> {
        let generation = language_generation();
        {
            let cache = self.decomposed.lock().unwrap();
            if let Some((gen, parts)) = cache.as_ref() {
                if *gen == generation {
                    return parts.clone();
                }
            }
        }
        let language = Language::get_instance();
        let template = match &self.fallback {
            Some(fb) => language.get_or_default_with(&self.key, fb),
            None => language.get_or_default(&self.key),
        };
        let mut parts = Vec::new();
        match self.decompose_template(&template) {
            Ok(decomposed) => parts = decomposed,
            Err(_) => parts.push(FormattedText::of(template)),
        }
        *self.decomposed.lock().unwrap() = Some((generation, parts.clone()));
        parts
    }

    fn get_argument(&self, index: usize) -> Result<FormattedText, TranslatableFormatException> {
        match self.args.get(index) {
            Some(TranslatableArg::Component(c)) => Ok(FormattedText::Component((**c).clone())),
            Some(TranslatableArg::Text(s)) => {
                if s == "null" {
                    Ok(FormattedText::of("null"))
                } else {
                    Ok(FormattedText::of(s.clone()))
                }
            }
            None => Err(TranslatableFormatException {
                key: self.key.clone(),
                message: format!("Argument index {} out of range", index),
            }),
        }
    }

    /// `decomposeTemplate` — `%[n$]s`, `%%`.
    fn decompose_template(
        &self,
        template: &str,
    ) -> Result<Vec<FormattedText>, TranslatableFormatException> {
        static FORMAT_PATTERN: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(r"%(?:(\d+)\$)?([A-Za-z%]|$)").unwrap()
        });
        let mut parts = Vec::new();
        let mut last_end = 0usize;
        let mut auto_index = 0usize;
        for m in FORMAT_PATTERN.find_iter(template) {
            let start = m.start();
            let end = m.end();
            if start > last_end {
                let sub = &template[last_end..start];
                if sub.contains('%') {
                    return Err(TranslatableFormatException {
                        key: self.key.clone(),
                        message: "Unexpected '%' in literal".into(),
                    });
                }
                parts.push(FormattedText::of(sub));
            }
            let matched = m.as_str();
            // группа 1: \d+\$ ; группа 2: [A-Za-z%]|$
            let caps = FORMAT_PATTERN.captures(matched).unwrap();
            let arg_num = caps.get(1).map(|g| g.as_str().trim_end_matches('$').to_string());
            let fmt_char = caps.get(2).map(|g| g.as_str()).unwrap_or("");
            if matched == "%%" {
                parts.push(FormattedText::of("%"));
            } else {
                if fmt_char != "s" {
                    return Err(TranslatableFormatException {
                        key: self.key.clone(),
                        message: format!("Unsupported format: '{}'", matched),
                    });
                }
                let idx = match arg_num {
                    Some(n) => n
                        .parse::<usize>()
                        .map_err(|_| TranslatableFormatException {
                            key: self.key.clone(),
                            message: format!("Invalid argument index: {}", n),
                        })?
                        .checked_sub(1)
                        .ok_or_else(|| TranslatableFormatException {
                            key: self.key.clone(),
                            message: "Argument index 0".into(),
                        })?,
                    None => {
                        let i = auto_index;
                        auto_index += 1;
                        i
                    }
                };
                parts.push(self.get_argument(idx)?);
            }
            last_end = end;
        }
        if last_end < template.len() {
            let tail = &template[last_end..];
            if tail.contains('%') {
                return Err(TranslatableFormatException {
                    key: self.key.clone(),
                    message: "Unexpected '%' in literal".into(),
                });
            }
            parts.push(FormattedText::of(tail));
        }
        Ok(parts)
    }
}

impl PartialEq for TranslatableContents {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.fallback == other.fallback && self.args == other.args
    }
}

impl Eq for TranslatableContents {}

impl std::fmt::Debug for TranslatableContents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranslatableContents")
            .field("key", &self.key)
            .field("fallback", &self.fallback)
            .field("args", &self.args)
            .finish()
    }
}

// ------------------------------------------------------------ KeybindResolver

/// `KeybindResolver` — резолвер keybind-имён в компоненты.
type KeyResolver = Arc<dyn Fn(&str) -> Component + Send + Sync>;

fn default_key_resolver() -> KeyResolver {
    Arc::new(|name: &str| Component::literal(name))
}

static KEY_RESOLVER: RwLock<Option<KeyResolver>> = RwLock::new(None);

/// `setKeyResolver`.
pub fn set_key_resolver(resolver: KeyResolver) {
    *KEY_RESOLVER.write().unwrap() = Some(resolver);
}

fn resolve_keybind(name: &str) -> Component {
    let guard = KEY_RESOLVER.read().unwrap();
    match guard.as_ref() {
        Some(r) => r(name),
        None => default_key_resolver()(name),
    }
}

// ------------------------------------------------------------ ComponentContents

/// `ComponentContents` — содержимое компонента.
#[derive(Clone, Debug)]
pub enum ComponentContents {
    /// `PlainTextContents` (literal/empty).
    PlainText(String),
    /// `TranslatableContents`.
    Translatable(TranslatableContents),
    /// `KeybindContents`.
    Keybind(String),
    /// `ScoreContents`.
    Score { name: Either<SelectorPattern, String>, objective: String },
    /// `SelectorContents`.
    Selector { selector: SelectorPattern, separator: Option<Box<Component>> },
    /// `NbtContents`.
    Nbt {
        nbt_path_pattern: String,
        interpreting: bool,
        separator: Option<Box<Component>>,
        data_source: DataSource,
    },
    /// `ObjectContents`.
    Object(ObjectInfo),
}

impl ComponentContents {
    /// `visit(ContentConsumer)`.
    pub fn visit_content<F>(&self, acceptor: &mut F) -> bool
    where
        F: FnMut(&str) -> bool,
    {
        match self {
            ComponentContents::PlainText(text) => acceptor(text),
            ComponentContents::Translatable(t) => {
                for part in t.decompose() {
                    if !part.visit_content(acceptor) {
                        return false;
                    }
                }
                true
            }
            ComponentContents::Keybind(name) => resolve_keybind(name).visit_content(acceptor),
            // Score/Selector/Nbt/Object не выводят текст до resolve().
            _ => true,
        }
    }

    /// `visit(StyledContentConsumer, style)`.
    pub fn visit_styled<F>(&self, style: &Style, acceptor: &mut F) -> bool
    where
        F: FnMut(&Style, &str) -> bool,
    {
        match self {
            ComponentContents::PlainText(text) => acceptor(style, text),
            ComponentContents::Translatable(t) => {
                for part in t.decompose() {
                    if !part.visit_styled(style, acceptor) {
                        return false;
                    }
                }
                true
            }
            ComponentContents::Keybind(name) => resolve_keybind(name).visit_styled(style, acceptor),
            _ => true,
        }
    }
}

impl PartialEq for ComponentContents {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ComponentContents::PlainText(a), ComponentContents::PlainText(b)) => a == b,
            (ComponentContents::Translatable(a), ComponentContents::Translatable(b)) => a == b,
            (ComponentContents::Keybind(a), ComponentContents::Keybind(b)) => a == b,
            (
                ComponentContents::Score { name: n1, objective: o1 },
                ComponentContents::Score { name: n2, objective: o2 },
            ) => n1 == n2 && o1 == o2,
            (
                ComponentContents::Selector { selector: s1, separator: sep1 },
                ComponentContents::Selector { selector: s2, separator: sep2 },
            ) => s1 == s2 && sep1 == sep2,
            (
                ComponentContents::Nbt {
                    nbt_path_pattern: p1,
                    interpreting: i1,
                    separator: s1,
                    data_source: d1,
                },
                ComponentContents::Nbt {
                    nbt_path_pattern: p2,
                    interpreting: i2,
                    separator: s2,
                    data_source: d2,
                },
            ) => p1 == p2 && i1 == i2 && s1 == s2 && d1 == d2,
            (ComponentContents::Object(a), ComponentContents::Object(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ComponentContents {}
