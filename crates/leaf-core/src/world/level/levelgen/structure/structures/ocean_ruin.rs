//! Перенос `structures/OceanRuinStructure.java` +
//! `structures/OceanRuinPieces.java` — подводные руины
//! (warm/cold варианты, кластерная генерация, suspicious-блоки
//! через RuleProcessor+AppendLoot, маркеры chest/drowned,
//! подгонка высоты по OCEAN_FLOOR_WG).

use super::super::bounding_box::BoundingBox;
use super::super::structure::{
    on_top_of_chunk_center, GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{StructurePiece, StructurePieceAccessor};
use super::super::structure_type::StructureType;
use super::super::template_structure_piece::TemplateStructurePiece;
use super::super::templatesystem::pos_rule_tests::PosAlwaysTrueTest;
use super::super::templatesystem::processors::{
    BlockIgnoreProcessor, BlockRotProcessor, CappedProcessor, RuleProcessor,
    StructureProcessor,
};
use super::super::templatesystem::processor_rule::AppendLoot;
use super::super::templatesystem::rule_tests::{AlwaysTrueTest, BlockMatchTest, RuleTest};
use super::super::templatesystem::structure_place_settings::StructurePlaceSettings;
use super::super::templatesystem::structure_template::transform;
use super::super::templatesystem::structure_template_manager::StructureTemplateManager;
use super::super::pools::structure_pool_element::PoolGeneratorAccess;
use super::{block_state, with_bool_property};
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::world::entity::entity_spawn_reason::EntitySpawnReason;
use crate::entity::entity_type::entity_type_by_identifier;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::tags::block_tags::BlockTags;
use crate::util::mth::next_int;
use crate::util::valueproviders::constant_int::ConstantInt;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use std::rc::Rc;
use std::sync::Arc;

/// `OceanRuinStructure.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OceanRuinType {
    /// `WARM`.
    Warm,
    /// `COLD`.
    Cold,
}

impl OceanRuinType {
    /// `getSerializedName`.
    pub fn serialized_name(&self) -> &'static str {
        match self {
            OceanRuinType::Warm => "warm",
            OceanRuinType::Cold => "cold",
        }
    }
}

fn ruin_id(path: &str) -> Identifier {
    Identifier::with_default_namespace(path).unwrap_or_else(|_| Identifier::minecraft(path))
}

fn block_arc(id: &str) -> Arc<crate::world::level::block::Block> {
    crate::world::level::block::block_by_identifier(&Identifier::minecraft(id))
        .unwrap_or_else(|| Arc::new(crate::world::level::block::Block::new(Identifier::minecraft(id))))
}

/// `archyRuleProcessor(block, suspiciousBlock, lootTable)` —
/// RuleProcessor(BlockMatchTest → suspicious+AppendLoot) под
/// CappedProcessor(ConstantInt(5)).
fn archy_rule_processor(
    block: &str,
    suspicious_block: &str,
    loot_table: &str,
) -> Rc<dyn StructureProcessor> {
    let input: Rc<dyn RuleTest> = Rc::new(BlockMatchTest::new(block_arc(block)));
    let loc: Rc<dyn RuleTest> = Rc::new(AlwaysTrueTest);
    let pos = Rc::new(PosAlwaysTrueTest);
    let rule = super::super::templatesystem::processor_rule::ProcessorRule::with_modifier(
        input,
        loc,
        pos,
        block_state(suspicious_block),
        Rc::new(AppendLoot::new(Identifier::minecraft(loot_table))),
    );
    Rc::new(CappedProcessor::new(
        Rc::new(RuleProcessor::new(vec![rule])),
        Box::new(ConstantInt::of(5)),
    ))
}

fn warm_suspicious_processor() -> Rc<dyn StructureProcessor> {
    archy_rule_processor("sand", "suspicious_sand", "archaeology/ocean_ruin_warm")
}

fn cold_suspicious_processor() -> Rc<dyn StructureProcessor> {
    archy_rule_processor("gravel", "suspicious_gravel", "archaeology/ocean_ruin_cold")
}

const WARM_RUINS: [&str; 8] = [
    "underwater_ruin/warm_1", "underwater_ruin/warm_2", "underwater_ruin/warm_3",
    "underwater_ruin/warm_4", "underwater_ruin/warm_5", "underwater_ruin/warm_6",
    "underwater_ruin/warm_7", "underwater_ruin/warm_8",
];
const RUINS_BRICK: [&str; 8] = [
    "underwater_ruin/brick_1", "underwater_ruin/brick_2", "underwater_ruin/brick_3",
    "underwater_ruin/brick_4", "underwater_ruin/brick_5", "underwater_ruin/brick_6",
    "underwater_ruin/brick_7", "underwater_ruin/brick_8",
];
const RUINS_CRACKED: [&str; 8] = [
    "underwater_ruin/cracked_1", "underwater_ruin/cracked_2", "underwater_ruin/cracked_3",
    "underwater_ruin/cracked_4", "underwater_ruin/cracked_5", "underwater_ruin/cracked_6",
    "underwater_ruin/cracked_7", "underwater_ruin/cracked_8",
];
const RUINS_MOSSY: [&str; 8] = [
    "underwater_ruin/mossy_1", "underwater_ruin/mossy_2", "underwater_ruin/mossy_3",
    "underwater_ruin/mossy_4", "underwater_ruin/mossy_5", "underwater_ruin/mossy_6",
    "underwater_ruin/mossy_7", "underwater_ruin/mossy_8",
];
const BIG_RUINS_BRICK: [&str; 4] = [
    "underwater_ruin/big_brick_1", "underwater_ruin/big_brick_2",
    "underwater_ruin/big_brick_3", "underwater_ruin/big_brick_8",
];
const BIG_RUINS_MOSSY: [&str; 4] = [
    "underwater_ruin/big_mossy_1", "underwater_ruin/big_mossy_2",
    "underwater_ruin/big_mossy_3", "underwater_ruin/big_mossy_8",
];
const BIG_RUINS_CRACKED: [&str; 4] = [
    "underwater_ruin/big_cracked_1", "underwater_ruin/big_cracked_2",
    "underwater_ruin/big_cracked_3", "underwater_ruin/big_cracked_8",
];
const BIG_WARM_RUINS: [&str; 4] = [
    "underwater_ruin/big_warm_4", "underwater_ruin/big_warm_5",
    "underwater_ruin/big_warm_6", "underwater_ruin/big_warm_7",
];

/// `makeSettings(rotation, integrity, structureType)`.
fn make_settings(rotation: Rotation, integrity: f32, biome_type: OceanRuinType) -> StructurePlaceSettings {
    let processor = if biome_type == OceanRuinType::Cold {
        cold_suspicious_processor()
    } else {
        warm_suspicious_processor()
    };
    let mut settings = StructurePlaceSettings::default();
    settings
        .set_rotation(rotation)
        .set_mirror(Mirror::NONE)
        .add_processor(Rc::new(BlockRotProcessor::new(integrity)))
        .add_processor(BlockIgnoreProcessor::structure_and_air())
        .add_processor(processor);
    settings
}

/// `OceanRuinPieces.OceanRuinPiece`.
pub struct OceanRuinPiece {
    /// `TemplateStructurePiece`-часть.
    pub base: TemplateStructurePiece,
    /// `biomeType`.
    pub biome_type: OceanRuinType,
    /// `integrity`.
    pub integrity: f32,
    /// `isLarge`.
    pub is_large: bool,
}

impl OceanRuinPiece {
    /// `OceanRuinPiece(manager, location, pos, rotation, integrity,
    /// biomeType, isLarge)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        structure_template_manager: &StructureTemplateManager,
        location: &Identifier,
        pos: BlockPos,
        rotation: Rotation,
        integrity: f32,
        biome_type: OceanRuinType,
        is_large: bool,
    ) -> Self {
        let mut piece = Self {
            base: TemplateStructurePiece::new(
                Identifier::minecraft("ocean_ruin"),
                0,
                structure_template_manager,
                location,
                location.to_string(),
                make_settings(rotation, integrity, biome_type),
                pos,
            ),
            biome_type,
            integrity,
            is_large,
        };
        // `handleDataMarker` — chest/drowned.
        let is_large = piece.is_large;
        piece.base.data_marker_handler =
            Some(std::rc::Rc::new(move |name, pos, level, random, _box| {
                if name == "chest" {
                    // Ванильный chest с waterlogged по fluid.
                    let waterlogged = level
                        .get_fluid_state(pos)
                        .map(|f| f.is_water())
                        .unwrap_or(false);
                    let chest = with_bool_property(block_state("chest"), "waterlogged", waterlogged);
                    level.set_block_state(pos, &chest);
                    let loot = if is_large {
                        "chests/underwater_ruin_big"
                    } else {
                        "chests/underwater_ruin_small"
                    };
                    level.place_loot_container(pos, &Identifier::minecraft(loot), random.next_long());
                } else if name == "drowned" {
                    if let Some(entity_type) = entity_type_by_identifier(&Identifier::minecraft("drowned")) {
                        if let Some(mut mob) = level.create_mob_for_spawn(entity_type.as_ref()) {
                            mob.set_persistence_required(true);
                            mob.living.entity.snap_to(
                                pos.x as f64 + 0.5,
                                pos.y as f64,
                                pos.z as f64 + 0.5,
                                0.0,
                                0.0,
                            );
                            let difficulty = level.current_difficulty_at(pos);
                            mob.finalize_spawn(
                                level,
                                difficulty as i32 as f32,
                                EntitySpawnReason::Structure,
                                None,
                            );
                            level.try_add_fresh_entity_with_passengers(mob.living.entity.clone());
                            if pos.y > level.sea_level() {
                                let air = BlockState::air();
                                level.set_block_state(pos, &air);
                            } else {
                                let water = block_state("water");
                                level.set_block_state(pos, &water);
                            }
                        }
                    }
                }
            }));
        piece
    }

    /// `OceanRuinPiece.create(manager, tag)`.
    pub fn from_nbt(
        tag: &CompoundTag,
        structure_template_manager: &StructureTemplateManager,
    ) -> Self {
        let rot = tag.get_int("Rot").unwrap_or(0);
        let integrity = tag.get_float("Integrity").unwrap_or(0.0);
        let biome = match tag.get_string("BiomeType").unwrap_or("warm") {
            "cold" => OceanRuinType::Cold,
            _ => OceanRuinType::Warm,
        };
        let is_large = tag.get_bool("IsLarge").unwrap_or(false);
        Self {
            base: TemplateStructurePiece::from_nbt(
                Identifier::minecraft("ocean_ruin"),
                tag,
                structure_template_manager,
                move |_location| make_settings(Rotation::by_index(rot), integrity, biome),
            ),
            biome_type: biome,
            integrity,
            is_large,
        }
    }

    /// `getHeight(templatePos, level, pos)` — сканирование вниз от
    /// templatePos.y-1 до первого не-воздушного/не-водного/не-льда
    /// по каждой колонке прямоугольника.
    fn get_height(
        template_pos: BlockPos,
        level: &dyn LevelReader,
        pos: BlockPos,
    ) -> i32 {
        let mut y = template_pos.y;
        let mut i = 512i32;
        let i1 = y - 1;
        let mut i2 = 0i32;
        let ice_tag = BlockTags::ice();
        let end = pos;
        for p in BlockPos::between_closed(&template_pos, &end) {
            let x = p.x;
            let z = p.z;
            let mut i3 = template_pos.y - 1;
            let mut mp = BlockPos::new(x, i3, z);
            let mut state = level
                .get_block_state(mp)
                .unwrap_or_else(BlockState::air);
            let mut fluid = level.get_fluid_state(mp);
            while (state.is_air()
                || fluid.as_ref().map(|f| f.is_water()).unwrap_or(false)
                || state.is_named_tag(&ice_tag))
                && i3 > level.min_build_height() + 1
            {
                i3 -= 1;
                mp = BlockPos::new(x, i3, z);
                state = level
                    .get_block_state(mp)
                    .unwrap_or_else(BlockState::air);
                fluid = level.get_fluid_state(mp);
            }
            i = i.min(i3);
            if i3 < i1 - 2 {
                i2 += 1;
            }
        }
        let abs = (template_pos.x - end.x).abs();
        if i1 - i > 2 && i2 > abs - 2 {
            y = i + 1;
        }
        y
    }
}

impl StructurePiece for OceanRuinPiece {
    fn bounding_box(&self) -> BoundingBox {
        self.base.bounding_box()
    }

    fn move_by(&mut self, x: i32, y: i32, z: i32) {
        self.base.move_by(x, y, z);
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
        tag.put("Rot", Tag::Int(self.base.place_settings.get_rotation().get_index() as i32));
        tag.put("Integrity", Tag::Float(self.integrity));
        tag.put(
            "BiomeType",
            Tag::String(self.biome_type.serialized_name().to_string()),
        );
        tag.put("IsLarge", Tag::Byte(if self.is_large { 1 } else { 0 }));
    }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.base.load_additional_save_data(tag);
        self.integrity = tag.get_float("Integrity").unwrap_or(0.0);
        self.biome_type = match tag.get_string("BiomeType").unwrap_or("warm") {
            "cold" => OceanRuinType::Cold,
            _ => OceanRuinType::Warm,
        };
        self.is_large = tag.get_bool("IsLarge").unwrap_or(false);
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
        // Java postProcess: height по OCEAN_FLOOR_WG + getHeight-скан.
        let height = level.get_height(
            HeightmapType::OceanFloorWg,
            self.base.template_position.x,
            self.base.template_position.z,
        );
        self.base.template_position = BlockPos::new(
            self.base.template_position.x,
            height,
            self.base.template_position.z,
        );
        let size = self.base.template.get_size();
        let rel = transform(
            BlockPos::new(size.x - 1, 0, size.z - 1),
            Mirror::NONE,
            self.base.place_settings.get_rotation(),
            BlockPos::ZERO,
        );
        let block_pos = rel.offset(
            self.base.template_position.x,
            self.base.template_position.y,
            self.base.template_position.z,
        );
        let new_y = Self::get_height(self.base.template_position, level, block_pos);
        self.base.template_position = BlockPos::new(
            self.base.template_position.x,
            new_y,
            self.base.template_position.z,
        );
        self.base.post_process_template(level, structure_manager, generator, random, box_, chunk_pos, pos);
    }
}

fn pick_id(ids: &[&str], random: &mut dyn RandomSource) -> Identifier {
    let index = random.next_int_bound(ids.len() as i32) as usize;
    ruin_id(ids[index])
}

/// `addPiece` — warm: 1 шаблон; cold: 3 шаблона (brick/cracked/mossy
/// с одним индексом и integrity 1.0/0.7/0.5).
#[allow(clippy::too_many_arguments)]
fn add_piece(
    structure_template_manager: &StructureTemplateManager,
    pos: BlockPos,
    rotation: Rotation,
    pieces: &mut Vec<Box<dyn StructurePiece>>,
    random: &mut dyn RandomSource,
    biome_type: OceanRuinType,
    is_large: bool,
    integrity: f32,
) {
    match biome_type {
        OceanRuinType::Warm => {
            let id = if is_large {
                pick_id(&BIG_WARM_RUINS, random)
            } else {
                pick_id(&WARM_RUINS, random)
            };
            pieces.push(Box::new(OceanRuinPiece::new(
                structure_template_manager, &id, pos, rotation, integrity, biome_type, is_large,
            )));
        }
        OceanRuinType::Cold => {
            let brick = if is_large { &BIG_RUINS_BRICK[..] } else { &RUINS_BRICK[..] };
            let cracked = if is_large { &BIG_RUINS_CRACKED[..] } else { &RUINS_CRACKED[..] };
            let mossy = if is_large { &BIG_RUINS_MOSSY[..] } else { &RUINS_MOSSY[..] };
            let index = random.next_int_bound(brick.len() as i32) as usize;
            for (ids, integ) in [(brick, 1.0f32), (cracked, 0.7), (mossy, 0.5)] {
                let id = ruin_id(ids[index]);
                pieces.push(Box::new(OceanRuinPiece::new(
                    structure_template_manager, &id, pos, rotation, integ, biome_type, is_large,
                )));
            }
        }
    }
}

/// `allPositions` — 8 кандидатов вокруг pos.
fn all_positions(random: &mut dyn RandomSource, pos: BlockPos) -> Vec<BlockPos> {
    vec![
        pos.offset(-16 + next_int(random, 1, 8), 0, 16 + next_int(random, 1, 7)),
        pos.offset(-16 + next_int(random, 1, 8), 0, next_int(random, 1, 7)),
        pos.offset(-16 + next_int(random, 1, 8), 0, -16 + next_int(random, 4, 8)),
        pos.offset(next_int(random, 1, 7), 0, 16 + next_int(random, 1, 7)),
        pos.offset(next_int(random, 1, 7), 0, -16 + next_int(random, 4, 6)),
        pos.offset(16 + next_int(random, 1, 7), 0, 16 + next_int(random, 3, 8)),
        pos.offset(16 + next_int(random, 1, 7), 0, next_int(random, 1, 7)),
        pos.offset(16 + next_int(random, 1, 7), 0, -16 + next_int(random, 4, 8)),
    ]
}

/// `addClusterRuins`.
fn add_cluster_ruins(
    structure_template_manager: &StructureTemplateManager,
    random: &mut dyn RandomSource,
    rotation: Rotation,
    pos: BlockPos,
    biome_type: OceanRuinType,
    pieces: &mut Vec<Box<dyn StructurePiece>>,
) {
    let block_pos = BlockPos::new(pos.x, 90, pos.z);
    let rel = transform(BlockPos::new(15, 0, 15), Mirror::NONE, rotation, BlockPos::ZERO);
    let block_pos1 = rel.offset(block_pos.x, block_pos.y, block_pos.z);
    let bounding_box = BoundingBox::from_corners(block_pos.to_vec3i(), block_pos1.to_vec3i());
    let block_pos2 = BlockPos::new(
        block_pos.x.min(block_pos1.x),
        block_pos.y,
        block_pos.z.min(block_pos1.z),
    );
    let mut list = all_positions(random, block_pos2);
    let random_int = next_int(random, 4, 8);
    for _ in 0..random_int {
        if list.is_empty() {
            break;
        }
        let index = random.next_int_bound(list.len() as i32) as usize;
        let block_pos3 = list.remove(index);
        let random1 = Rotation::get_random(random);
        let rel2 = transform(BlockPos::new(5, 0, 6), Mirror::NONE, random1, BlockPos::ZERO);
        let block_pos4 = rel2.offset(block_pos3.x, block_pos3.y, block_pos3.z);
        let bounding_box1 = BoundingBox::from_corners(block_pos3.to_vec3i(), block_pos4.to_vec3i());
        if !bounding_box1.intersects(&bounding_box) {
            add_piece(
                structure_template_manager,
                block_pos3,
                random1,
                pieces,
                random,
                biome_type,
                false,
                0.8,
            );
        }
    }
}

/// `addPieces`.
fn add_pieces(
    structure_template_manager: &StructureTemplateManager,
    pos: BlockPos,
    rotation: Rotation,
    pieces: &mut Vec<Box<dyn StructurePiece>>,
    random: &mut dyn RandomSource,
    biome_type: OceanRuinType,
    large_probability: f32,
    cluster_probability: f32,
) {
    let is_large = random.next_float() <= large_probability;
    let integrity = if is_large { 0.9 } else { 0.8 };
    add_piece(
        structure_template_manager,
        pos,
        rotation,
        pieces,
        random,
        biome_type,
        is_large,
        integrity,
    );
    if is_large && random.next_float() <= cluster_probability {
        add_cluster_ruins(
            structure_template_manager,
            random,
            rotation,
            pos,
            biome_type,
            pieces,
        );
    }
}

/// `OceanRuinStructure` — `biome_temp`/`large_probability`/
/// `cluster_probability` + `findGenerationPoint`.
pub fn ocean_ruin_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
    biome_type: OceanRuinType,
    large_probability: f32,
    cluster_probability: f32,
) -> Structure {
    let mut structure = Structure::new(id, settings, structure_type);
    structure.find_generation_point =
        Some(Arc::new(move |context: &mut GenerationContext| -> Option<GenerationStub> {
            let chunk_pos = context.chunk_pos;
            let min_x = chunk_pos.get_min_block_x();
            let min_z = chunk_pos.get_min_block_z();
            let stub = on_top_of_chunk_center(context, HeightmapType::OceanFloorWg, move |pieces, context| {
                let Some(template_manager) = context.template_manager.clone() else {
                    return;
                };
                let rotation = Rotation::get_random(&mut context.random);
                let start_pos = BlockPos::new(min_x, 90, min_z);
                add_pieces(
                    &template_manager,
                    start_pos,
                    rotation,
                    pieces,
                    &mut context.random,
                    biome_type,
                    large_probability,
                    cluster_probability,
                );
            });
            Some(stub)
        }));
    structure
}
