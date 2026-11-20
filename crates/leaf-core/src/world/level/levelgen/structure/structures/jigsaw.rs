//! `JigsawStructure` — jigsaw-структура поверх `JigsawPlacement`
//! (шаблонные пулы, expansion hack, heightmap-проекция, max distance,
//! pool aliases, dimension padding, liquid settings).

use super::super::structure::{GenerationContext, GenerationStub, Structure, StructureSettings};
use super::super::structure_type::StructureType;
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::resources::resource_key::ResourceKey;
use crate::resources::identifier::Identifier;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::heightproviders::{HeightProvider, WorldGenerationContext};
use crate::world::level::levelgen::structure::pools::jigsaw_placement::{
    add_pieces, DimensionPadding, JigsawContext, JigsawMaxDistance,
};
use crate::world::level::levelgen::structure::pools::pool_alias::{
    create_alias_lookup, PoolAliasBinding,
};
use crate::world::level::levelgen::structure::pools::structure_template_pool::StructureTemplatePool;
use crate::world::level::levelgen::structure::templatesystem::liquid_settings::LiquidSettings;
use std::rc::Rc;
use std::sync::Arc;

type PoolKey = ResourceKey<Rc<StructureTemplatePool>>;

/// `JigsawStructure` — конфиг jigsaw-структуры.
pub struct JigsawStructureConfig {
    /// `startPool`.
    pub start_pool: Holder<Rc<StructureTemplatePool>>,
    /// `startJigsawName`.
    pub start_jigsaw_name: Option<Identifier>,
    /// `maxDepth` (0..=20).
    pub max_depth: i32,
    /// `startHeight`.
    pub start_height: HeightProvider,
    /// `useExpansionHack`.
    pub use_expansion_hack: bool,
    /// `projectStartToHeightmap`.
    pub project_start_to_heightmap: Option<HeightmapType>,
    /// `maxDistanceFromCenter`.
    pub max_distance_from_center: JigsawMaxDistance,
    /// `poolAliases`.
    pub pool_aliases: Vec<Rc<dyn PoolAliasBinding>>,
    /// `dimensionPadding`.
    pub dimension_padding: DimensionPadding,
    /// `liquidSettings`.
    pub liquid_settings: LiquidSettings,
}

/// `JigsawStructure::verifyRange` — проверка
/// `maxDistance.horizontal + terrainAdaptation <= 128`.
pub fn verify_range(horizontal: i32, adaptation_blocks: i32) -> bool {
    horizontal + adaptation_blocks <= 128
}

/// `JigsawStructure.findGenerationPoint`.
fn find_generation_point(cfg: &JigsawStructureConfig, context: &mut GenerationContext) -> Option<GenerationStub> {
    let chunk_pos = context.chunk_pos;
    let min_x = chunk_pos.get_min_block_x();
    let min_z = chunk_pos.get_min_block_z();

    // `startHeight.sample(random, new WorldGenerationContext(generator, heightAccessor))`.
    // У нас границы берём из LevelReader (generator-границы — через
    // pool_generator, если он их отдаёт; иначе level).
    let (gen_min_y, gen_depth) = context
        .pool_generator
        .as_ref()
        .map(|g| (g.get_min_y(), (g.get_max_y() - g.get_min_y()).max(0)))
        .unwrap_or((context.level.min_build_height(), context.level.height()));
    let wg_context = WorldGenerationContext::from_bounds(
        context.level.min_build_height(),
        context.level.height(),
        gen_min_y,
        gen_depth,
    );
    let i = cfg.start_height.sample(&mut context.random, &wg_context);
    let block_pos = BlockPos::new(min_x, i, min_z);

    let template_manager = context.template_manager.clone()?;
    let pools = context.pool_registry.clone()?;
    let generator = context.pool_generator.clone()?;
    let alias_lookup = create_alias_lookup(&cfg.pool_aliases, block_pos, context.seed);

    {
        let mut jc = JigsawContext {
            level: context.level,
            generator: generator.as_ref(),
            template_manager,
            pools: pools.as_ref(),
            random: &mut context.random,
            chunk_pos,
        };
        add_pieces(
            &mut jc,
            cfg.start_pool.clone(),
            cfg.start_jigsaw_name.clone(),
            cfg.max_depth,
            block_pos,
            cfg.use_expansion_hack,
            cfg.project_start_to_heightmap,
            cfg.max_distance_from_center,
            alias_lookup.as_ref(),
            cfg.dimension_padding,
            cfg.liquid_settings,
        )
    }
}

/// `JigsawStructure(id, settings, type, cfg)` — фабрика `Structure`.
pub fn jigsaw_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
    cfg: JigsawStructureConfig,
) -> Structure {
    let cfg = Arc::new(cfg);
    let mut structure = Structure::new(id, settings, structure_type);
    structure.find_generation_point =
        Some(Arc::new(move |context: &mut GenerationContext| -> Option<GenerationStub> {
            find_generation_point(&cfg, context)
        }));
    structure
}
