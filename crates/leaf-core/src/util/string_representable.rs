//! Перенос `net.minecraft.util.StringRepresentable` (без DFU-части).
//!
//! `getSerializedName` + name-lookup по значениям. `EnumCodec`/
//! `StringRepresentableCodec` требуют DataFixerUpper — будут добавлены
//! при порте codec-слоя.

use std::collections::HashMap;
use std::sync::Arc;

/// `StringRepresentable` — объект с сериализуемым именем.
pub trait StringRepresentable {
    fn get_serialized_name(&self) -> String;
}

impl<T: StringRepresentable + ?Sized> StringRepresentable for Arc<T> {
    fn get_serialized_name(&self) -> String {
        (**self).get_serialized_name()
    }
}

impl<T: StringRepresentable + ?Sized> StringRepresentable for &T {
    fn get_serialized_name(&self) -> String {
        (**self).get_serialized_name()
    }
}

impl<T: StringRepresentable + ?Sized> StringRepresentable for Box<T> {
    fn get_serialized_name(&self) -> String {
        (**self).get_serialized_name()
    }
}

/// `PRE_BUILT_MAP_THRESHOLD`.
pub const PRE_BUILT_MAP_THRESHOLD: usize = 16;

/// `createNameLookup(values)` — имя → значение.
/// Для >16 элементов строится HashMap, иначе линейный поиск.
pub fn create_name_lookup<'a, T: StringRepresentable>(values: &'a [T]) -> Arc<dyn Fn(&str) -> Option<&'a T> + 'a> {
    if values.len() > PRE_BUILT_MAP_THRESHOLD {
        let map: HashMap<String, usize> = values
            .iter()
            .enumerate()
            .map(|(i, v)| (v.get_serialized_name(), i))
            .collect();
        Arc::new(move |name: &str| map.get(name).map(|&i| &values[i]))
    } else {
        Arc::new(move |name: &str| values.iter().find(|v| v.get_serialized_name() == name))
    }
}

/// `createNameLookup(values, keyFunction)` — произвольный ключ сериализации.
pub fn create_name_lookup_with<'a, T, F>(
    values: &'a [T],
    key_function: F,
) -> Arc<dyn Fn(&str) -> Option<&'a T> + 'a>
where
    F: Fn(&T) -> String + 'a,
{
    if values.len() > PRE_BUILT_MAP_THRESHOLD {
        let map: HashMap<String, usize> = values
            .iter()
            .enumerate()
            .map(|(i, v)| (key_function(v), i))
            .collect();
        Arc::new(move |name: &str| map.get(name).map(|&i| &values[i]))
    } else {
        Arc::new(move |name: &str| {
            values.iter().find(|v| key_function(v) == name)
        })
    }
}
