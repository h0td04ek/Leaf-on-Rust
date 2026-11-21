//! SNBT-парсер — аналог `net.minecraft.nbt.TagParser` + `SnbtGrammar`.
//!
//! Поддерживает: `{k:v,...}`, `[v,...]`, `[B;..]`/`[I;..]`/`[L;..]`,
//! строки ("..", '..', unquoted), числа с суффиксами b/s/l/f/d,
//! `true`/`false`, разделители `,`/`:`/пробелы.

use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use std::fmt;

/// Ошибка парсинга SNBT.
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub cursor: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at offset {}", self.message, self.cursor)
    }
}
impl std::error::Error for ParseError {}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input: input.as_bytes(), pos: 0, depth: 0 }
    }

    fn error<T>(&self, msg: impl Into<String>) -> Result<T, ParseError> {
        Err(ParseError { message: msg.into(), cursor: self.pos })
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn peek_at(&self, off: usize) -> Option<u8> {
        self.input.get(self.pos + off).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, b: u8, what: &str) -> Result<(), ParseError> {
        self.skip_ws();
        if self.peek() == Some(b) {
            self.pos += 1;
            Ok(())
        } else {
            self.error(format!("expected '{}', found {:?}", b as char, self.peek().map(|c| c as char)))
                .map(|_: ()| ())
                .and_then(|_| unreachable!())
        }
    }

    // ------------------------------------------------------------ values

    fn parse_value(&mut self) -> Result<Tag, ParseError> {
        if self.depth >= Tag::MAX_DEPTH {
            return self.error("SNBT is too deeply nested");
        }
        self.depth += 1;
        let result = self.parse_value_at_depth();
        self.depth -= 1;
        result
    }

    fn parse_value_at_depth(&mut self) -> Result<Tag, ParseError> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_compound(),
            Some(b'[') => self.parse_list_or_array(),
            Some(b'"') | Some(b'\'') => Ok(Tag::String(self.parse_quoted()?)),
            Some(_) => self.parse_atom(),
            None => self.error("unexpected end of input"),
        }
    }

    fn parse_compound(&mut self) -> Result<Tag, ParseError> {
        self.expect(b'{', "{")?;
        let mut compound = CompoundTag::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Tag::Compound(compound));
        }
        loop {
            self.skip_ws();
            let key = self.parse_key()?;
            self.expect(b':', ":")?;
            let value = self.parse_value()?;
            compound.put(key, value);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Tag::Compound(compound));
                }
                other => {
                    return self.error(format!(
                        "expected ',' or '}}', found {:?}",
                        other.map(|c| c as char)
                    ))
                }
            }
        }
    }

    fn parse_key(&mut self) -> Result<String, ParseError> {
        self.skip_ws();
        match self.peek() {
            Some(b'"') | Some(b'\'') => self.parse_quoted(),
            Some(_) => self.parse_unquoted(),
            None => self.error("expected key"),
        }
    }

    fn parse_quoted(&mut self) -> Result<String, ParseError> {
        let quote = self.bump().unwrap();
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return self.error("unterminated string"),
                Some(b) if b == quote => return Ok(out),
                Some(b'\\') => match self.bump() {
                    None => return self.error("unterminated escape"),
                    Some(b'n') => out.push('\n'),
                    Some(b't') => out.push('\t'),
                    Some(b'r') => out.push('\r'),
                    Some(b'b') => out.push('\u{8}'),
                    Some(b'f') => out.push('\u{c}'),
                    Some(b'\\') => out.push('\\'),
                    Some(b'"') => out.push('"'),
                    Some(b'\'') => out.push('\''),
                    Some(b'u') => {
                        let hex = self
                            .input
                            .get(self.pos..self.pos + 4)
                            .and_then(|s| std::str::from_utf8(s).ok())
                            .ok_or_else(|| ParseError {
                                message: "bad unicode escape".into(),
                                cursor: self.pos,
                            })?;
                        let code = u32::from_str_radix(hex, 16).map_err(|_| ParseError {
                            message: "bad unicode escape".into(),
                            cursor: self.pos,
                        })?;
                        self.pos += 4;
                        out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                    }
                    Some(other) => out.push(other as char),
                },
                Some(b) => {
                    // UTF-8: собираем байты до следующего ASCII-спецсимвола
                    if b < 0x80 {
                        out.push(b as char);
                    } else {
                        // многобайтовый UTF-8 — откат и посимвольное чтение
                        self.pos -= 1;
                        let rest = std::str::from_utf8(&self.input[self.pos..])
                            .map_err(|_| ParseError {
                                message: "invalid utf-8".into(),
                                cursor: self.pos,
                            })?;
                        let ch = rest.chars().next().unwrap();
                        out.push(ch);
                        self.pos += ch.len_utf8();
                    }
                }
            }
        }
    }

    /// Unquoted-строка: до `,`/`}`/`]`/`:`/пробела.
    fn parse_unquoted(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if matches!(b, b',' | b'}' | b']' | b':' | b' ' | b'\t' | b'\n' | b'\r') {
                break;
            }
            self.pos += 1;
        }
        if self.pos == start {
            return self.error("expected unquoted string");
        }
        std::str::from_utf8(&self.input[start..self.pos])
            .map(|s| s.to_string())
            .map_err(|_| ParseError { message: "invalid utf-8".into(), cursor: start })
    }

    fn parse_list_or_array(&mut self) -> Result<Tag, ParseError> {
        self.expect(b'[', "[")?;
        self.skip_ws();
        // Типизированные массивы: [B;..] [I;..] [L;..]
        if let Some(t) = self.peek() {
            if matches!(t, b'B' | b'I' | b'L') && self.peek_at(1) == Some(b';') {
                let kind = self.bump().unwrap();
                self.pos += 1; // ';'
                return self.parse_typed_array(kind);
            }
        }
        let mut list = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Tag::List(list));
        }
        loop {
            list.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Tag::List(list));
                }
                other => {
                    return self.error(format!(
                        "expected ',' or ']', found {:?}",
                        other.map(|c| c as char)
                    ))
                }
            }
        }
    }

    fn parse_typed_array(&mut self, kind: u8) -> Result<Tag, ParseError> {
        let mut values = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return self.typed_array_result(kind, values);
        }
        loop {
            values.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    return self.typed_array_result(kind, values);
                }
                other => {
                    return self.error(format!(
                        "expected ',' or ']', found {:?}",
                        other.map(|c| c as char)
                    ))
                }
            }
        }
    }

    /// Числовое значение тега (any numeric variant → f64).
    fn numeric(tag: &Tag) -> Option<f64> {
        match tag {
            Tag::Byte(v) => Some(*v as f64),
            Tag::Short(v) => Some(*v as f64),
            Tag::Int(v) => Some(*v as f64),
            Tag::Long(v) => Some(*v as f64),
            Tag::Float(v) => Some(*v as f64),
            Tag::Double(v) => Some(*v),
            _ => None,
        }
    }

    fn typed_array_result(&self, kind: u8, values: Vec<Tag>) -> Result<Tag, ParseError> {
        let nums: Vec<f64> = values
            .iter()
            .map(|t| Self::numeric(t).ok_or_else(|| ParseError {
                message: format!("non-numeric element in typed array: {}", t.type_name()),
                cursor: self.pos,
            }))
            .collect::<Result<_, _>>()?;
        Ok(match kind {
            b'B' => Tag::ByteArray(nums.iter().map(|v| *v as i8).collect()),
            b'I' => Tag::IntArray(nums.iter().map(|v| *v as i32).collect()),
            b'L' => Tag::LongArray(nums.iter().map(|v| *v as i64).collect()),
            _ => unreachable!(),
        })
    }

    /// Атом: число / true / false / unquoted-строка.
    fn parse_atom(&mut self) -> Result<Tag, ParseError> {
        let start = self.pos;
        // Идентификатор-подобные атомы
        while let Some(b) = self.peek() {
            if matches!(b, b',' | b'}' | b']' | b':' | b' ' | b'\t' | b'\n' | b'\r') {
                break;
            }
            self.pos += 1;
        }
        if self.pos == start {
            return self.error("expected value");
        }
        let text = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| ParseError { message: "invalid utf-8".into(), cursor: start })?;
        Ok(parse_atom_text(text))
    }
}

fn parse_atom_text(text: &str) -> Tag {
    // Суффиксы типов
    let lower = text.to_ascii_lowercase();
    if lower == "true" {
        return Tag::Byte(1);
    }
    if lower == "false" {
        return Tag::Byte(0);
    }

    let (num, suffix) = split_suffix(text);
    match suffix {
        Some('b') | Some('B') => {
            if let Ok(v) = num.parse::<i64>() {
                return Tag::Byte(v as i8);
            }
        }
        Some('s') | Some('S') => {
            if let Ok(v) = num.parse::<i64>() {
                return Tag::Short(v as i16);
            }
        }
        Some('l') | Some('L') => {
            if let Ok(v) = num.parse::<i64>() {
                return Tag::Long(v);
            }
        }
        Some('f') | Some('F') => {
            if let Ok(v) = num.parse::<f64>() {
                return Tag::Float(v as f32);
            }
        }
        Some('d') | Some('D') => {
            if let Ok(v) = num.parse::<f64>() {
                return Tag::Double(v);
            }
        }
        _ => {}
    }

    // Числа без суффикса: int → long → double (порядок Java TagParser)
    if let Ok(v) = text.parse::<i32>() {
        return Tag::Int(v);
    }
    if let Ok(v) = text.parse::<i64>() {
        return Tag::Long(v);
    }
    if let Ok(v) = text.parse::<f64>() {
        if text.contains('.') || text.contains('e') || text.contains('E') {
            return Tag::Double(v);
        }
    }
    Tag::String(text.to_string())
}

fn split_suffix(text: &str) -> (&str, Option<char>) {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return (text, None);
    }
    let last = bytes[bytes.len() - 1] as char;
    if matches!(last, 'b' | 'B' | 's' | 'S' | 'l' | 'L' | 'f' | 'F' | 'd' | 'D')
        && bytes.len() > 1
        && (bytes[bytes.len() - 2] as char).is_ascii_digit()
    {
        (&text[..text.len() - 1], Some(last))
    } else {
        (text, None)
    }
}

pub(crate) fn parse_prefix(input: &str) -> Result<(Tag, usize), ParseError> {
    let mut parser = Parser::new(input);
    let tag = parser.parse_value()?;
    Ok((tag, parser.pos))
}

/// `TagParser.parseTag`.
pub fn parse(input: &str) -> Result<Tag, ParseError> {
    let mut parser = Parser::new(input);
    let tag = parser.parse_value()?;
    parser.skip_ws();
    if parser.pos != parser.input.len() {
        return Err(ParseError {
            message: format!(
                "trailing characters at offset {}",
                parser.pos
            ),
            cursor: parser.pos,
        });
    }
    Ok(tag)
}

/// `TagParser.parseCompoundTag`.
pub fn parse_compound(input: &str) -> Result<CompoundTag, ParseError> {
    match parse(input)? {
        Tag::Compound(c) => Ok(c),
        other => Err(ParseError {
            message: format!("expected compound, got {}", other.type_name()),
            cursor: 0,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compound() {
        let c = parse_compound("{name:\"test\",count:5b,flag:true,list:[1,2.5,\"x\"]}").unwrap();
        assert_eq!(c.get_string("name"), Some("test"));
        assert_eq!(c.get_byte("count"), Some(5));
        assert_eq!(c.get_bool("flag"), Some(true));
        let list = c.get_list("list").unwrap();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_roundtrip() {
        let c = parse_compound("{a:1,b:[I;1,2,3],c:{d:\"hi\"}}").unwrap();
        let s = crate::nbt::snbt::compound_to_string(&c);
        let c2 = parse_compound(&s).unwrap();
        assert_eq!(c.to_sorted_map().len(), c2.to_sorted_map().len());
    }

    #[test]
    fn test_typed_arrays() {
        let t = parse("[B;1,2,3]").unwrap();
        assert_eq!(t.as_byte_array(), Some(&[1i8, 2, 3][..]));
        let t = parse("[L;1l,2l]").unwrap();
        assert_eq!(t.as_long_array(), Some(&[1i64, 2][..]));
    }
}
