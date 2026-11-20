//! Перенос `StructureTemplatePool` + `Projection` + `JigsawJunction`.

use super::structure_pool_element::StructurePoolElement;
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::random::RandomSource;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::structure::templatesystem::processors::{
    GravityProcessor, StructureProcessor,
};
use crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager;
use std::collections::HashMap;
use std::rc::Rc;

/// `SIZE_UNSET`.
const SIZE_UNSET: i32 = i32::MIN;

/// `Pair` — пара (элемент, вес) из `getTemplates`.
pub type PoolElementWeight = (Rc<dyn StructurePoolElement>, i32);

/// `StructureTemplatePool.Projection`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Projection {
    /// `TERRAIN_MATCHING` — гравитация к поверхности.
    TerrainMatching,
    /// `RIGID` — без адаптации.
    Rigid,
}

impl Projection {
    /// `getName`/`getSerializedName`.
    pub fn get_name(&self) -> &'static str {
        match self {
            Projection::TerrainMatching => "terrain_matching",
            Projection::Rigid => "rigid",
        }
    }

    /// `byName`.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "terrain_matching" => Some(Self::TerrainMatching),
            "rigid" => Some(Self::Rigid),
            _ => None,
        }
    }

    /// `getProcessors` — процессоры проекции.
    pub fn get_processors(&self) -> Vec<Rc<dyn StructureProcessor>> {
        match self {
            Projection::TerrainMatching => vec![Rc::new(GravityProcessor::new(
                HeightmapType::WorldSurfaceWg,
                -1,
            ))],
            Projection::Rigid => Vec::new(),
        }
    }
}

/// `StructureTemplatePool`.
pub struct StructureTemplatePool {
    /// `rawTemplates` — (элемент, вес).
    raw_templates: Vec<PoolElementWeight>,
    /// `templates` — развёрнутые по весу элементы.
    templates: Vec<Rc<dyn StructurePoolElement>>,
    /// `fallback` — holder на резервный пул (`Rc`-индирекция
    /// против бесконечного размера, как reference в Java).
    pub fallback: Holder<Rc<StructureTemplatePool>>,
    /// `maxSize` — кэш (`SIZE_UNSET` пока не вычислено).
    max_size: std::cell::Cell<i32>,
}

impl StructureTemplatePool {
    /// `new(fallback, rawTemplates)` — разворачивает веса.
    pub fn new(
        fallback: Holder<Rc<StructureTemplatePool>>,
        raw_templates: Vec<PoolElementWeight>,
    ) -> Self {
        let mut templates = Vec::new();
        for (element, weight) in &raw_templates {
            for _ in 0..*weight {
                templates.push(element.clone());
            }
        }
        Self {
            raw_templates,
            templates,
            fallback,
            max_size: std::cell::Cell::new(SIZE_UNSET),
        }
    }

    /// `getMaxSize` — максимальный Y-span bounding box'ов
    /// (кэшируется).
    pub fn get_max_size(&self, structure_template_manager: &StructureTemplateManager) -> i32 {
        if self.max_size.get() == SIZE_UNSET {
            let max = self
                .templates
                .iter()
                .filter(|t| !t.is_empty_element())
                .map(|t| {
                    t.get_bounding_box(structure_template_manager, BlockPos::ZERO, crate::world::level::block::rotation::Rotation::NONE)
                        .get_y_span()
                })
                .max()
                .unwrap_or(0);
            self.max_size.set(max);
        }
        self.max_size.get()
    }

    /// `getTemplates` (VisibleForTesting).
    pub fn get_templates(&self) -> &[(Rc<dyn StructurePoolElement>, i32)] {
        &self.raw_templates
    }

    /// `getFallback`.
    pub fn get_fallback(&self) -> &Holder<Rc<StructureTemplatePool>> {
        &self.fallback
    }

    /// `getRandomTemplate`.
    pub fn get_random_template(&self, random: &mut dyn RandomSource) -> Rc<dyn StructurePoolElement> {
        if self.templates.is_empty() {
            return super::empty_pool_element::empty_element_rc();
        }
        let i = random.next_int_bound(self.templates.len() as i32) as usize;
        self.templates[i].clone()
    }

    /// `getShuffledTemplates`.
    pub fn get_shuffled_templates(
        &self,
        random: &mut dyn RandomSource,
    ) -> Vec<Rc<dyn StructurePoolElement>> {
        let mut out = self.templates.clone();
        // Fisher–Yates как Util.shuffledCopy.
        for i in (1..out.len()).rev() {
            let j = random.next_int_bound((i + 1) as i32) as usize;
            out.swap(i, j);
        }
        out
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.templates.len()
    }
}

/// `JigsawJunction` — связь jigsaw-блоков.
#[derive(Clone, Debug, PartialEq)]
pub struct JigsawJunction {
    /// `sourceX`.
    pub source_x: i32,
    /// `sourceGroundY`.
    pub source_ground_y: i32,
    /// `sourceZ`.
    pub source_z: i32,
    /// `deltaY`.
    pub delta_y: i32,
    /// `destProjection`.
    pub dest_projection: Projection,
}

impl JigsawJunction {
    /// `new`.
    pub fn new(
        source_x: i32,
        source_ground_y: i32,
        source_z: i32,
        delta_y: i32,
        dest_projection: Projection,
    ) -> Self {
        Self {
            source_x,
            source_ground_y,
            source_z,
            delta_y,
            dest_projection,
        }
    }
}

/// `HashMap`-алиас для alias-пакета (понадобится позже).
pub type PoolElementRegistry = HashMap<String, Rc<dyn StructurePoolElement>>;
