//! Перенос `net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings`.

use crate::core::block_pos::BlockPos;
use crate::random::RandomSource;
use crate::random::create;
use crate::util::mth;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::templatesystem::liquid_settings::LiquidSettings;
use crate::world::level::levelgen::structure::templatesystem::processors::StructureProcessor;
use crate::world::level::levelgen::structure::templatesystem::structure_template::Palette;
use std::rc::Rc;

/// `StructurePlaceSettings` — настройки размещения шаблона
/// (mirror/rotation/pivot/processors/liquidSettings/palette).
#[derive(Clone)]
pub struct StructurePlaceSettings {
    /// `mirror`.
    pub mirror: Mirror,
    /// `rotation`.
    pub rotation: Rotation,
    /// `rotationPivot`.
    pub rotation_pivot: BlockPos,
    /// `ignoreEntities`.
    pub ignore_entities: bool,
    /// `boundingBox`.
    pub bounding_box: Option<BoundingBox>,
    /// `liquidSettings`.
    pub liquid_settings: LiquidSettings,
    /// `random` — заданный RandomSource (None → из seedPos).
    pub random: Option<Rc<std::cell::RefCell<Box<dyn RandomSource>>>>,
    /// `palette` — CraftBukkit force-palette (-1 = случайная).
    pub palette: i32,
    /// `processors`.
    pub processors: Vec<Rc<dyn StructureProcessor>>,
    /// `knownShape`.
    pub known_shape: bool,
    /// `finalizeEntities`.
    pub finalize_entities: bool,
}

impl Default for StructurePlaceSettings {
    fn default() -> Self {
        Self {
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            rotation_pivot: BlockPos::ZERO,
            ignore_entities: false,
            bounding_box: None,
            liquid_settings: LiquidSettings::ApplyWaterlogging,
            random: None,
            palette: -1,
            processors: Vec::new(),
            known_shape: false,
            finalize_entities: false,
        }
    }
}

impl StructurePlaceSettings {
    /// `copy`.
    pub fn copy(&self) -> Self {
        Self {
            mirror: self.mirror,
            rotation: self.rotation,
            rotation_pivot: self.rotation_pivot,
            ignore_entities: self.ignore_entities,
            bounding_box: self.bounding_box,
            liquid_settings: self.liquid_settings,
            random: self.random.clone(),
            palette: self.palette,
            processors: self.processors.clone(),
            known_shape: self.known_shape,
            finalize_entities: self.finalize_entities,
        }
    }

    /// `setMirror`.
    pub fn set_mirror(&mut self, mirror: Mirror) -> &mut Self {
        self.mirror = mirror;
        self
    }
    /// `setRotation`.
    pub fn set_rotation(&mut self, rotation: Rotation) -> &mut Self {
        self.rotation = rotation;
        self
    }
    /// `setRotationPivot`.
    pub fn set_rotation_pivot(&mut self, pivot: BlockPos) -> &mut Self {
        self.rotation_pivot = pivot;
        self
    }
    /// `setIgnoreEntities`.
    pub fn set_ignore_entities(&mut self, ignore: bool) -> &mut Self {
        self.ignore_entities = ignore;
        self
    }
    /// `setBoundingBox`.
    pub fn set_bounding_box(&mut self, bounding_box: BoundingBox) -> &mut Self {
        self.bounding_box = Some(bounding_box);
        self
    }
    /// `setRandom`.
    pub fn set_random(&mut self, random: Box<dyn RandomSource>) -> &mut Self {
        self.random = Some(Rc::new(std::cell::RefCell::new(random)));
        self
    }
    /// `setLiquidSettings`.
    pub fn set_liquid_settings(&mut self, settings: LiquidSettings) -> &mut Self {
        self.liquid_settings = settings;
        self
    }
    /// `setKnownShape`.
    pub fn set_known_shape(&mut self, known: bool) -> &mut Self {
        self.known_shape = known;
        self
    }
    /// `clearProcessors`.
    pub fn clear_processors(&mut self) -> &mut Self {
        self.processors.clear();
        self
    }
    /// `addProcessor`.
    pub fn add_processor(&mut self, processor: Rc<dyn StructureProcessor>) -> &mut Self {
        self.processors.push(processor);
        self
    }
    /// `popProcessor`.
    pub fn pop_processor(&mut self, processor: &Rc<dyn StructureProcessor>) -> &mut Self {
        if let Some(pos) = self
            .processors
            .iter()
            .position(|p| Rc::ptr_eq(p, processor))
        {
            self.processors.remove(pos);
        }
        self
    }
    /// `getMirror`.
    pub fn get_mirror(&self) -> Mirror {
        self.mirror
    }
    /// `getRotation`.
    pub fn get_rotation(&self) -> Rotation {
        self.rotation
    }
    /// `getRotationPivot`.
    pub fn get_rotation_pivot(&self) -> BlockPos {
        self.rotation_pivot
    }
    /// `getRandom(seedPos)` — заданный random или новый от
    /// `Mth.getSeed(seedPos)`/millis.
    pub fn get_random(&self, seed_pos: Option<BlockPos>) -> Box<dyn RandomSource> {
        if let Some(r) = &self.random {
            // общий источник — возвращаем независимый fork
            // (Java делит один RandomSource; fork сохраняет
            // детерминизм без aliasing).
            return r.borrow_mut().fork();
        }
        match seed_pos {
            None => create(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0)),
            Some(pos) => create(mth::get_seed(pos.x, pos.y, pos.z)),
        }
    }
    /// `isIgnoreEntities`.
    pub fn is_ignore_entities(&self) -> bool {
        self.ignore_entities
    }
    /// `getBoundingBox`.
    pub fn get_bounding_box(&self) -> Option<BoundingBox> {
        self.bounding_box
    }
    /// `getKnownShape`.
    pub fn get_known_shape(&self) -> bool {
        self.known_shape
    }
    /// `getProcessors`.
    pub fn get_processors(&self) -> &[Rc<dyn StructureProcessor>] {
        &self.processors
    }
    /// `shouldApplyWaterlogging`.
    pub fn should_apply_waterlogging(&self) -> bool {
        self.liquid_settings == LiquidSettings::ApplyWaterlogging
    }
    /// `getRandomPalette` — palette index >= 0 → фиксированная,
    /// иначе случайная.
    pub fn get_random_palette<'a>(&self, palettes: &'a [Palette], pos: Option<BlockPos>) -> &'a Palette {
        let size = palettes.len();
        assert!(size > 0, "No palettes");
        if self.palette >= 0 {
            assert!(
                (self.palette as usize) < size,
                "Palette index out of bounds. Got {} where there are only {} palettes available.",
                self.palette,
                size
            );
            &palettes[self.palette as usize]
        } else {
            let mut random = self.get_random(pos);
            &palettes[random.next_int_bound(size as i32) as usize]
        }
    }
    /// `setFinalizeEntities`.
    pub fn set_finalize_entities(&mut self, finalize: bool) -> &mut Self {
        self.finalize_entities = finalize;
        self
    }
    /// `shouldFinalizeEntities`.
    pub fn should_finalize_entities(&self) -> bool {
        self.finalize_entities
    }
}
