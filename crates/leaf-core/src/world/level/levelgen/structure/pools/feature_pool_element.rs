//! Перенос `FeaturePoolElement` — элемент пула, размещающий
//! `PlacedFeature`.

use super::structure_pool_element::{PoolGeneratorAccess, StructurePoolElement};
use super::structure_template_pool::Projection;
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::core::vec3i::Vec3i;
use crate::random::RandomSource;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::levelgen::feature::feature::{
    FeatureGeneratorAccess, FeatureLevelAccess, FeaturePlaceContext,
};
use crate::world::level::levelgen::placement::placement_modifier::{
    PlacementContext, PlacementGeneratorAccess, PlacementLevelAccess, PlacementModifier,
};
use crate::world::level::levelgen::feature::placed_feature::PlacedFeature;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::structure_manager::StructureManager;
use crate::world::level::levelgen::structure::templatesystem::liquid_settings::LiquidSettings;
use crate::world::level::levelgen::structure::templatesystem::structure_template::{
    JigsawBlockInfo, StructureBlockInfo,
};
use crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager;
use crate::world::level::level_reader::LevelReader;
use std::rc::Rc;
use std::sync::Arc;

/// Обёртки для trait-object upcasting (Rust не умеет
/// `&dyn LevelReader` → `&dyn FeatureLevelAccess`).
pub struct LevelAccess<'a>(pub &'a dyn LevelReader);
impl FeatureLevelAccess for LevelAccess<'_> {}
impl PlacementLevelAccess for LevelAccess<'_> {}

pub struct GeneratorAccess<'a>(pub &'a dyn PoolGeneratorAccess);
impl FeatureGeneratorAccess for GeneratorAccess<'_> {}
impl PlacementGeneratorAccess for GeneratorAccess<'_> {}

/// `FeaturePoolElement.PoolFeaturePlaceContext` — контекст
/// `PlacedFeature.place`.
///
/// `FeaturePlaceContext::random(&self) -> &mut dyn RandomSource`
/// требует interior mutability: храним сырой указатель на
/// внешний `random` (worldgen однотредовый, как в Java).
pub struct PoolFeaturePlaceContext<'a> {
    level: LevelAccess<'a>,
    generator: GeneratorAccess<'a>,
    random: *mut (dyn RandomSource + 'a),
    pos: BlockPos,
    _marker: std::marker::PhantomData<&'a mut (dyn RandomSource + 'a)>,
}

impl<'a> PoolFeaturePlaceContext<'a> {
    /// `new`.
    pub fn new(
        level: &'a dyn LevelReader,
        generator: &'a dyn PoolGeneratorAccess,
        random: &'a mut dyn RandomSource,
        pos: BlockPos,
    ) -> Self {
        Self {
            level: LevelAccess(level),
            generator: GeneratorAccess(generator),
            random,
            pos,
            _marker: std::marker::PhantomData,
        }
    }
}

// SAFETY: контекст живёт не дольше `random`, который ему
// передан; доступ к random идёт только через &mut-методы
// контекста по одному за раз (worldgen последователен).
unsafe impl Send for PoolFeaturePlaceContext<'_> {}

impl<'a> FeaturePlaceContext for PoolFeaturePlaceContext<'a> {
    fn origin(&self) -> BlockPos {
        self.pos
    }
    fn level(&self) -> &dyn FeatureLevelAccess {
        &self.level
    }
    fn chunk_generator(&self) -> &dyn FeatureGeneratorAccess {
        &self.generator
    }
    fn random(&self) -> &mut dyn RandomSource {
        unsafe { &mut *self.random }
    }
}

impl<'a> PlacementContext for PoolFeaturePlaceContext<'a> {
    fn level(&self) -> &dyn PlacementLevelAccess {
        &self.level
    }
    fn chunk_generator(&self) -> &dyn PlacementGeneratorAccess {
        &self.generator
    }
    fn random(&self) -> &mut dyn RandomSource {
        unsafe { &mut *self.random }
    }
    fn origin(&self) -> BlockPos {
        self.pos
    }
}

/// `FeaturePoolElement`.
pub struct FeaturePoolElement {
    /// `feature` — holder на PlacedFeature.
    pub feature: Holder<PlacedFeature>,
    /// `projection`.
    pub projection: Projection,
}

impl FeaturePoolElement {
    /// `new`.
    pub fn new(feature: Holder<PlacedFeature>, projection: Projection) -> Self {
        Self { feature, projection }
    }
}

impl StructurePoolElement for FeaturePoolElement {
    fn get_size(&self, _m: &StructureTemplateManager, _r: Rotation) -> Vec3i {
        Vec3i::ZERO
    }

    fn get_shuffled_jigsaw_blocks(
        &self,
        _m: &StructureTemplateManager,
        pos: BlockPos,
        _r: Rotation,
        _random: &mut dyn RandomSource,
    ) -> Vec<JigsawBlockInfo> {
        // Один jigsaw-блок с дефолтным NBT (final_state=air).
        let mut nbt = crate::nbt::compound::CompoundTag::new();
        nbt.put("name", crate::nbt::tag::Tag::String("minecraft:empty".into()));
        nbt.put("final_state", crate::nbt::tag::Tag::String("minecraft:air".into()));
        nbt.put("pool", crate::nbt::tag::Tag::String("minecraft:empty".into()));
        nbt.put("target", crate::nbt::tag::Tag::String("minecraft:empty".into()));
        nbt.put("joint", crate::nbt::tag::Tag::String("rollable".into()));
        vec![JigsawBlockInfo {
            info: crate::world::level::levelgen::structure::templatesystem::structure_template::StructureBlockInfo::new(
                pos,
                crate::world::level::block::state::BlockState::air(),
                Some(nbt),
            ),
            attachment_pos: pos,
            target: String::new(),
            pool: None,
            name: String::new(),
            final_state: "minecraft:air".to_string(),
        }]
    }

    fn get_bounding_box(
        &self,
        _m: &StructureTemplateManager,
        pos: BlockPos,
        _r: Rotation,
    ) -> BoundingBox {
        BoundingBox::new(pos.x, pos.y, pos.z, pos.x, pos.y, pos.z)
    }

    fn place(
        &self,
        _m: &StructureTemplateManager,
        level: &dyn LevelReader,
        _sm: &StructureManager,
        generator: &dyn PoolGeneratorAccess,
        _offset: BlockPos,
        pos: BlockPos,
        _rotation: Rotation,
        _box: Option<BoundingBox>,
        random: &mut dyn RandomSource,
        _liquid: LiquidSettings,
        _keep_jigsaws: bool,
    ) -> bool {
        let Some(placed) = self.feature.value() else {
            return false;
        };
        // PlacedFeature.place — placement-модификаторы дают
        // count (произведение по цепочке, как у getPositions).
        let mut pos = pos;
        let mut count: i32 = 1;
        {
            let mut ctx = PoolFeaturePlaceContext::new(level, generator, &mut *random, pos);
            for modifier in &placed.placement {
                let m: &dyn PlacementModifier = modifier.as_ref();
                count = count.saturating_mul(m.get_place(&mut ctx, &mut pos));
            }
        }
        let Some(configured) = placed.feature.value() else {
            return false;
        };
        let mut placed_any = false;
        for _ in 0..count.max(0) {
            let mut ctx = PoolFeaturePlaceContext::new(level, generator, &mut *random, pos);
            if configured.feature.place(&mut ctx) {
                placed_any = true;
            }
        }
        placed_any
    }

    fn get_type_name(&self) -> &'static str {
        "feature_pool_element"
    }

    fn get_projection(&self) -> Projection {
        self.projection
    }
}

/// `feature(holder)` фабрика.
pub fn feature(feature: Holder<PlacedFeature>, projection: Projection) -> Rc<dyn StructurePoolElement> {
    Rc::new(FeaturePoolElement::new(feature, projection))
}

/// `Arc` реэкспорт (для сигнатур фабрик).
pub type SharedPlacedFeature = Arc<PlacedFeature>;
