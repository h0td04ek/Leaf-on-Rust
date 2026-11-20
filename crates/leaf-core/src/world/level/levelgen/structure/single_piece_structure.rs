//! Перенос `net.minecraft.world.level.levelgen.structure.SinglePieceStructure`
//! — база структур из одного куска: `findGenerationPoint` через
//! `getLowestY` + `onTopOfChunkCenter` + один кусок от
//! `PieceConstructor`.

use crate::core::block_pos::BlockPos;
use crate::resources::identifier::Identifier;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::structure::structure::{
    get_lowest_y, on_top_of_chunk_center, GenerationContext, GenerationStub, Structure,
    StructureSettings,
};
use crate::world::level::levelgen::structure::structure_piece::StructurePiece;
use crate::world::level::levelgen::structure::structure_type::StructureType;
use std::sync::Arc;

/// `SinglePieceStructure.PieceConstructor` —
/// `construct(WorldgenRandom, minBlockX, minBlockZ)`.
pub type PieceConstructor = Arc<
    dyn Fn(&mut dyn crate::random::RandomSource, i32, i32) -> Box<dyn StructurePiece>
        + Send
        + Sync,
>;

/// `SinglePieceStructure` — фабрика `Structure` с
/// ванильным `findGenerationPoint`:
/// `getLowestY(context, width, depth) < seaLevel → empty`,
/// иначе `onTopOfChunkCenter(WORLD_SURFACE_WG)` + один кусок
/// от конструктора в `minBlockX/minBlockZ` чанка.
pub fn single_piece_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
    width: i32,
    depth: i32,
    constructor: PieceConstructor,
) -> Structure {
    let ctor = constructor.clone();
    let mut structure = Structure::new(id, settings, structure_type);
    structure.find_generation_point = Some(Arc::new(move |context: &mut GenerationContext| {
        let lowest_y = get_lowest_y(context, width, depth);
        if lowest_y < context.level.sea_level() {
            return None;
        }
        let seed = context.seed;
        let chunk_pos = context.chunk_pos;
        let min_x = chunk_pos.get_min_block_x();
        let min_z = chunk_pos.get_min_block_z();
        let ctor = ctor.clone();
        let stub = on_top_of_chunk_center(context, HeightmapType::WorldSurfaceWg, move |pieces| {
            // `context.random` в состоянии после findGenerationPoint —
            // onTopOfChunkCenter его не расходует, поэтому свежий
            // WorldgenRandom с тем же seed/чанком эквивалентен.
            let mut random = GenerationContext::make_random(seed, chunk_pos);
            pieces.push(ctor(&mut random, min_x, min_z));
        });
        Some(stub)
    }));
    structure
}
