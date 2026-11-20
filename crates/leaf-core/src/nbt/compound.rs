//! Перенос `net.minecraft.nbt.CompoundTag` — упорядоченный map String → Tag.
//!
//! Порядок вставки сохраняется (как LinkedHashMap в оригинале),
//! дубликаты перезаписываются.

use crate::nbt::tag::Tag;
use indexmap::IndexMap;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompoundTag {
    map: IndexMap<String, Tag>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self { map: IndexMap::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { map: IndexMap::with_capacity(capacity) }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&Tag> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Tag> {
        self.map.get_mut(key)
    }

    /// `put` — возвращает заменённый тег, если ключ уже был.
    pub fn put(&mut self, key: impl Into<String>, tag: Tag) -> Option<Tag> {
        self.map.insert(key.into(), tag)
    }

    pub fn remove(&mut self, key: &str) -> Option<Tag> {
        self.map.shift_remove(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Tag)> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Tag)> {
        self.map.iter_mut()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &Tag> {
        self.map.values()
    }

    /// Удобный доступ к числовым тегам (как `getInt`/`getLong`/... в оригинале
    /// через `get` + `asInt` и т.п.).
    pub fn get_byte(&self, key: &str) -> Option<i8> {
        self.get(key).and_then(Tag::as_byte)
    }

    pub fn get_short(&self, key: &str) -> Option<i16> {
        self.get(key).and_then(Tag::as_short)
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.get(key).and_then(Tag::as_int)
    }

    pub fn get_long(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(Tag::as_long)
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(Tag::as_float)
    }

    pub fn get_double(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(Tag::as_double)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(Tag::as_bool)
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Tag::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_compound(&self, key: &str) -> Option<&CompoundTag> {
        match self.get(key) {
            Some(Tag::Compound(c)) => Some(c),
            _ => None,
        }
    }

    pub fn get_compound_mut(&mut self, key: &str) -> Option<&mut CompoundTag> {
        match self.get_mut(key) {
            Some(Tag::Compound(c)) => Some(c),
            _ => None,
        }
    }

    /// `getOrCreateCompound` — создаёт compound по ключу, если его нет
    /// или он не compound.
    pub fn get_or_create_compound(&mut self, key: &str) -> &mut CompoundTag {
        let needs_insert = !matches!(self.map.get(key), Some(Tag::Compound(_)));
        if needs_insert {
            self.map.insert(key.to_owned(), Tag::Compound(CompoundTag::new()));
        }
        match self.map.get_mut(key) {
            Some(Tag::Compound(c)) => c,
            _ => unreachable!(),
        }
    }

    pub fn get_list(&self, key: &str) -> Option<&Vec<Tag>> {
        match self.get(key) {
            Some(Tag::List(l)) => Some(l),
            _ => None,
        }
    }

    /// `getListOrEmpty` — пустой срез, если ключ отсутствует/не список.
    pub fn get_list_or_empty(&self, key: &str) -> &[Tag] {
        match self.get(key) {
            Some(Tag::List(l)) => l.as_slice(),
            _ => &[],
        }
    }

    /// `getCompoundOrEmpty` — клон compound или пустой.
    /// В Java возвращается живая ссылка; здесь — owned-копия,
    /// паттерн использования `get → mutate → put` сохраняется.
    pub fn get_compound_or_empty(&self, key: &str) -> CompoundTag {
        self.get_compound(key).cloned().unwrap_or_default()
    }

    /// `getIntOr`.
    pub fn get_int_or(&self, key: &str, default: i32) -> i32 {
        self.get_int(key).unwrap_or(default)
    }

    /// `getLongOr`.
    pub fn get_long_or(&self, key: &str, default: i64) -> i64 {
        self.get_long(key).unwrap_or(default)
    }

    /// `getLongArray`.
    pub fn get_long_array(&self, key: &str) -> Option<&[i64]> {
        match self.get(key) {
            Some(Tag::LongArray(a)) => Some(a.as_slice()),
            _ => None,
        }
    }

    /// `getByteArray`.
    pub fn get_byte_array(&self, key: &str) -> Option<&[i8]> {
        match self.get(key) {
            Some(Tag::ByteArray(a)) => Some(a.as_slice()),
            _ => None,
        }
    }

    /// `getIntArray`.
    pub fn get_int_array(&self, key: &str) -> Option<&[i32]> {
        match self.get(key) {
            Some(Tag::IntArray(a)) => Some(a.as_slice()),
            _ => None,
        }
    }

    /// `getStringOr` — `getString(key).orElse(default)`.
    pub fn get_string_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get_string(key).unwrap_or(default)
    }

    /// `getBooleanOr`.
    pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
        self.get_bool(key).unwrap_or(default)
    }

    // ---------- typed put ----------

    /// `putByte`.
    pub fn put_byte(&mut self, key: &str, value: i8) {
        self.put(key, Tag::Byte(value));
    }

    /// `putShort`.
    pub fn put_short(&mut self, key: &str, value: i16) {
        self.put(key, Tag::Short(value));
    }

    /// `putInt`.
    pub fn put_int(&mut self, key: &str, value: i32) {
        self.put(key, Tag::Int(value));
    }

    /// `putLong`.
    pub fn put_long(&mut self, key: &str, value: i64) {
        self.put(key, Tag::Long(value));
    }

    /// `putFloat`.
    pub fn put_float(&mut self, key: &str, value: f32) {
        self.put(key, Tag::Float(value));
    }

    /// `putDouble`.
    pub fn put_double(&mut self, key: &str, value: f64) {
        self.put(key, Tag::Double(value));
    }

    /// `putBoolean`.
    pub fn put_bool(&mut self, key: &str, value: bool) {
        self.put(key, Tag::Byte(if value { 1 } else { 0 }));
    }

    /// `putString`.
    pub fn put_string(&mut self, key: &str, value: &str) {
        self.put(key, Tag::String(value.to_owned()));
    }

    /// `putLongArray`.
    pub fn put_long_array(&mut self, key: &str, value: &[i64]) {
        self.put(key, Tag::LongArray(value.to_vec()));
    }

    /// `putIntArray`.
    pub fn put_int_array(&mut self, key: &str, value: &[i32]) {
        self.put(key, Tag::IntArray(value.to_vec()));
    }

    /// `putByteArray`.
    pub fn put_byte_array(&mut self, key: &str, value: &[i8]) {
        self.put(key, Tag::ByteArray(value.to_vec()));
    }

    /// `toSortedMap` — сортировка ключей (используется в SNBT/тестах).
    pub fn to_sorted_map(&self) -> BTreeMap<String, Tag> {
        let mut map = BTreeMap::new();
        for (k, v) in &self.map {
            map.insert(k.clone(), v.clone());
        }
        map
    }

}

impl FromIterator<(String, Tag)> for CompoundTag {
    fn from_iter<I: IntoIterator<Item = (String, Tag)>>(iter: I) -> Self {
        Self { map: IndexMap::from_iter(iter) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_preserved() {
        let mut c = CompoundTag::new();
        c.put("b", Tag::Int(1));
        c.put("a", Tag::Int(2));
        c.put("b", Tag::Int(3));
        let keys: Vec<_> = c.keys().cloned().collect();
        assert_eq!(keys, vec!["b".to_string(), "a".to_string()]);
        assert_eq!(c.get_int("b"), Some(3));
        assert_eq!(c.len(), 2);
    }
}
