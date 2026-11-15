//! Перенос `StringTagVisitor` + `StringTag.quoteAndEscape/escapeWithoutQuotes`
//! + `SnbtGrammar.escapeControlCharacters` — SNBT-печать тегов (`toString`).
//!
//! Парсер SNBT (`TagParser`/`SnbtGrammar`) требует brigadier + packrat +
//! DynamicOps — будет портирован отдельно; здесь только вывод.

use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use std::fmt::Write as _;

const DOUBLE_QUOTE: char = '"';
const SINGLE_QUOTE: char = '\'';
const BACKSLASH: char = '\\';

/// `SnbtGrammar.escapeControlCharacters` — escape-имя для символа
/// или None, если escape не нужен.
pub fn escape_control_characters(c: char) -> Option<String> {
    Some(match c {
        '0' => "0".into(),
        '\u{7}' => "a".into(),
        '\u{8}' => "b".into(),
        '\t' => "t".into(),
        '\n' => "n".into(),
        '\u{b}' => "v".into(),
        '\u{c}' => "f".into(),
        '\r' => "r".into(),
        '"' => "\"".into(),
        '\'' => "'".into(),
        '\\' => "\\".into(),
        c if (c as u32) < 0x20 => format!("x{:02X}", c as u32),
        _ => return None,
    })
}

/// Паттерн `UNQUOTED_KEY_MATCH`: `[A-Za-z._]+[A-Za-z0-9._+-]*`
fn is_unquoted_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '.' || c == '_' => {}
        _ => return false,
    }
    // первый символ подходит под [A-Za-z._]; остальные — под [A-Za-z0-9._+-]
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-'))
}

/// `StringTag.quoteAndEscape` — возвращает строку с кавычками.
pub fn quote_and_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    quote_and_escape_into(text, &mut out);
    out
}

/// `StringTag.quoteAndEscape(text, builder)` — пишет в String.
pub fn quote_and_escape_into(text: &str, out: &mut String) {
    let len = out.len();
    out.push(' '); // placeholder для открывающей кавычки
    let mut quote = '\0';
    for c in text.chars() {
        if c == BACKSLASH {
            out.push(BACKSLASH);
            out.push(BACKSLASH);
        } else if c != DOUBLE_QUOTE && c != SINGLE_QUOTE {
            if let Some(esc) = escape_control_characters(c) {
                out.push(BACKSLASH);
                out.push_str(&esc);
            } else {
                out.push(c);
            }
        } else {
            if quote == '\0' {
                quote = if c == DOUBLE_QUOTE { SINGLE_QUOTE } else { DOUBLE_QUOTE };
            }
            if quote == c {
                out.push(BACKSLASH);
            }
            out.push(c);
        }
    }
    if quote == '\0' {
        quote = DOUBLE_QUOTE;
    }
    out.replace_range(len..=len, &quote.to_string());
    out.push(quote);
}

/// `StringTag.escapeWithoutQuotes`.
pub fn escape_without_quotes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '"' | '\'' | '\\' => {
                out.push(BACKSLASH);
                out.push(c);
            }
            _ => {
                if let Some(esc) = escape_control_characters(c) {
                    out.push(BACKSLASH);
                    out.push_str(&esc);
                } else {
                    out.push(c);
                }
            }
        }
    }
    out
}

// ---------- Java-стиль форматирования чисел ----------
// Java Double.toString/Float.toString отличается от Rust: всегда есть
// десятичная точка ("1.0"), научная нотация для |x|>=1e7 или <1e-3.

fn format_java_float(v: f32) -> String {
    if v.is_nan() {
        return "NaN".into();
    }
    if v.is_infinite() {
        return if v > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    let av = v.abs();
    if av != 0.0 && (av >= 1e7 || av < 1e-3) {
        java_scientific(&format!("{:e}", v))
    } else {
        java_plain(&format!("{:.10}", v))
    }
}

fn format_java_double(v: f64) -> String {
    if v.is_nan() {
        return "NaN".into();
    }
    if v.is_infinite() {
        return if v > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    let av = v.abs();
    if av != 0.0 && (av >= 1e7 || av < 1e-3) {
        java_scientific(&format!("{:e}", v))
    } else {
        java_plain(&format!("{:.20}", v))
    }
}

/// Убирает хвостовые нули, гарантирует наличие десятичной точки.
fn java_plain(s: &str) -> String {
    let mut s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    if !s.contains(['.', 'e', 'E', 'N', 'I']) {
        s.push_str(".0");
    }
    s
}

/// Rust scientific ("1.5e7", "1e-3") → Java-стиль ("1.5E7", "1.0E-3").
/// Java не ставит '+' и не дополняет показатель нулями.
fn java_scientific(s: &str) -> String {
    let (mant, exp) = s.split_once('e').unwrap_or((s, "0"));
    let mant = java_plain(mant);
    let exp: i32 = exp.parse().unwrap_or(0);
    format!("{mant}E{exp}")
}

// ---------- SNBT-вывод ----------

/// `StringTagVisitor` — SNBT-представление тега.
pub fn tag_to_string(tag: &Tag, out: &mut String) {
    match tag {
        Tag::End => out.push_str("END"),
        Tag::Byte(v) => {
            let _ = write!(out, "{v}b");
        }
        Tag::Short(v) => {
            let _ = write!(out, "{v}s");
        }
        Tag::Int(v) => {
            let _ = write!(out, "{v}");
        }
        Tag::Long(v) => {
            let _ = write!(out, "{v}L");
        }
        Tag::Float(v) => {
            out.push_str(&format_java_float(*v));
            out.push('f');
        }
        Tag::Double(v) => {
            out.push_str(&format_java_double(*v));
            out.push('d');
        }
        Tag::ByteArray(a) => {
            out.push_str("[B;");
            for (i, v) in a.iter().enumerate() {
                if i != 0 {
                    out.push(',');
                }
                let _ = write!(out, "{v}B");
            }
            out.push(']');
        }
        Tag::String(s) => {
            out.push_str(&quote_and_escape(s));
        }
        Tag::List(list) => {
            out.push('[');
            for (i, t) in list.iter().enumerate() {
                if i != 0 {
                    out.push(',');
                }
                tag_to_string(t, out);
            }
            out.push(']');
        }
        Tag::Compound(c) => {
            out.push('{');
            // StringTagVisitor сортирует ключи
            let mut entries: Vec<_> = c.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            for (i, (key, t)) in entries.iter().enumerate() {
                if i != 0 {
                    out.push(',');
                }
                write_key(key, out);
                out.push(':');
                tag_to_string(t, out);
            }
            out.push('}');
        }
        Tag::IntArray(a) => {
            out.push_str("[I;");
            for (i, v) in a.iter().enumerate() {
                if i != 0 {
                    out.push(',');
                }
                let _ = write!(out, "{v}");
            }
            out.push(']');
        }
        Tag::LongArray(a) => {
            out.push_str("[L;");
            for (i, v) in a.iter().enumerate() {
                if i != 0 {
                    out.push(',');
                }
                let _ = write!(out, "{v}L");
            }
            out.push(']');
        }
    }
}

/// `handleKeyEscape` — ключ без кавычек если матчится паттерну и не true/false.
fn write_key(key: &str, out: &mut String) {
    if !key.eq_ignore_ascii_case("true")
        && !key.eq_ignore_ascii_case("false")
        && is_unquoted_key(key)
    {
        out.push_str(key);
    } else {
        quote_and_escape_into(key, out);
    }
}

/// `Tag.toString()` → SNBT.
pub fn to_string(tag: &Tag) -> String {
    let mut out = String::new();
    tag_to_string(tag, &mut out);
    out
}

/// `CompoundTag.toString()`.
pub fn compound_to_string(c: &CompoundTag) -> String {
    to_string(&Tag::Compound(c.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nbt::io::{read_bytes_unlimited, write};

    #[test]
    fn snbt_output() {
        let mut c = CompoundTag::new();
        c.put("name", Tag::String("hello world".into()));
        c.put("count", Tag::Int(42));
        c.put("b", Tag::Byte(1));
        c.put("l", Tag::Long(5));
        c.put("f", Tag::Float(1.5));
        c.put("d", Tag::Double(2.0));
        c.put("list", Tag::List(vec![Tag::Int(1), Tag::Int(2)]));
        c.put("true", Tag::Int(1));
        let s = compound_to_string(&c);
        assert!(s.starts_with('{') && s.ends_with('}'));
        assert!(s.contains("count:42"));
        assert!(s.contains("name:\"hello world\""));
        assert!(s.contains("b:1b"));
        assert!(s.contains("l:5L"));
        assert!(s.contains("f:1.5f"));
        assert!(s.contains("d:2.0d"));
        assert!(s.contains("\"true\":1")); // true — зарезервировано, ключ в кавычках
    }

    #[test]
    fn escapes() {
        assert_eq!(escape_control_characters('\n'), Some("n".into()));
        assert_eq!(escape_control_characters('\u{1}'), Some("x01".into()));
        assert_eq!(escape_control_characters('x'), None);
        assert_eq!(escape_without_quotes("a\nb"), "a\\nb");
        assert_eq!(escape_without_quotes("a\"b"), "a\\\"b");

        // Java выбирает "противоположную" кавычку: внутри " не надо escape
        let q = quote_and_escape("say \"hi\"");
        assert_eq!(q, "'say \"hi\"'");
        // escape ставится на кавычку, совпадающую с выбранным разделителем
        let q2 = quote_and_escape("a\"b'c");
        assert_eq!(q2, "'a\"b\\'c'");
        // roundtrip через экранирование
        let q2 = quote_and_escape("plain");
        assert_eq!(q2, "\"plain\"");
    }

    #[test]
    fn snbt_binary_roundtrip() {
        let mut c = CompoundTag::new();
        c.put("k", Tag::String("v".into()));
        let bytes = write(&c).unwrap();
        let back = read_bytes_unlimited(&bytes).unwrap();
        assert_eq!(compound_to_string(&back), compound_to_string(&c));
    }

    #[test]
    fn unquoted_key_rules() {
        assert!(is_unquoted_key("abc"));
        assert!(is_unquoted_key("a.b_c+d-e"));
        assert!(!is_unquoted_key("1abc")); // начинается с цифры
        assert!(!is_unquoted_key("a b")); // пробел
        assert!(!is_unquoted_key(""));
    }
}
