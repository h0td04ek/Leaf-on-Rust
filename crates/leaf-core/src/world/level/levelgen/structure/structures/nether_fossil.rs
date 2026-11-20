//! Перенос `structures/NetherFossilStructure.java` +
//! `structures/NetherFossilPieces.java`.

use super::super::bounding_box::BoundingBox;
use super::super::structure::{
    GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_piece::{StructurePiece, StructurePieceAccessor, state_rotate};
use super::super::structure_type::StructureType;
use super::super::template_structure_piece::{TemplateStructurePiece, UPDATE_CLIENTS};
use super::super::templatesystem::processors::block_ignore_processor::BlockIgnoreProcessor;
use super::super::templatesystem::structure_place_settings::StructurePlaceSettings;
use super::super::templatesystem::structure_template_manager::StructureTemplateManager;
use super::super::structure_manager::StructureManagerAccess;
use super::super::pools::structure_pool_element::PoolGeneratorAccess;
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightproviders::{HeightProvider, WorldGenerationContext};
use std::sync::Arc;

/// `FOSSILS` — 14 шаблонов nether_fossils.
const FOSSILS: [&str; 14] = [
    "nether_fossils/fossil_1",
    "nether_fossils/fossil_2",
    "nether_fossils/fossil_3",
    "nether_fossils/fossil_4",
    "nether_fossils/fossil_5",
    "nether_fossils/fossil_6",
    "nether_fossils/fossil_7",
    "nether_fossils/fossil_8",
    "nether_fossils/fossil_9",
    "nether_fossils/fossil_10",
    "nether_fossils/fossil_11",
    "nether_fossils/fossil_12",
    "nether_fossils/fossil_13",
    "nether_fossils/fossil_14",
];

fn fossil_id(path: &str) -> Identifier {
    Identifier::with_default_namespace(path).unwrap_or_else(|_| Identifier::minecraft(path))
}

/// `NetherFossilPieces.addPieces` — eager-вариант: возвращает
/// готовый кусок (Java добавляет в pieces напрямую).
pub fn add_piece(
    structure_template_manager: &StructureTemplateManager,
    random: &mut dyn RandomSource,
    pos: BlockPos,
) -> NetherFossilPiece {
    let random1 = Rotation::get_random(random);
    let location = FOSSILS[random.next_int_bound(FOSSILS.len() as i32) as usize];
    NetherFossilPiece::new(
        structure_template_manager,
        &fossil_id(location),
        pos,
        random1,
    )
}

/// `NetherFossilPieces.NetherFossilPiece`.
pub struct NetherFossilPiece {
    /// `TemplateStructurePiece`-часть.
    pub base: TemplateStructurePiece,
}

impl NetherFossilPiece {
    /// `new(manager, location, pos, rotation)`.
    pub fn new(
        structure_template_manager: &StructureTemplateManager,
        location: &Identifier,
        pos: BlockPos,
        rotation: Rotation,
    ) -> Self {
        Self {
            base: TemplateStructurePiece::new(
                Identifier::minecraft("nether_fossil"),
                0,
                structure_template_manager,
                location,
                location.to_string(),
                make_settings(rotation),
                pos,
            ),
        }
    }

    /// `new(tag, manager)` — загрузка из NBT.
    pub fn from_nbt(
        tag: &CompoundTag,
        structure_template_manager: &StructureTemplateManager,
    ) -> Self {
        let rot = tag
            .get_int("Rot")
            .map(Rotation::by_index)
            .unwrap_or(Rotation::NONE);
        Self {
            base: TemplateStructurePiece::from_nbt(
                Identifier::minecraft("nether_fossil"),
                tag,
                structure_template_manager,
                |_location| make_settings(rot),
            ),
        }
    }

    /// `placeDriedGhast`.
    fn place_dried_ghast(
        &self,
        level: &dyn LevelReader,
        template_box: &BoundingBox,
        box_: &BoundingBox,
    ) {
        // `RandomSource.create(seed).forkPositional().at(center)` —
        // детерминированный positional random от сида мира и
        // центра bounding box'а шаблона.
        let center = template_box.get_center();
        let mut random_source = crate::random::legacy::LegacyRandomSource::new(mix_seed(
            level.world_seed(),
            center,
        ));
        if random_source.next_float() >= 0.5 {
            return;
        }
        let i = template_box.min_x + random_source.next_int_bound(template_box.get_x_span());
        let min_y = template_box.min_y;
        let i1 = template_box.min_z + random_source.next_int_bound(template_box.get_z_span());
        let block_pos = BlockPos::new(i, min_y, i1);
        let is_air = level
            .get_block_state(block_pos)
            .map(|s| s.is_air())
            .unwrap_or(true);
        if is_air && box_.is_inside_pos(block_pos) {
            let state = super::block_state("dried_ghast");
            let rotated = state_rotate(
                state,
                Rotation::get_random(&mut random_source),
            );
            level.set_block_state(block_pos, &rotated);
        }
    }
}

/// Микс сида для positional random (`fork().at(pos)`-эквивалент).
fn mix_seed(seed: i64, pos: BlockPos) -> i64 {
    let mut s = seed;
    s = s.wrapping_mul(0x5DEECE66D).wrapping_add(pos.x as i64);
    s = s.wrapping_mul(0x5DEECE66D).wrapping_add(pos.y as i64);
    s = s.wrapping_mul(0x5DEECE66D).wrapping_add(pos.z as i64);
    s ^ (s >> 33)
}

impl StructurePiece for NetherFossilPiece {
    fn bounding_box(&self) -> BoundingBox {
        self.base.bounding_box()
    }

    fn gen_depth(&self) -> i32 {
        self.base.gen_depth()
    }

    fn set_gen_depth(&mut self, gen_depth: i32) {
        self.base.set_gen_depth(gen_depth);
    }

    fn orientation(&self) -> Option<Direction> {
        self.base.orientation()
    }

    fn set_orientation(&mut self, orientation: Option<Direction>) {
        self.base.set_orientation(orientation);
    }

    fn rotation(&self) -> Rotation {
        self.base.rotation()
    }

    fn mirror(&self) -> Mirror {
        self.base.mirror()
    }

    fn piece_type_id(&self) -> Identifier {
        self.base.piece_type_id()
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        self.base.add_additional_save_data(tag);
        // Rotation.LEGACY_CODEC — int ordinal.
        tag.put("Rot", Tag::Int(self.rotation().get_index() as i32));
    }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.base.load_additional_save_data(tag);
    }

    fn add_children(
        &mut self,
        piece: &mut dyn StructurePiece,
        pieces: &mut dyn StructurePieceAccessor,
        random: &mut dyn RandomSource,
    ) {
        self.base.add_children(piece, pieces, random);
    }

    #[allow(clippy::too_many_arguments)]
    fn post_process(
        &mut self,
        level: &dyn LevelReader,
        structure_manager: &dyn StructureManagerAccess,
        generator: Option<&dyn PoolGeneratorAccess>,
        random: &mut dyn RandomSource,
        box_: &BoundingBox,
        chunk_pos: ChunkPos,
        pos: BlockPos,
    ) {
        // Java: box.encapsulate(templateBB) до super.postProcess —
        // box у нас immutable, поэтому расширенный бокс передаём
        // в post_process_template (placeInWorld не клиппится им,
        // но handleDataMarker/финальный jigsaw-проход используют его).
        let template_bb = self
            .base
            .template()
            .get_bounding_box(&self.base.place_settings, self.base.template_position());
        let expanded = box_.encapsulate(&template_bb);
        self.base.post_process_template(
            level,
            structure_manager,
            generator,
            random,
            &expanded,
            chunk_pos,
            pos,
        );
        self.place_dried_ghast(level, &template_bb, &expanded);
    }
}

/// `makeSettings(rotation)`.
fn make_settings(rotation: Rotation) -> StructurePlaceSettings {
    let mut settings = StructurePlaceSettings::default();
    settings
        .set_rotation(rotation)
        .set_mirror(Mirror::NONE)
        .add_processor(BlockIgnoreProcessor::structure_and_air());
    settings
}

/// `NetherFossilStructure` — фабрика с `HeightProvider`.
pub fn nether_fossil_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
    height: HeightProvider,
) -> Structure {
    let mut structure = Structure::new(id, settings, structure_type);
    structure.find_generation_point =
        Some(Arc::new(move |context: &mut GenerationContext| -> Option<GenerationStub> {
            let worldgen_random = &mut context.random;
            let i = context.chunk_pos.get_min_block_x() + worldgen_random.next_int_bound(16);
            let i1 = context.chunk_pos.get_min_block_z() + worldgen_random.next_int_bound(16);
            let sea_level = context.level.sea_level();
            let gen_ctx = WorldGenerationContext::from_bounds(
                context.level.min_build_height(),
                context.level.height(),
                context.level.min_build_height(),
                context.level.height(),
            );
            let i2 = height.sample(worldgen_random, &gen_ctx);
            let base_column = context.level.get_base_column(i, i1);
            let mut y = i2;
            let mut mutable_pos = BlockPos::new(i, y, i1);

            while y > sea_level {
                let block = base_column.get_block(y);
                y -= 1;
                mutable_pos = BlockPos::new(i, y, i1);
                let block1 = base_column.get_block(y);
                let is_soul_sand = block1.block.id.path == "soul_sand";
                let sturdy = is_soul_sand
                    || context.level.is_face_sturdy(mutable_pos, Direction::UP);
                if block.is_air() && sturdy {
                    break;
                }
            }

            if y <= sea_level {
                return None;
            }
            let block_pos = BlockPos::new(i, y, i1);
            let template_manager = context.template_manager.clone()?;
            // Java: generator откладывается до stub — у нас pieces
            // генерируются eagerly (детерминированно, см. jigsaw).
            let piece = add_piece(&template_manager, worldgen_random, block_pos);
            Some(GenerationStub::new(block_pos, move |out| {
                out.push(Box::new(piece));
            }))
        }));
    structure
}
