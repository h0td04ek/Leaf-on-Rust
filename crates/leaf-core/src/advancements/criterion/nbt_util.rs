//! Вспомогательные функции NBT-кодеков (`RecordCodecBuilder`
//! optionalFieldOf/fieldOf → NBT compound).

use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;

/// `optionalFieldOf` — значение поля или None (поле отсутствует).
pub fn opt_field<'a>(c: &'a CompoundTag, name: &str) -> Option<&'a Tag> {
    c.get(name)
}

/// `fieldOf` — обязательное поле.
pub fn field<'a>(c: &'a CompoundTag, name: &str) -> Result<&'a Tag, String> {
    c.get(name).ok_or_else(|| format!("missing field '{name}'"))
}

/// Записать необязательное поле (None → поле не пишется).
pub fn put_opt(c: &mut CompoundTag, name: &str, tag: Option<Tag>) {
    if let Some(t) = tag {
        c.put(name, t);
    }
}

/// Строковое поле → String.
pub fn get_string(tag: &Tag, what: &str) -> Result<String, String> {
    match tag {
        Tag::String(s) => Ok(s.clone()),
        _ => Err(format!("expected string for {what}")),
    }
}

/// Булево поле (`Codec.BOOL` принимает Byte/Boolean? — в NBT bool =
/// Byte 0/1).
pub fn get_bool(tag: &Tag, what: &str) -> Result<bool, String> {
    match tag {
        Tag::Byte(v) => Ok(*v != 0),
        _ => Err(format!("expected bool for {what}")),
    }
}

/// Целочисленное поле.
pub fn get_i64(tag: &Tag, what: &str) -> Result<i64, String> {
    match tag {
        Tag::Byte(v) => Ok(*v as i64),
        Tag::Short(v) => Ok(*v as i64),
        Tag::Int(v) => Ok(*v as i64),
        Tag::Long(v) => Ok(*v),
        _ => Err(format!("expected int for {what}")),
    }
}

/// `Tag::List` элементов.
pub fn get_list<'a>(tag: &'a Tag, what: &str) -> Result<&'a [Tag], String> {
    match tag {
        Tag::List(l) => Ok(l),
        _ => Err(format!("expected list for {what}")),
    }
}

/// Compound → ссылка.
pub fn as_compound<'a>(tag: &'a Tag, what: &str) -> Result<&'a CompoundTag, String> {
    match tag {
        Tag::Compound(c) => Ok(c),
        _ => Err(format!("expected compound for {what}")),
    }
}

/// `compactListCodec`/`listOf` decode: одиночный элемент или список.
pub fn decode_compact_list<T>(
    tag: &Tag,
    decode_one: impl Fn(&Tag) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    match tag {
        Tag::List(l) => l.iter().map(decode_one).collect(),
        other => Ok(vec![decode_one(other)?]),
    }
}

/// Encode списка как compactList: 0 → пустой список, 1 → одиночный
/// элемент, >1 → список. Java `ExtraCodecs.compactListCodec` пишет
/// пустой список как `[]`, один элемент — без списка.
/// ВНИМАНИЕ: `Codec.listOf()` всегда пишет список; compact — нет.
/// Здесь используется компактная форма там, где Java использует
/// `compactListCodec`; для `listOf` передавать `always_list=true`.
pub fn encode_compact_list<T>(
    items: &[T],
    encode_one: impl Fn(&T) -> Tag,
) -> Tag {
    match items.len() {
        0 => Tag::List(Vec::new()),
        1 => encode_one(&items[0]),
        _ => Tag::List(items.iter().map(encode_one).collect()),
    }
}

/// `Codec.listOf` encode — всегда список.
pub fn encode_list<T>(items: &[T], encode_one: impl Fn(&T) -> Tag) -> Tag {
    Tag::List(items.iter().map(encode_one).collect())
}
