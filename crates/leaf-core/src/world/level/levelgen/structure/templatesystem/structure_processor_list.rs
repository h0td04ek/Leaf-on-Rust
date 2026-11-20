//! Перенос `net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessorList`
//! — именованный список процессоров (реестровое значение).
//!
//! Процессоры хранятся как `Rc` (как в `StructurePlaceSettings`),
//! поэтому реестр — thread-local.

use super::processors::StructureProcessor;
use std::cell::RefCell;
use std::rc::Rc;

/// `StructureProcessorList`.
#[derive(Clone)]
pub struct StructureProcessorList {
    /// `processors`.
    pub processors: Vec<Rc<dyn StructureProcessor>>,
}

impl StructureProcessorList {
    /// `new`.
    pub fn new(processors: Vec<Rc<dyn StructureProcessor>>) -> Self {
        Self { processors }
    }

    /// Пустой список.
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }
}

thread_local! {
    static REGISTRY: RefCell<std::collections::HashMap<String, StructureProcessorList>> =
        RefCell::new(std::collections::HashMap::new());
}

/// `register(name, list)`.
pub fn register_processor_list(name: &str, list: StructureProcessorList) {
    REGISTRY.with(|r| r.borrow_mut().insert(name.to_string(), list));
}

/// `byName(name)`.
pub fn processor_list_by_name(name: &str) -> Option<StructureProcessorList> {
    REGISTRY.with(|r| r.borrow().get(name).cloned())
}

/// Пустой `EMPTY`-список (`StructurePoolElement.EMPTY`).
pub fn empty_processor_list() -> StructureProcessorList {
    StructureProcessorList::empty()
}
