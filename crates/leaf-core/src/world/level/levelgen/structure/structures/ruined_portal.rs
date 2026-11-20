//! Перенос `structures/RuinedPortalStructure.java` +
//! `structures/RuinedPortalPiece.java` — разрушенные порталы
//! (setups с весами, giant-порталы 5%, VerticalPlacement,
//! netherrack-спред, vines/leaves, suspicious-правила).

use super::super::bounding_box::BoundingBox;
use super::super::structure::{
    GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{
    get_random_horizontal_direction, StructurePiece, StructurePieceAccessor,
};
use super::super::structure_type::StructureType;
use super::super::template_structure_piece::TemplateStructurePiece;
use super::super::templatesystem::processors::{
    BlackstoneReplaceProcessor, BlockAgeProcessor, BlockIgnoreProcessor,
    LavaSubmergedBlockProcessor, ProtectedBlockProcessor, RuleProcessor,
    StructureProcessor,
};
use super::super::templatesystem::processor_rule::ProcessorRule;
use super::super::templatesystem::rule_tests::{
    AlwaysTrueTest, BlockMatchTest, RandomBlockMatchTest, RuleTest,
};
use super::super::templatesystem::structure_place_settings::StructurePlaceSettings;
use super::super::templatesystem::structure_template_manager::StructureTemplateManager;
use super::super::pools::structure_pool_element::PoolGeneratorAccess;
use super::{block_state, with_bool_property};
use crate::core::block_pos::BlockPos;
use crate::core::direction::{Axis, AxisDirection, Direction};
use crate::core::quart_pos;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::tags::block_tags::BlockTags;
use crate::util::mth::random_between_inclusive;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::noise_column::NoiseColumn;
use std::rc::Rc;
use std::sync::Arc;

const STRUCTURE_LOCATION_PORTALS: [&str; 10] = [
    "ruined_portal/portal_1", "ruined_portal/portal_2", "ruined_portal/portal_3",
    "ruined_portal/portal_4", "ruined_portal/portal_5", "ruined_portal/portal_6",
    "ruined_portal/portal_7", "ruined_portal/portal_8", "ruined_portal/portal_9",
    "ruined_portal/portal_10",
];
const STRUCTURE_LOCATION_GIANT_PORTALS: [&str; 3] = [
    "ruined_portal/giant_portal_1", "ruined_portal/giant_portal_2",
    "ruined_portal/giant_portal_3",
];
const PROBABILITY_OF_GIANT_PORTAL: f32 = 0.05;

fn portal_id(path: &str) -> Identifier {
    Identifier::with_default_namespace(path).unwrap_or_else(|_| Identifier::minecraft(path))
}

fn block_arc(path: &str) -> Arc<crate::world::level::block::Block> {
    crate::world::level::block::block_by_identifier(&Identifier::minecraft(path))
        .unwrap_or_else(|| Arc::new(crate::world::level::block::Block::new(Identifier::minecraft(path))))
}

/// `VerticalPlacement`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalPlacement {
    /// `on_land_surface`.
    OnLandSurface,
    /// `partly_buried`.
    PartlyBuried,
    /// `on_ocean_floor`.
    OnOceanFloor,
    /// `in_mountain`.
    InMountain,
    /// `underground`.
    Underground,
    /// `in_nether`.
    InNether,
}

impl VerticalPlacement {
    /// `getSerializedName`.
    pub fn serialized_name(&self) -> &'static str {
        match self {
            VerticalPlacement::OnLandSurface => "on_land_surface",
            VerticalPlacement::PartlyBuried => "partly_buried",
            VerticalPlacement::OnOceanFloor => "on_ocean_floor",
            VerticalPlacement::InMountain => "in_mountain",
            VerticalPlacement::Underground => "underground",
            VerticalPlacement::InNether => "in_nether",
        }
    }

    /// `byName`.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "on_land_surface" => Some(VerticalPlacement::OnLandSurface),
            "partly_buried" => Some(VerticalPlacement::PartlyBuried),
            "on_ocean_floor" => Some(VerticalPlacement::OnOceanFloor),
            "in_mountain" => Some(VerticalPlacement::InMountain),
            "underground" => Some(VerticalPlacement::Underground),
            "in_nether" => Some(VerticalPlacement::InNether),
            _ => None,
        }
    }
}

/// `RuinedPortalPiece.Properties`.
#[derive(Clone, Copy, Debug)]
pub struct PortalProperties {
    /// `cold`.
    pub cold: bool,
    /// `mossiness`.
    pub mossiness: f32,
    /// `airPocket`.
    pub air_pocket: bool,
    /// `overgrown`.
    pub overgrown: bool,
    /// `vines`.
    pub vines: bool,
    /// `replaceWithBlackstone`.
    pub replace_with_blackstone: bool,
}

impl PortalProperties {
    /// Загрузка из compound-тега (`Properties` codec).
    fn load(tag: &CompoundTag) -> Self {
        Self {
            cold: tag.get_bool("cold").unwrap_or(false),
            mossiness: tag.get_float("mossiness").unwrap_or(0.0),
            air_pocket: tag.get_bool("air_pocket").unwrap_or(false),
            overgrown: tag.get_bool("overgrown").unwrap_or(false),
            vines: tag.get_bool("vines").unwrap_or(false),
            replace_with_blackstone: tag.get_bool("replace_with_blackstone").unwrap_or(false),
        }
    }

    /// Сохранение в compound-тег.
    fn save(&self) -> CompoundTag {
        let mut tag = CompoundTag::new();
        tag.put("cold", Tag::Byte(if self.cold { 1 } else { 0 }));
        tag.put("mossiness", Tag::Float(self.mossiness));
        tag.put("air_pocket", Tag::Byte(if self.air_pocket { 1 } else { 0 }));
        tag.put("overgrown", Tag::Byte(if self.overgrown { 1 } else { 0 }));
        tag.put("vines", Tag::Byte(if self.vines { 1 } else { 0 }));
        tag.put(
            "replace_with_blackstone",
            Tag::Byte(if self.replace_with_blackstone { 1 } else { 0 }),
        );
        tag
    }
}

/// `RuinedPortalStructure.Setup`.
#[derive(Clone, Copy, Debug)]
pub struct PortalSetup {
    /// `placement`.
    pub placement: VerticalPlacement,
    /// `airPocketProbability`.
    pub air_pocket_probability: f32,
    /// `mossiness`.
    pub mossiness: f32,
    /// `overgrown`.
    pub overgrown: bool,
    /// `vines`.
    pub vines: bool,
    /// `canBeCold`.
    pub can_be_cold: bool,
    /// `replaceWithBlackstone`.
    pub replace_with_blackstone: bool,
    /// `weight`.
    pub weight: f32,
}

/// `state.is(block)` по id-пути.
fn state_is(state: &BlockState, path: &str) -> bool {
    state.block.id.path == path
}

/// `Block.isFaceFull(collisionShape, direction)` — аппроксимация:
/// face считается полной, если хотя бы один AABB формы покрывает
/// грань целиком (для single-box форм эквивалентно Java).
fn is_face_full(state: &BlockState, level: &dyn LevelReader, pos: BlockPos, dir: Direction) -> bool {
    let shape = state.get_collision_shape(level, pos);
    if shape.is_empty() {
        return false;
    }
    let axis = dir.get_axis();
    let (u, v) = match axis {
        Axis::X => (Axis::Y, Axis::Z),
        Axis::Y => (Axis::X, Axis::Z),
        Axis::Z => (Axis::X, Axis::Y),
    };
    shape.to_aabbs().iter().any(|aabb| {
        let face_ok = match dir.get_axis_direction() {
            AxisDirection::POSITIVE => aabb.max_axis(axis) >= 1.0 - 1e-7,
            AxisDirection::NEGATIVE => aabb.min_axis(axis) <= 1e-7,
        };
        face_ok
            && aabb.min_axis(u) <= 1e-7
            && aabb.max_axis(u) >= 1.0 - 1e-7
            && aabb.min_axis(v) <= 1e-7
            && aabb.max_axis(v) >= 1.0 - 1e-7
    })
}

/// `getHeightMapType(verticalPlacement)`.
pub fn get_height_map_type(vp: VerticalPlacement) -> HeightmapType {
    if vp == VerticalPlacement::OnOceanFloor {
        HeightmapType::OceanFloorWg
    } else {
        HeightmapType::WorldSurfaceWg
    }
}

/// `getBlockReplaceRule(block, probability, replaceBlock)`.
fn replace_rule_prob(block: &str, probability: f32, replace: &str) -> ProcessorRule {
    let input: Rc<dyn RuleTest> =
        Rc::new(RandomBlockMatchTest::new(block_arc(block), probability));
    let loc: Rc<dyn RuleTest> = Rc::new(AlwaysTrueTest);
    ProcessorRule::new(input, loc, block_state(replace))
}

/// `getBlockReplaceRule(block, replaceBlock)`.
fn replace_rule(block: &str, replace: &str) -> ProcessorRule {
    let input: Rc<dyn RuleTest> = Rc::new(BlockMatchTest::new(block_arc(block)));
    let loc: Rc<dyn RuleTest> = Rc::new(AlwaysTrueTest);
    ProcessorRule::new(input, loc, block_state(replace))
}

/// `getLavaProcessorRule`.
fn lava_rule(vp: VerticalPlacement, properties: &PortalProperties) -> ProcessorRule {
    if vp == VerticalPlacement::OnOceanFloor {
        replace_rule("lava", "magma_block")
    } else if properties.cold {
        replace_rule("lava", "netherrack")
    } else {
        replace_rule_prob("lava", 0.2, "magma_block")
    }
}

/// `makeSettings(mirror, rotation, verticalPlacement, pos, properties)`.
fn make_settings(
    mirror: Mirror,
    rotation: Rotation,
    vertical_placement: VerticalPlacement,
    pos: BlockPos,
    properties: &PortalProperties,
) -> StructurePlaceSettings {
    let processor: Rc<dyn StructureProcessor> = if properties.air_pocket {
        BlockIgnoreProcessor::structure_block()
    } else {
        BlockIgnoreProcessor::structure_and_air()
    };
    let mut rules = vec![
        replace_rule_prob("gold_block", 0.3, "air"),
        lava_rule(vertical_placement, properties),
    ];
    if !properties.cold {
        rules.push(replace_rule_prob("netherrack", 0.07, "magma_block"));
    }
    let mut settings = StructurePlaceSettings::default();
    settings
        .set_rotation(rotation)
        .set_mirror(mirror)
        .set_rotation_pivot(pos)
        .add_processor(processor)
        .add_processor(Rc::new(RuleProcessor::new(rules)))
        .add_processor(Rc::new(BlockAgeProcessor::new(properties.mossiness)))
        .add_processor(Rc::new(ProtectedBlockProcessor {
            cannot_replace: BlockTags::features_cannot_replace(),
        }))
        .add_processor(Rc::new(LavaSubmergedBlockProcessor));
    if properties.replace_with_blackstone {
        settings.add_processor(Rc::new(BlackstoneReplaceProcessor));
    }
    settings
}

/// `RuinedPortalPiece`.
pub struct RuinedPortalPiece {
    /// `TemplateStructurePiece`-часть.
    pub base: TemplateStructurePiece,
    /// `verticalPlacement`.
    pub vertical_placement: VerticalPlacement,
    /// `properties`.
    pub properties: PortalProperties,
}

impl RuinedPortalPiece {
    /// `RuinedPortalPiece(manager, templatePosition, verticalPlacement,
    /// properties, location, template, rotation, mirror, pivotPos)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        structure_template_manager: &StructureTemplateManager,
        template_position: BlockPos,
        vertical_placement: VerticalPlacement,
        properties: PortalProperties,
        location: &Identifier,
        rotation: Rotation,
        mirror: Mirror,
        pivot_pos: BlockPos,
    ) -> Self {
        // Java конструктор получает template и pivot = size/2 —
        // pivot вычисляется по шаблону внутри makeSettings.
        let template = structure_template_manager.get_or_create(location);
        let size = template.get_size();
        let pivot = BlockPos::new(size.x / 2, 0, size.z / 2);
        let _ = pivot_pos;
        Self {
            base: TemplateStructurePiece::new(
                Identifier::minecraft("ruined_portal"),
                0,
                structure_template_manager,
                location,
                location.to_string(),
                make_settings(mirror, rotation, vertical_placement, pivot, &properties),
                template_position,
            ),
            vertical_placement,
            properties,
        }
    }

    /// `RuinedPortalPiece(manager, tag)` — загрузка из NBT.
    pub fn from_nbt(
        tag: &CompoundTag,
        structure_template_manager: &StructureTemplateManager,
    ) -> Self {
        let mirror = Mirror::VALUES
            .get(tag.get_int("Mirror").unwrap_or(0).rem_euclid(3) as usize)
            .copied()
            .unwrap_or(Mirror::NONE);
        let rotation = Rotation::by_index(tag.get_int("Rotation").unwrap_or(0));
        let vertical_placement = VerticalPlacement::by_name(
            tag.get_string("VerticalPlacement").unwrap_or("on_land_surface"),
        )
        .unwrap_or(VerticalPlacement::OnLandSurface);
        let properties = tag
            .get_compound("Properties")
            .map(PortalProperties::load)
            .unwrap_or(PortalProperties {
                cold: false,
                mossiness: 0.0,
                air_pocket: false,
                overgrown: false,
                vines: false,
                replace_with_blackstone: false,
            });
        Self {
            base: TemplateStructurePiece::from_nbt(
                Identifier::minecraft("ruined_portal"),
                tag,
                structure_template_manager,
                move |location| {
                    let template = structure_template_manager.get_or_create(location);
                    let size = template.get_size();
                    let pivot = BlockPos::new(size.x / 2, 0, size.z / 2);
                    make_settings(mirror, rotation, vertical_placement, pivot, &properties)
                },
            ),
            vertical_placement,
            properties,
        }
    }

    /// `maybeAddVines`.
    fn maybe_add_vines(&self, random: &mut dyn RandomSource, level: &dyn LevelReader, pos: BlockPos) {
        let Some(state) = level.get_block_state(pos) else { return };
        if !state.is_air() && state.block.id.path != "vine" {
            let dir = get_random_horizontal_direction(random);
            let neighbor = pos.relative(dir);
            let Some(neighbor_state) = level.get_block_state(neighbor) else { return };
            if neighbor_state.is_air() && is_face_full(&state, level, pos, dir) {
                // `VineBlock.getPropertyForFace(dir.getOpposite())`.
                let prop = dir.get_opposite().get_serialized_name();
                let vine = with_bool_property(block_state("vine"), prop, true);
                level.set_block_state(neighbor, &vine);
            }
        }
    }

    /// `maybeAddLeavesAbove`.
    fn maybe_add_leaves_above(&self, random: &mut dyn RandomSource, level: &dyn LevelReader, pos: BlockPos) {
        let above = pos.above();
        let is_netherrack = level
            .get_block_state(pos)
            .map(|s| state_is(&s, "netherrack"))
            .unwrap_or(false);
        let is_air = level
            .get_block_state(above)
            .map(|s| s.is_air())
            .unwrap_or(false);
        if random.next_float() < 0.5 && is_netherrack && is_air {
            let leaves = with_bool_property(block_state("jungle_leaves"), "persistent", true);
            level.set_block_state(above, &leaves);
        }
    }

    /// `addNetherrackDripColumnsBelowPortal`.
    fn add_netherrack_drip_columns_below_portal(&self, random: &mut dyn RandomSource, level: &dyn LevelReader) {
        let bb = &self.base.bounding_box;
        for i in (bb.min_x + 1)..bb.max_x {
            for i1 in (bb.min_z + 1)..bb.max_z {
                let pos = BlockPos::new(i, bb.min_y, i1);
                let is_netherrack = level
                    .get_block_state(pos)
                    .map(|s| state_is(&s, "netherrack"))
                    .unwrap_or(false);
                if is_netherrack {
                    self.add_netherrack_drip_column(random, level, pos.below());
                }
            }
        }
    }

    /// `addNetherrackDripColumn`.
    fn add_netherrack_drip_column(&self, random: &mut dyn RandomSource, level: &dyn LevelReader, pos: BlockPos) {
        let mut current = pos;
        self.place_netherrack_or_magma(random, level, current);
        let mut i = 8;
        while i > 0 && random.next_float() < 0.5 {
            current = current.below();
            i -= 1;
            self.place_netherrack_or_magma(random, level, current);
        }
    }

    /// `spreadNetherrack`.
    fn spread_netherrack(&self, random: &mut dyn RandomSource, level: &dyn LevelReader) {
        let flag = self.vertical_placement == VerticalPlacement::OnLandSurface
            || self.vertical_placement == VerticalPlacement::OnOceanFloor;
        let center = self.base.bounding_box.get_center();
        let (x, z) = (center.x, center.z);
        let floats: [f32; 14] = [
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.9, 0.9, 0.8, 0.7, 0.6, 0.4, 0.2,
        ];
        let i = floats.len() as i32;
        let i1 = (self.base.bounding_box.get_x_span() + self.base.bounding_box.get_z_span()) / 2;
        let random_int = random.next_int_bound((8 - i1 / 2).max(1));
        let bb = self.base.bounding_box.clone();
        for i3 in (x - i)..=(x + i) {
            for i4 in (z - i)..=(z + i) {
                let i5 = (i3 - x).abs() + (i4 - z).abs();
                let max = (i5 + random_int).max(0);
                if max < i {
                    let f = floats[max as usize];
                    if random.next_double() < f as f64 {
                        let surface_y = Self::get_surface_y(level, i3, i4, self.vertical_placement);
                        let i6 = if flag {
                            surface_y
                        } else {
                            bb.min_y.min(surface_y)
                        };
                        let pos = BlockPos::new(i3, i6, i4);
                        if (i6 - bb.min_y).abs() <= 3
                            && self.can_block_be_replaced_by_netherrack_or_magma(level, pos)
                        {
                            self.place_netherrack_or_magma(random, level, pos);
                            if self.properties.overgrown {
                                self.maybe_add_leaves_above(random, level, pos);
                            }
                            self.add_netherrack_drip_column(random, level, pos.below());
                        }
                    }
                }
            }
        }
    }

    /// `canBlockBeReplacedByNetherrackOrMagma`.
    fn can_block_be_replaced_by_netherrack_or_magma(&self, level: &dyn LevelReader, pos: BlockPos) -> bool {
        let Some(state) = level.get_block_state(pos) else { return false };
        !state.is_air()
            && !state_is(&state, "obsidian")
            && !state.is_named_tag(&BlockTags::features_cannot_replace())
            && (self.vertical_placement == VerticalPlacement::InNether
                || !state_is(&state, "lava"))
    }

    /// `placeNetherrackOrMagma`.
    fn place_netherrack_or_magma(&self, random: &mut dyn RandomSource, level: &dyn LevelReader, pos: BlockPos) {
        if !self.properties.cold && random.next_float() < 0.07 {
            level.set_block_state(pos, &block_state("magma_block"));
        } else {
            level.set_block_state(pos, &block_state("netherrack"));
        }
    }

    /// `getSurfaceY`.
    fn get_surface_y(level: &dyn LevelReader, x: i32, z: i32, vp: VerticalPlacement) -> i32 {
        level.get_height(get_height_map_type(vp), x, z) - 1
    }
}

impl StructurePiece for RuinedPortalPiece {
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
        tag.put("Rotation", Tag::Int(self.base.place_settings.get_rotation().get_index() as i32));
        let mirror_index = Mirror::VALUES
            .iter()
            .position(|m| *m == self.base.place_settings.get_mirror())
            .unwrap_or(0) as i32;
        tag.put("Mirror", Tag::Int(mirror_index));
        tag.put(
            "VerticalPlacement",
            Tag::String(self.vertical_placement.serialized_name().to_string()),
        );
        tag.put("Properties", Tag::Compound(self.properties.save()));
    }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.base.load_additional_save_data(tag);
        self.vertical_placement = VerticalPlacement::by_name(
            tag.get_string("VerticalPlacement").unwrap_or("on_land_surface"),
        )
        .unwrap_or(VerticalPlacement::OnLandSurface);
        self.properties = tag
            .get_compound("Properties")
            .map(PortalProperties::load)
            .unwrap_or(self.properties);
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
        let bounding_box = self
            .base
            .template
            .get_bounding_box(&self.base.place_settings, self.base.template_position);
        if !box_.is_inside(bounding_box.get_center().x, bounding_box.get_center().y, bounding_box.get_center().z) {
            return;
        }
        // Java: `box.encapsulate(boundingBox)` — расширяет chunk-box;
        // у нас box неизменяем, поэтому расширяем локально для
        // post_process_template (семантика для этого куска та же).
        let expanded_box = box_.encapsulate(&bounding_box);
        self.base.post_process_template(
            level, structure_manager, generator, random, &expanded_box, chunk_pos, pos,
        );
        self.spread_netherrack(random, level);
        self.add_netherrack_drip_columns_below_portal(random, level);
        if self.properties.vines || self.properties.overgrown {
            let bb = self.base.bounding_box.clone();
            for x in bb.min_x..=bb.max_x {
                for y in bb.min_y..=bb.max_y {
                    for z in bb.min_z..=bb.max_z {
                        let p = BlockPos::new(x, y, z);
                        if self.properties.vines {
                            self.maybe_add_vines(random, level, p);
                        }
                        if self.properties.overgrown {
                            self.maybe_add_leaves_above(random, level, p);
                        }
                    }
                }
            }
        }
    }
}

/// `sample(random, threshold)`.
fn sample(random: &mut dyn RandomSource, threshold: f32) -> bool {
    threshold != 0.0 && (threshold == 1.0 || random.next_float() < threshold)
}

/// `isCold(pos, biome, seaLevel)`.
fn is_cold(pos: BlockPos, biome: &crate::world::level::biome::biome::Biome, sea_level: i32) -> bool {
    biome.cold_enough_to_snow(pos, sea_level)
}

/// `getRandomWithinInterval`.
fn get_random_within_interval(random: &mut dyn RandomSource, min: i32, max: i32) -> i32 {
    if min < max {
        random_between_inclusive(random, min, max)
    } else {
        max
    }
}

/// `findSuitableY`.
#[allow(clippy::too_many_arguments)]
fn find_suitable_y(
    random: &mut dyn RandomSource,
    level: &dyn LevelReader,
    vertical_placement: VerticalPlacement,
    air_pocket: bool,
    height: i32,
    block_count_y: i32,
    box_: &BoundingBox,
) -> i32 {
    let i = level.min_build_height() + 15;
    let i1 = if vertical_placement == VerticalPlacement::InNether {
        if air_pocket {
            random_between_inclusive(random, 32, 100)
        } else if random.next_float() < 0.5 {
            random_between_inclusive(random, 27, 29)
        } else {
            random_between_inclusive(random, 29, 100)
        }
    } else if vertical_placement == VerticalPlacement::InMountain {
        let i2 = height - block_count_y;
        get_random_within_interval(random, 70, i2)
    } else if vertical_placement == VerticalPlacement::Underground {
        let i2 = height - block_count_y;
        get_random_within_interval(random, i, i2)
    } else if vertical_placement == VerticalPlacement::PartlyBuried {
        height - block_count_y + random_between_inclusive(random, 2, 8)
    } else {
        height
    };

    let corners = [
        (box_.min_x, box_.min_z),
        (box_.max_x, box_.min_z),
        (box_.min_x, box_.max_z),
        (box_.max_x, box_.max_z),
    ];
    let columns: Vec<NoiseColumn> = corners
        .iter()
        .map(|&(x, z)| level.get_base_column(x, z))
        .collect();
    let heightmap = get_height_map_type(vertical_placement);
    let is_opaque = heightmap.is_opaque();

    let mut i3 = i1;
    while i3 > i {
        let mut count = 0;
        for column in &columns {
            let block = column.get_block(i3);
            // `types.isOpaque().test(block)` — opaque-предикат
            // heightmap-типа (для WG-типов — solid/can_occlude).
            let opaque = if is_opaque {
                block.can_occlude()
            } else {
                block.blocks_motion()
            };
            if opaque {
                count += 1;
                if count == 3 {
                    return i3;
                }
            }
        }
        i3 -= 1;
    }
    i3
}

/// `RuinedPortalStructure` — setups + findGenerationPoint.
pub fn ruined_portal_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
    setups: Vec<PortalSetup>,
) -> Structure {
    let mut structure = Structure::new(id, settings, structure_type);
    let setups = Arc::new(setups);
    structure.find_generation_point =
        Some(Arc::new(move |context: &mut GenerationContext| -> Option<GenerationStub> {
            // Weighted-выбор setup.
            let mut setup: Option<PortalSetup> = None;
            if setups.len() > 1 {
                let total: f32 = setups.iter().map(|s| s.weight).sum();
                let mut random_float = context.random.next_float();
                for s in setups.iter() {
                    random_float -= s.weight / total;
                    if random_float < 0.0 {
                        setup = Some(*s);
                        break;
                    }
                }
            } else {
                setup = setups.first().copied();
            }
            let setup = setup?;

            let mut properties = PortalProperties {
                cold: false,
                mossiness: setup.mossiness,
                air_pocket: sample(&mut context.random, setup.air_pocket_probability),
                overgrown: setup.overgrown,
                vines: setup.vines,
                replace_with_blackstone: setup.replace_with_blackstone,
            };
            let identifier = if context.random.next_float() < PROBABILITY_OF_GIANT_PORTAL {
                let idx = context
                    .random
                    .next_int_bound(STRUCTURE_LOCATION_GIANT_PORTALS.len() as i32)
                    as usize;
                portal_id(STRUCTURE_LOCATION_GIANT_PORTALS[idx])
            } else {
                let idx = context
                    .random
                    .next_int_bound(STRUCTURE_LOCATION_PORTALS.len() as i32)
                    as usize;
                portal_id(STRUCTURE_LOCATION_PORTALS[idx])
            };
            let template_manager = context.template_manager.clone()?;
            let template = template_manager.get_or_create(&identifier);
            let rotation = Rotation::get_random(&mut context.random);
            let mirror = if context.random.next_float() < 0.5 {
                Mirror::NONE
            } else {
                Mirror::FRONT_BACK
            };
            let size = template.get_size();
            let pivot = BlockPos::new(size.x / 2, 0, size.z / 2);
            let world_position = context.chunk_pos.get_world_position();
            let bounding_box = template.get_bounding_box_full(
                world_position,
                rotation,
                pivot,
                mirror,
            );
            let center = bounding_box.get_center();
            let heightmap = get_height_map_type(setup.placement);
            let i = context.level.get_base_height(heightmap, center.x, center.z) - 1;
            let i1 = find_suitable_y(
                &mut context.random,
                context.level,
                setup.placement,
                properties.air_pocket,
                i,
                bounding_box.get_y_span(),
                &bounding_box,
            );
            let block_pos1 = BlockPos::new(world_position.x, i1, world_position.z);
            properties.cold = false; // выставляется в генераторе при canBeCold
            Some(GenerationStub::new(block_pos1, move |pieces, context| {
                let mut props = properties;
                if setup.can_be_cold {
                    let qx = quart_pos::from_block(block_pos1.x);
                    let qy = quart_pos::from_block(block_pos1.y);
                    let qz = quart_pos::from_block(block_pos1.z);
                    if let Some(holder) = context.level.noise_biome_at(qx, qy, qz) {
                        if let Some(biome) = holder.value() {
                            props.cold = is_cold(block_pos1, biome, context.level.sea_level());
                        }
                    }
                }
                if let Some(tm) = context.template_manager.clone() {
                    pieces.push(Box::new(RuinedPortalPiece::new(
                        &tm,
                        block_pos1,
                        setup.placement,
                        props,
                        &identifier,
                        rotation,
                        mirror,
                        pivot,
                    )));
                }
            }))
        }));
    structure
}
