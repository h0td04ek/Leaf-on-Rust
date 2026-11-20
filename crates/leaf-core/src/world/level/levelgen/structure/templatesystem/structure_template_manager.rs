//! Перенос `net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager`
//! — загрузка/кэш шаблонов из ресурсов (`structures/*.nbt`).

use crate::nbt::compound::CompoundTag;
use crate::resources::identifier::Identifier;
use crate::world::level::levelgen::structure::templatesystem::structure_template::StructureTemplate;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Загрузчик NBT-ресурса (`ResourceManager.getResourceOrThrow`
/// + `NbtIo.readCompressed`). Подключается с resource-слоем.
pub type NbtLoader = Arc<dyn Fn(&Identifier) -> Option<CompoundTag> + Send + Sync>;

fn loader() -> &'static Mutex<Option<NbtLoader>> {
    static L: OnceLock<Mutex<Option<NbtLoader>>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(None))
}

/// Установка глобального NBT-загрузчика (аналог ResourceManager).
pub fn set_nbt_loader(load: NbtLoader) {
    *loader().lock().unwrap() = Some(load);
}

/// `StructureTemplateManager` — кэш `Identifier → Arc<StructureTemplate>`.
#[derive(Default)]
pub struct StructureTemplateManager {
    /// `cache` — id → шаблон (загруженные).
    cache: Mutex<HashMap<Identifier, Arc<StructureTemplate>>>,
}

impl StructureTemplateManager {
    /// `new`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `get(location)` — шаблон по id (`structures/` prefix).
    /// Возвращает `Option` — None если ресурс не найден.
    pub fn get(&self, location: &Identifier) -> Option<Arc<StructureTemplate>> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(t) = cache.get(location) {
                return Some(t.clone());
            }
        }
        // `structures/` + path + `.nbt`
        let resource_id = Identifier::with_default_namespace(&format!(
            "structures/{}.nbt",
            location.path
        ))
        .unwrap_or_else(|_| location.clone());
        let nbt = loader().lock().unwrap().as_ref().and_then(|l| l(&resource_id))?;
        let template = StructureTemplate::load(&nbt)?;
        let arc = Arc::new(template);
        self.cache
            .lock()
            .unwrap()
            .insert(location.clone(), arc.clone());
        Some(arc)
    }

    /// `getOrCreate` — создать пустой шаблон и закэшировать
    /// (для `/place` и т.п.).
    pub fn get_or_create(&self, location: &Identifier) -> Arc<StructureTemplate> {
        if let Some(t) = self.get(location) {
            return t;
        }
        let arc = Arc::new(StructureTemplate::new());
        self.cache
            .lock()
            .unwrap()
            .insert(location.clone(), arc.clone());
        arc
    }

    /// `readStructure(location)` — без кэша (принудительное
    /// чтение).
    pub fn read_structure(&self, location: &Identifier) -> Option<StructureTemplate> {
        let resource_id = Identifier::with_default_namespace(&format!(
            "structures/{}.nbt",
            location.path
        ))
        .unwrap_or_else(|_| location.clone());
        let nbt = loader().lock().unwrap().as_ref().and_then(|l| l(&resource_id))?;
        StructureTemplate::load(&nbt)
    }

    /// Очистка кэша.
    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().clear();
    }
}
