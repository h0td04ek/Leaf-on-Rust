//! `Stat<T>` — статистика (type + value).

/// `Stat<T>` — `{ statType, value }` с именем `type:value`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Stat {
    /// `type` — `StatType` registry id.
    pub stat_type: i32,
    /// `value` — `T` registry id (item/block/entity id).
    pub value: i32,
    /// `name` — `buildName(type, value)` = `"type_registry_key.value_registry_key"`.
    pub name: String,
}

impl Stat {
    /// `new(statType, value, name)`.
    pub fn new(stat_type: i32, value: i32, name: String) -> Self {
        Self { stat_type, value, name }
    }

    /// `buildName(type, value)` — `locationToKey(type) + ":" + locationToKey(value)`.
    pub fn build_name(stat_type_key: &str, value_key: &str) -> String {
        format!("{}.{}", stat_type_key.replace(':', "."), value_key.replace(':', "."))
    }

    /// `getType`.
    pub fn get_type(&self) -> i32 {
        self.stat_type
    }

    /// `getValue`.
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// `getName`.
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
