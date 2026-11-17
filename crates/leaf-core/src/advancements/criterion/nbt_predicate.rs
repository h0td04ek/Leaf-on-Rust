//! Перенос `net.minecraft.advancements.criterion.NbtPredicate`.

use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;

/// `NbtPredicate` — сравнение NBT (`NbtUtils.compareNbt(tag, other,
/// true)` — нестрогое сравнение: другие числовые типы равны по
/// значению, отсутствующие ключи в `other` игнорируются? —
/// `compareNbt(a, b, true)` проверяет, что `a` содержится в `b`).
#[derive(Clone, Debug, PartialEq)]
pub struct NbtPredicate {
    /// `tag`.
    pub tag: CompoundTag,
}

impl NbtPredicate {
    /// `new`.
    pub fn new(tag: CompoundTag) -> Self {
        Self { tag }
    }

    /// `NbtPredicate.CODEC` → NBT: значение — сам compound-тег.
    pub fn encode_nbt(&self) -> Tag {
        Tag::Compound(self.tag.clone())
    }

    /// `NbtPredicate.CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        match tag {
            Tag::Compound(c) => Ok(Self::new(c.clone())),
            _ => Err("expected compound for NbtPredicate".into()),
        }
    }

    /// `matches(Tag?)` — `NbtUtils.compareNbt(this.tag, tag, true)`:
    /// `tag` (фактический) должен содержать все ключи/значения
    /// предиката; числа сравниваются по значению независимо от типа.
    pub fn matches(&self, other: Option<&Tag>) -> bool {
        match other {
            Some(Tag::Compound(c)) => compare_compound(&self.tag, c, true),
            _ => false,
        }
    }

    /// `matches(DataComponentGetter)` — CustomData компонент.
    pub fn matches_custom_data(&self, custom_data: Option<&CompoundTag>) -> bool {
        match custom_data {
            Some(c) => compare_compound(&self.tag, c, true),
            None => self.tag.is_empty(),
        }
    }
}

/// `NbtUtils.compareNbt(a, b, strict)` — `a` ⊆ `b`.
/// `strict=true`: типы должны совпадать точно? В оригинале
/// `compareNbt(tag, other, true)` — числа разных типов НЕ равны
/// (strict), но контейнеры сравниваются по содержимому.
pub fn compare_nbt(a: &Tag, b: &Tag, strict: bool) -> bool {
    match (a, b) {
        (Tag::End, Tag::End) => true,
        (Tag::Byte(x), Tag::Byte(y)) => x == y,
        (Tag::Short(x), Tag::Short(y)) => x == y,
        (Tag::Int(x), Tag::Int(y)) => x == y,
        (Tag::Long(x), Tag::Long(y)) => x == y,
        (Tag::Float(x), Tag::Float(y)) => x == y,
        (Tag::Double(x), Tag::Double(y)) => x == y,
        (Tag::ByteArray(x), Tag::ByteArray(y)) => x == y,
        (Tag::String(x), Tag::String(y)) => x == y,
        (Tag::IntArray(x), Tag::IntArray(y)) => x == y,
        (Tag::LongArray(x), Tag::LongArray(y)) => x == y,
        (Tag::List(x), Tag::List(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| compare_nbt(a, b, strict))
        }
        (Tag::Compound(x), Tag::Compound(y)) => compare_compound(x, y, strict),
        // Числа разных типов: strict → не равны.
        _ if !strict && numeric_equal(a, b) => true,
        _ => false,
    }
}

fn numeric_equal(a: &Tag, b: &Tag) -> bool {
    match (tag_as_f64(a), tag_as_f64(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

fn tag_as_f64(t: &Tag) -> Option<f64> {
    match t {
        Tag::Byte(v) => Some(*v as f64),
        Tag::Short(v) => Some(*v as f64),
        Tag::Int(v) => Some(*v as f64),
        Tag::Long(v) => Some(*v as f64),
        Tag::Float(v) => Some(*v as f64),
        Tag::Double(v) => Some(*v),
        _ => None,
    }
}

/// `NbtUtils.compareNbt(CompoundTag, CompoundTag, strict)` —
/// каждый ключ `a` присутствует в `b` с равным значением.
pub fn compare_compound(a: &CompoundTag, b: &CompoundTag, strict: bool) -> bool {
    a.iter().all(|(k, v)| match b.get(k) {
        Some(other) => compare_nbt(v, other, strict),
        None => false,
    })
}
