//! Перенос `StructureType`/`StructureType`-атрибутов структуры.

use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

/// `StructureType` — дескриптор типа структуры
/// (в Java — record в `StructureType`-enum-подобной иерархии;
/// здесь — идентичность + константы типов).
#[derive(Clone, Debug)]
pub struct StructureType {
    /// id типа (`minecraft:village` и т.п.).
    pub id: Identifier,
}

impl StructureType {
    /// `new(id)`.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }
}

fn registry() -> &'static RwLock<HashMap<Identifier, Arc<StructureType>>> {
    static REG: OnceLock<RwLock<HashMap<Identifier, Arc<StructureType>>>> = OnceLock::new();
    REG.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Регистрация типа структуры.
pub fn register_structure_type(id: Identifier) -> Arc<StructureType> {
    let ty = Arc::new(StructureType::new(id.clone()));
    registry().write().unwrap().insert(id, ty.clone());
    ty
}

/// `by id`.
pub fn structure_type_by_id(id: &Identifier) -> Option<Arc<StructureType>> {
    registry().read().unwrap().get(id).cloned()
}
