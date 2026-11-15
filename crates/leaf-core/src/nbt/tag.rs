//! Перенос `net.minecraft.nbt.Tag` и всех типов тегов — как единый enum
//! (в Java это sealed interface + record-классы; в Rust enum идиоматичнее
//! и покрывает тот же набор вариантов).

use crate::nbt::compound::CompoundTag;

pub const TAG_END: u8 = 0;
pub const TAG_BYTE: u8 = 1;
pub const TAG_SHORT: u8 = 2;
pub const TAG_INT: u8 = 3;
pub const TAG_LONG: u8 = 4;
pub const TAG_FLOAT: u8 = 5;
pub const TAG_DOUBLE: u8 = 6;
pub const TAG_BYTE_ARRAY: u8 = 7;
pub const TAG_STRING: u8 = 8;
pub const TAG_LIST: u8 = 9;
pub const TAG_COMPOUND: u8 = 10;
pub const TAG_INT_ARRAY: u8 = 11;
pub const TAG_LONG_ARRAY: u8 = 12;

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<Tag>),
    Compound(CompoundTag),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    pub const MAX_DEPTH: usize = 512;

    /// `getId()`
    pub fn id(&self) -> u8 {
        match self {
            Tag::End => TAG_END,
            Tag::Byte(_) => TAG_BYTE,
            Tag::Short(_) => TAG_SHORT,
            Tag::Int(_) => TAG_INT,
            Tag::Long(_) => TAG_LONG,
            Tag::Float(_) => TAG_FLOAT,
            Tag::Double(_) => TAG_DOUBLE,
            Tag::ByteArray(_) => TAG_BYTE_ARRAY,
            Tag::String(_) => TAG_STRING,
            Tag::List(_) => TAG_LIST,
            Tag::Compound(_) => TAG_COMPOUND,
            Tag::IntArray(_) => TAG_INT_ARRAY,
            Tag::LongArray(_) => TAG_LONG_ARRAY,
        }
    }

    /// `getType().getName()` — имя типа капсом.
    pub fn type_name(&self) -> &'static str {
        match self {
            Tag::End => "END",
            Tag::Byte(_) => "BYTE",
            Tag::Short(_) => "SHORT",
            Tag::Int(_) => "INT",
            Tag::Long(_) => "LONG",
            Tag::Float(_) => "FLOAT",
            Tag::Double(_) => "DOUBLE",
            Tag::ByteArray(_) => "BYTE_ARRAY",
            Tag::String(_) => "STRING",
            Tag::List(_) => "LIST",
            Tag::Compound(_) => "COMPOUND",
            Tag::IntArray(_) => "INT_ARRAY",
            Tag::LongArray(_) => "LONG_ARRAY",
        }
    }

    /// `sizeInBytes()` — оценка размера для accounter'а (как в оригинале).
    pub fn size_in_bytes(&self) -> i64 {
        match self {
            Tag::End => 8,
            Tag::Byte(_) => 9,
            Tag::Short(_) => 10,
            Tag::Int(_) | Tag::Float(_) => 12,
            Tag::Long(_) | Tag::Double(_) => 16,
            Tag::ByteArray(a) => 24 + a.len() as i64,
            Tag::String(s) => 36 + s.len() as i64 * 2,
            Tag::List(l) => {
                36 + l.iter().map(|t| t.size_in_bytes()).sum::<i64>() + 4 * l.len() as i64
            }
            Tag::Compound(c) => {
                let mut total = 48i64;
                for (k, v) in c.iter() {
                    total += 36 + k.len() as i64 * 2 + v.size_in_bytes();
                }
                total
            }
            Tag::IntArray(a) => 24 + 4 * a.len() as i64,
            Tag::LongArray(a) => 24 + 8 * a.len() as i64,
        }
    }

    // ---- as*()-аксессоры (Optional<T> в Java → Option<T> здесь) ----

    pub fn as_byte(&self) -> Option<i8> {
        match self {
            Tag::Byte(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_short(&self) -> Option<i16> {
        match self {
            Tag::Byte(v) => Some(*v as i16),
            Tag::Short(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            Tag::Byte(v) => Some(*v as i32),
            Tag::Short(v) => Some(*v as i32),
            Tag::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_long(&self) -> Option<i64> {
        match self {
            Tag::Byte(v) => Some(*v as i64),
            Tag::Short(v) => Some(*v as i64),
            Tag::Int(v) => Some(*v as i64),
            Tag::Long(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            Tag::Byte(v) => Some(*v as f32),
            Tag::Short(v) => Some(*v as f32),
            Tag::Int(v) => Some(*v as f32),
            Tag::Long(v) => Some(*v as f32),
            Tag::Float(v) => Some(*v),
            Tag::Double(v) => Some(*v as f32),
            _ => None,
        }
    }

    pub fn as_double(&self) -> Option<f64> {
        match self {
            Tag::Byte(v) => Some(*v as f64),
            Tag::Short(v) => Some(*v as f64),
            Tag::Int(v) => Some(*v as f64),
            Tag::Long(v) => Some(*v as f64),
            Tag::Float(v) => Some(*v as f64),
            Tag::Double(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        self.as_byte().map(|b| b != 0)
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Tag::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_byte_array(&self) -> Option<&[i8]> {
        match self {
            Tag::ByteArray(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_int_array(&self) -> Option<&[i32]> {
        match self {
            Tag::IntArray(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_long_array(&self) -> Option<&[i64]> {
        match self {
            Tag::LongArray(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_compound(&self) -> Option<&CompoundTag> {
        match self {
            Tag::Compound(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Tag]> {
        match self {
            Tag::List(l) => Some(l),
            _ => None,
        }
    }
}
