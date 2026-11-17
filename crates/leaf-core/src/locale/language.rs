//! Перенос `net.minecraft.locale.Language`.
//!
//! `getVisualOrder` использует `StringDecomposer` — пока возвращает
//! текст как есть (LTR-порядок); bidi-развёртка добавится с портом
//! `FormattedCharSequence`/`StringDecomposer`.

use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// `Language.DEFAULT`.
pub const DEFAULT_LANGUAGE: &str = "en_us";

/// Поколение инстанса — для инвалидации кэшей (`TranslatableContents`).
static GENERATION: AtomicU64 = AtomicU64::new(0);

fn next_generation() -> u64 {
    GENERATION.fetch_add(1, Ordering::Relaxed) + 1
}

/// Текущее поколение языка (аналог сравнения `instance != decomposedWith`).
pub fn language_generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

/// `Language` — словарь строк + флаг RTL.
pub struct Language {
    storage: HashMap<String, String>,
    right_to_left: bool,
}

impl Language {
    /// Пустой язык (все ключи → сами ключи).
    pub fn empty() -> Language {
        Language { storage: HashMap::new(), right_to_left: false }
    }

    /// Язык из готовой map.
    pub fn new(storage: HashMap<String, String>, right_to_left: bool) -> Language {
        Language { storage, right_to_left }
    }

    /// `getOrDefault(id)` — ключ, если перевода нет.
    pub fn get_or_default(&self, id: &str) -> String {
        self.get_or_default_with(id, id)
    }

    /// `getOrDefault(key, defaultValue)`.
    pub fn get_or_default_with(&self, key: &str, default_value: &str) -> String {
        self.storage.get(key).cloned().unwrap_or_else(|| default_value.to_string())
    }

    /// `has(id)`.
    pub fn has(&self, id: &str) -> bool {
        self.storage.contains_key(id)
    }

    /// `isDefaultRightToLeft`.
    pub fn is_default_right_to_left(&self) -> bool {
        self.right_to_left
    }

    /// `loadFromJson(stream, output)` — парсит flat JSON `{ "key": "value" }`.
    /// Поддерживает только строковые значения (как lang-файлы Minecraft).
    /// `%d`/`%f`-спецификаторы конвертируются в `%s` (UNSUPPORTED_FORMAT_PATTERN).
    pub fn load_from_json(
        stream: impl Read,
        output: &mut dyn FnMut(&str, &str),
    ) -> Result<(), String> {
        let mut buf = String::new();
        let mut reader = stream;
        reader.read_to_string(&mut buf).map_err(|e| e.to_string())?;
        for (key, value) in parse_flat_json_strings(&buf)? {
            let converted = convert_unsupported_formats(&value);
            output(&key, &converted);
        }
        Ok(())
    }
}

/// `%(\d+\$)?[\d.]*[df]` → `%$1s`.
fn convert_unsupported_formats(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let mut out = String::with_capacity(value.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' {
            let mut j = i + 1;
            let mut arg_index = String::new();
            let start = j;
            while j < chars.len() && chars[j].is_ascii_digit() {
                j += 1;
            }
            if j > start && j < chars.len() && chars[j] == '$' {
                arg_index = chars[start..j].iter().collect();
                j += 1;
            }
            while j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '.') {
                j += 1;
            }
            if j < chars.len() && (chars[j] == 'd' || chars[j] == 'f') {
                out.push('%');
                if !arg_index.is_empty() {
                    out.push_str(&arg_index);
                    out.push('$');
                }
                out.push('s');
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Минимальный парсер flat JSON-объекта строк (без внешних зависимостей).
fn parse_flat_json_strings(input: &str) -> Result<Vec<(String, String)>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    skip_ws(&chars, &mut i);
    if i >= chars.len() || chars[i] != '{' {
        return Err("Expected '{' at start of JSON object".into());
    }
    i += 1;
    let mut out = Vec::new();
    loop {
        skip_ws(&chars, &mut i);
        if i >= chars.len() {
            return Err("Unterminated JSON object".into());
        }
        if chars[i] == '}' {
            i += 1;
            break;
        }
        let key = parse_json_string(&chars, &mut i)?;
        skip_ws(&chars, &mut i);
        if i >= chars.len() || chars[i] != ':' {
            return Err("Expected ':' after key".into());
        }
        i += 1;
        skip_ws(&chars, &mut i);
        let value = parse_json_string(&chars, &mut i)?;
        out.push((key, value));
        skip_ws(&chars, &mut i);
        if i < chars.len() && chars[i] == ',' {
            i += 1;
        } else if i < chars.len() && chars[i] == '}' {
            i += 1;
            break;
        } else {
            return Err("Expected ',' or '}'".into());
        }
    }
    Ok(out)
}

fn skip_ws(chars: &[char], i: &mut usize) {
    while *i < chars.len() && chars[*i].is_whitespace() {
        *i += 1;
    }
}

fn parse_json_string(chars: &[char], i: &mut usize) -> Result<String, String> {
    if *i >= chars.len() || chars[*i] != '"' {
        return Err("Expected string".into());
    }
    *i += 1;
    let mut out = String::new();
    while *i < chars.len() {
        let c = chars[*i];
        *i += 1;
        match c {
            '"' => return Ok(out),
            '\\' => {
                if *i >= chars.len() {
                    return Err("Unterminated escape".into());
                }
                let e = chars[*i];
                *i += 1;
                match e {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    'b' => out.push('\u{8}'),
                    'f' => out.push('\u{c}'),
                    'u' => {
                        if *i + 4 > chars.len() {
                            return Err("Bad unicode escape".into());
                        }
                        let hex: String = chars[*i..*i + 4].iter().collect();
                        *i += 4;
                        let cp = u32::from_str_radix(&hex, 16).map_err(|_| "Bad unicode escape")?;
                        out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                    }
                    other => out.push(other),
                }
            }
            other => out.push(other),
        }
    }
    Err("Unterminated string".into())
}

static INSTANCE: RwLock<Option<Arc<Language>>> = RwLock::new(None);

impl Language {
    /// `getInstance()`.
    pub fn get_instance() -> Arc<Language> {
        let guard = INSTANCE.read().unwrap();
        if let Some(lang) = guard.as_ref() {
            return lang.clone();
        }
        drop(guard);
        let mut guard = INSTANCE.write().unwrap();
        if let Some(lang) = guard.as_ref() {
            return lang.clone();
        }
        let lang = Arc::new(Language::empty());
        *guard = Some(lang.clone());
        lang
    }

    /// `inject(Language)`.
    pub fn inject(language: Language) {
        let mut guard = INSTANCE.write().unwrap();
        *guard = Some(Arc::new(language));
        next_generation();
    }
}
