//! Основные типы аргументов Brigadier и их сетевые шаблоны.

use crate::network::friendly_byte_buf::FriendlyByteBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringMode {
    Word,
    Quotable,
    Greedy,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArgumentType {
    Bool,
    Float { min: f32, max: f32 },
    Double { min: f64, max: f64 },
    Integer { min: i32, max: i32 },
    Long { min: i64, max: i64 },
    String(StringMode),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArgumentValue {
    Bool(bool),
    Float(f32),
    Double(f64),
    Integer(i32),
    Long(i64),
    String(String),
}

impl ArgumentType {
    pub fn registry_id(self) -> i32 {
        match self {
            Self::Bool => 0,
            Self::Float { .. } => 1,
            Self::Double { .. } => 2,
            Self::Integer { .. } => 3,
            Self::Long { .. } => 4,
            Self::String(_) => 5,
        }
    }

    pub(crate) fn parse(self, input: &str, cursor: usize) -> Option<(ArgumentValue, usize)> {
        let reader = &input[cursor..];
        if reader.is_empty() {
            return None;
        }
        if matches!(self, Self::Bool | Self::String(StringMode::Quotable))
            && reader.starts_with(['"', '\''])
        {
            let (value, end) = read_quoted_string(reader)?;
            let value = match self {
                Self::Bool => ArgumentValue::Bool(parse_bool(&value)?),
                _ => ArgumentValue::String(value),
            };
            return Some((value, cursor + end));
        }
        let end = match self {
            Self::Bool | Self::String(StringMode::Word | StringMode::Quotable) => reader
                .bytes()
                .take_while(|&b| is_unquoted_string(b))
                .count(),
            Self::String(StringMode::Greedy) => reader.len(),
            _ => reader
                .bytes()
                .take_while(|b| b.is_ascii_digit() || matches!(b, b'.' | b'-'))
                .count(),
        };
        let token = &reader[..end];
        let value = match self {
            Self::Bool => ArgumentValue::Bool(parse_bool(token)?),
            Self::Integer { min, max } => ArgumentValue::Integer(parse_bounded(token, min, max)?),
            Self::Long { min, max } => ArgumentValue::Long(parse_bounded(token, min, max)?),
            Self::Float { min, max } => ArgumentValue::Float(parse_bounded(token, min, max)?),
            Self::Double { min, max } => ArgumentValue::Double(parse_bounded(token, min, max)?),
            Self::String(_) if !token.is_empty() => ArgumentValue::String(token.to_owned()),
            Self::String(_) => return None,
        };
        Some((value, cursor + end))
    }

    pub(crate) fn read_template(id: i32, buf: &mut FriendlyByteBuf) -> Option<Self> {
        match id {
            0 => Some(Self::Bool),
            1 => {
                let (min, max) = read_bounds(buf, f32::MIN, f32::MAX, FriendlyByteBuf::read_float);
                Some(Self::Float { min, max })
            }
            2 => {
                let (min, max) = read_bounds(buf, f64::MIN, f64::MAX, FriendlyByteBuf::read_double);
                Some(Self::Double { min, max })
            }
            3 => {
                let (min, max) = read_bounds(buf, i32::MIN, i32::MAX, FriendlyByteBuf::read_int);
                Some(Self::Integer { min, max })
            }
            4 => {
                let (min, max) = read_bounds(buf, i64::MIN, i64::MAX, FriendlyByteBuf::read_long);
                Some(Self::Long { min, max })
            }
            5 => match buf.read_var_int() {
                0 => Some(Self::String(StringMode::Word)),
                1 => Some(Self::String(StringMode::Quotable)),
                2 => Some(Self::String(StringMode::Greedy)),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn write_template(self, buf: &mut FriendlyByteBuf) {
        match self {
            Self::Bool => {}
            Self::Float { min, max } => {
                write_bounds(buf, min, max, f32::MIN, f32::MAX, |buf, value| {
                    buf.write_float(value);
                })
            }
            Self::Double { min, max } => {
                write_bounds(buf, min, max, f64::MIN, f64::MAX, |buf, value| {
                    buf.write_double(value);
                })
            }
            Self::Integer { min, max } => {
                write_bounds(buf, min, max, i32::MIN, i32::MAX, |buf, value| {
                    buf.write_int(value);
                })
            }
            Self::Long { min, max } => {
                write_bounds(buf, min, max, i64::MIN, i64::MAX, |buf, value| {
                    buf.write_long(value);
                })
            }
            Self::String(mode) => {
                buf.write_var_int(match mode {
                    StringMode::Word => 0,
                    StringMode::Quotable => 1,
                    StringMode::Greedy => 2,
                });
            }
        }
    }
}

fn parse_bool(token: &str) -> Option<bool> {
    match token {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_bounded<T: std::str::FromStr + PartialOrd>(input: &str, min: T, max: T) -> Option<T> {
    let value = input.parse::<T>().ok()?;
    (value >= min && value <= max).then_some(value)
}

fn is_unquoted_string(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'+')
}

fn read_quoted_string(input: &str) -> Option<(String, usize)> {
    let mut chars = input.char_indices();
    let (_, quote) = chars.next()?;
    let mut value = String::new();
    while let Some((index, c)) = chars.next() {
        if c == quote {
            return Some((value, index + c.len_utf8()));
        }
        if c != '\\' {
            value.push(c);
            continue;
        }
        let (_, escaped) = chars.next()?;
        if escaped != quote && escaped != '\\' {
            return None;
        }
        value.push(escaped);
    }
    None
}

fn read_bounds<T>(
    buf: &mut FriendlyByteBuf,
    default_min: T,
    default_max: T,
    read: fn(&mut FriendlyByteBuf) -> T,
) -> (T, T) {
    let flags = buf.read_unsigned_byte();
    let min = if flags & 1 != 0 {
        read(buf)
    } else {
        default_min
    };
    let max = if flags & 2 != 0 {
        read(buf)
    } else {
        default_max
    };
    (min, max)
}

fn write_bounds<T: PartialEq>(
    buf: &mut FriendlyByteBuf,
    min: T,
    max: T,
    default_min: T,
    default_max: T,
    write: impl Fn(&mut FriendlyByteBuf, T),
) {
    let has_min = min != default_min;
    let has_max = max != default_max;
    buf.write_byte(i32::from(has_min) | i32::from(has_max) << 1);
    if has_min {
        write(buf, min);
    }
    if has_max {
        write(buf, max);
    }
}
