//! Перенос `structures/BuriedTreasureStructure.java` +
//! `structures/BuriedTreasurePieces.java`.

use super::super::bounding_box::BoundingBox;
use super::super::structure::{
    on_top_of_chunk_center, GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_piece::StructurePiece;
use super::super::structure_type::StructureType;
use super::block_state;
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::nbt::compound::CompoundTag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess;
use crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess;
use crate::world::level::levelgen::structure::structure_piece::{create_chest_at, StructurePieceAccessor};
use std::sync::Arc;

/// `BuiltInLootTables.BURIED_TREASURE`.
fn buried_treasure_loot_table() -> Identifier {
    Identifier::minecraft("chests/buried_treasure")
}

/// `BuriedTreasurePieces.BuriedTreasurePiece`.
pub struct BuriedTreasurePiece {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: crate::world::level::block::mirror::Mirror,
    rotation: crate::world::level::block::rotation::Rotation,
}

impl BuriedTreasurePiece {
    /// `new(pos)` — `BoundingBox(pos)`.
    pub fn new(pos: BlockPos) -> Self {
        Self {
            bounding_box: BoundingBox::of_pos(pos),
            gen_depth: 0,
            orientation: None,
            mirror: crate::world::level::block::mirror::Mirror::NONE,
            rotation: crate::world::level::block::rotation::Rotation::NONE,
        }
    }

    /// `isLiquid(state)` — water/lava default state.
    fn is_liquid(state: &BlockState) -> bool {
        state.block.id.path == "water" || state.block.id.path == "lava"
    }
}

impl StructurePiece for BuriedTreasurePiece {
    fn bounding_box(&self) -> BoundingBox {
        self.bounding_box.clone()
    }

    fn bounding_box_mut(&mut self) -> Option<&mut BoundingBox> {
        Some(&mut self.bounding_box)
    }

    fn gen_depth(&self) -> i32 {
        self.gen_depth
    }

    fn set_gen_depth(&mut self, gen_depth: i32) {
        self.gen_depth = gen_depth;
    }

    fn orientation(&self) -> Option<Direction> {
        self.orientation
    }

    fn set_orientation(&mut self, orientation: Option<Direction>) {
        self.orientation = orientation;
        super::super::structure_piece::piece_set_orientation(
            orientation,
            &mut self.mirror,
            &mut self.rotation,
        );
    }

    fn rotation(&self) -> crate::world::level::block::rotation::Rotation {
        self.rotation
    }

    fn mirror(&self) -> crate::world::level::block::mirror::Mirror {
        self.mirror
    }

    fn piece_type_id(&self) -> Identifier {
        Identifier::minecraft("buried_treasure")
    }

    fn add_additional_save_data(&self, _tag: &mut CompoundTag) {}

    fn load_additional_save_data(&mut self, _tag: &CompoundTag) {}

    fn add_children(
        &mut self,
        _piece: &mut dyn StructurePiece,
        _pieces: &mut dyn StructurePieceAccessor,
        _random: &mut dyn RandomSource,
    ) {
    }

    fn post_process(
        &mut self,
        level: &dyn LevelReader,
        _structure_manager: &dyn StructureManagerAccess,
        _generator: Option<&dyn PoolGeneratorAccess>,
        random: &mut dyn RandomSource,
        box_: &BoundingBox,
        _chunk_pos: ChunkPos,
        _pos: BlockPos,
    ) {
        let height = level.get_height(
            HeightmapType::OceanFloorWg,
            self.bounding_box.min_x,
            self.bounding_box.min_z,
        );
        let mut mutable_pos = BlockPos::new(self.bounding_box.min_x, height, self.bounding_box.min_z);

        while mutable_pos.y > level.min_build_height() {
            let state = level
                .get_block_state(mutable_pos)
                .unwrap_or_else(BlockState::air);
            let below_pos = mutable_pos.below();
            let state1 = level
                .get_block_state(below_pos)
                .unwrap_or_else(BlockState::air);
            let is_base = [
                "sandstone", "stone", "andesite", "granite", "diorite",
            ]
            .iter()
            .any(|id| state1.block.id.path == *id);
            if !is_base {
                mutable_pos = below_pos;
                continue;
            }

            let state2 = if !state.is_air() && !Self::is_liquid(&state) {
                state
            } else {
                block_state("sand")
            };

            for direction in Direction::VALUES {
                let block_pos = mutable_pos.relative(direction);
                let state3 = level
                    .get_block_state(block_pos)
                    .unwrap_or_else(BlockState::air);
                if state3.is_air() || Self::is_liquid(&state3) {
                    let below1 = block_pos.below();
                    let state4 = level
                        .get_block_state(below1)
                        .unwrap_or_else(BlockState::air);
                    if (state4.is_air() || Self::is_liquid(&state4))
                        && direction != Direction::UP
                    {
                        level.set_block_state(block_pos, &state1);
                    } else {
                        level.set_block_state(block_pos, &state2);
                    }
                }
            }

            self.bounding_box = BoundingBox::of_pos(mutable_pos);
            create_chest_at(
                level,
                box_,
                random,
                mutable_pos,
                &buried_treasure_loot_table(),
                None,
            );
            return;
        }
    }
}

/// `BuriedTreasureStructure` — фабрика.
pub fn buried_treasure_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
) -> Structure {
    let mut structure = Structure::new(id, settings, structure_type);
    structure.find_generation_point =
        Some(Arc::new(|context: &mut GenerationContext| -> Option<GenerationStub> {
            let chunk_pos = context.chunk_pos;
            let seed = context.seed;
            on_top_of_chunk_center_opt(
                context,
                HeightmapType::OceanFloorWg,
                move |pieces, _context| {
                    let block_x = chunk_pos.get_block_x(2, 1); // getBlockX(9)
                    let block_z = chunk_pos.get_block_z(2, 1); // getBlockZ(9)
                    let _ = seed;
                    pieces.push(Box::new(BuriedTreasurePiece::new(BlockPos::new(
                        block_x, 90, block_z,
                    ))));
                },
            )
        }));
    structure
}

/// `onTopOfChunkCenter` → `Option` (Java возвращает
/// Optional.of(stub) — у нас on_top_of_chunk_center всегда Some).
fn on_top_of_chunk_center_opt(
    context: &mut GenerationContext,
    heightmap_types: HeightmapType,
    generator: impl for<'a, 'b> FnOnce(
            &mut Vec<Box<dyn StructurePiece>>,
            &mut GenerationContext<'b>,
        ) + 'static,
) -> Option<GenerationStub> {
    Some(on_top_of_chunk_center(context, heightmap_types, generator))
}
