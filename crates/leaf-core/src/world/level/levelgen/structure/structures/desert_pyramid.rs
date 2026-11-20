//! Перенос `structures/DesertPyramidStructure.java` +
//! `structures/DesertPyramidPiece.java` — пустынная пирамида
//! (21×15×21, ScatteredFeaturePiece, подозрительный песок в
//! подвале + 4 сундука).

use super::super::bounding_box::BoundingBox;
use super::super::pieces_container::PiecesContainer;
use super::super::scattered_feature_piece::ScatteredFeaturePieceData;
use super::super::single_piece_structure::single_piece_structure;
use super::super::structure::{
    GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{
    create_chest, fill_column_down, generate_box, place_block,
    state_rotate, piece_set_orientation, StructurePiece, StructurePieceAccessor,
};
use super::super::structure_type::StructureType;
use super::{block_state, with_string_property};
use crate::core::block_pos::BlockPos;
use crate::core::direction::{Direction, Plane};
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::util::shuffle;
use crate::util::mth::random_between_inclusive;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess;
use std::collections::BTreeSet;
use std::sync::Arc;

/// `DesertPyramidPiece` — `ScatteredFeaturePiece(DESERT_PYRAMID_PIECE,
/// x, 64, z, 21, 15, 21, randomHorizontalDirection)`.
pub struct DesertPyramidPiece {
    scattered: ScatteredFeaturePieceData,
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    /// `hasPlacedChest[4]` — по `get2DDataValue` направления.
    has_placed_chest: [bool; 4],
    /// `potentialSuspiciousSandWorldPositions`.
    potential_suspicious_sand_world_positions: Vec<BlockPos>,
    /// `randomCollapsedRoofPos`.
    random_collapsed_roof_pos: BlockPos,
}

impl DesertPyramidPiece {
    /// `new(random, x, z)`.
    pub fn new(random: &mut dyn RandomSource, x: i32, z: i32) -> Self {
        let direction = super::super::structure_piece::get_random_horizontal_direction(random);
        let (scattered, bounding_box) =
            ScatteredFeaturePieceData::new(x, 64, z, 21, 15, 21, direction);
        Self {
            scattered,
            bounding_box,
            gen_depth: 0,
            orientation: Some(direction),
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            has_placed_chest: [false; 4],
            potential_suspicious_sand_world_positions: Vec::new(),
            random_collapsed_roof_pos: BlockPos::ZERO,
        }
    }

    /// `new(tag)` — загрузка из NBT.
    pub fn from_nbt(tag: &CompoundTag) -> Self {
        let scattered = ScatteredFeaturePieceData::load(tag);
        let direction = Direction::by_name(tag.get_string("O").unwrap_or("north"))
            .unwrap_or(Direction::NORTH);
        let mut piece = Self {
            scattered,
            bounding_box: BoundingBox::new(
                tag.get_int("TPX").unwrap_or(0),
                tag.get_int("TPY").unwrap_or(0),
                tag.get_int("TPZ").unwrap_or(0),
                tag.get_int("TPX").unwrap_or(0) + 21 - 1,
                tag.get_int("TPY").unwrap_or(0) + 15 - 1,
                tag.get_int("TPZ").unwrap_or(0) + 21 - 1,
            ),
            gen_depth: 0,
            orientation: Some(direction),
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            has_placed_chest: [
                tag.get_bool("hasPlacedChest0").unwrap_or(false),
                tag.get_bool("hasPlacedChest1").unwrap_or(false),
                tag.get_bool("hasPlacedChest2").unwrap_or(false),
                tag.get_bool("hasPlacedChest3").unwrap_or(false),
            ],
            potential_suspicious_sand_world_positions: Vec::new(),
            random_collapsed_roof_pos: BlockPos::ZERO,
        };
        piece_set_orientation(Some(direction), &mut piece.mirror, &mut piece.rotation);
        piece
    }

    /// `getWorldPos(x, y, z)`.
    fn world_pos(&self, x: i32, y: i32, z: i32) -> BlockPos {
        super::super::structure_piece::PieceTransform {
            bb: &self.bounding_box,
            orientation: self.orientation,
        }
        .world_pos(x, y, z)
    }

    /// `placeSand` — регистрирует позицию подозрительного песка.
    fn place_sand(&mut self, x: i32, y: i32, z: i32) {
        let world_pos = self.world_pos(x, y, z);
        self.potential_suspicious_sand_world_positions.push(world_pos);
    }

    /// `placeSandBox`.
    fn place_sand_box(&mut self, min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    self.place_sand(x, y, z);
                }
            }
        }
    }

    /// `placeCollapsedRoofPiece` — sandstone (1/3) или sand.
    fn place_collapsed_roof_piece(
        &self,
        level: &dyn LevelReader,
        bb: &BoundingBox,
        orientation: Option<Direction>,
        x: i32,
        y: i32,
        z: i32,
        box_: &BoundingBox,
    ) {
        let state = if level.level_random_next_float() < 0.33 {
            block_state("sandstone")
        } else {
            block_state("sand")
        };
        place_block(level, bb, orientation, self.mirror, self.rotation, state, x, y, z, box_);
    }

    /// `placeCollapsedRoof` — крыша подвала + выбор
    /// `randomCollapsedRoofPos` через positional random.
    fn place_collapsed_roof(
        &mut self,
        level: &dyn LevelReader,
        box_: &BoundingBox,
        min_x: i32,
        y: i32,
        min_z: i32,
        max_x: i32,
        max_z: i32,
    ) {
        let bb = self.bounding_box.clone();
        let orient = self.orientation;
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                self.place_collapsed_roof_piece(level, &bb, orient, x, y, z, box_);
            }
        }
        let seed = level.world_seed();
        let corner = self.world_pos(min_x, y, min_z);
        let mut random_source = crate::random::legacy::LegacyRandomSource::new(seed)
            .fork_positional()
            .at(corner.x, corner.y, corner.z);
        let rx = random_between_inclusive(random_source.as_mut(), min_x, max_x);
        let rz = random_between_inclusive(random_source.as_mut(), min_z, max_z);
        self.random_collapsed_roof_pos = self.world_pos(rx, y, rz);
    }

    /// `addCellar` — лестница + комната подвала.
    fn add_cellar(&mut self, level: &dyn LevelReader, box_: &BoundingBox) {
        let pos = BlockPos::new(16, -4, 13);
        self.add_cellar_stairs(level, pos, box_);
        self.add_cellar_room(level, pos, box_);
    }

    /// `addCellarStairs`.
    fn add_cellar_stairs(&mut self, level: &dyn LevelReader, pos: BlockPos, box_: &BoundingBox) {
        let (x, y, z) = (pos.x, pos.y, pos.z);
        let bb = self.bounding_box.clone();
        let orient = self.orientation;
        let mirror = self.mirror;
        let rotation = self.rotation;
        let stairs = block_state("sandstone_stairs");
        let stairs_ccw = state_rotate(stairs.clone(), Rotation::COUNTERCLOCKWISE_90);
        place_block(level, &bb, orient, mirror, rotation, stairs_ccw.clone(), 13, -1, 17, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_ccw.clone(), 14, -2, 17, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_ccw, 15, -3, 17, box_);
        let sand = block_state("sand");
        let sandstone = block_state("sandstone");
        let random_boolean = level.level_random_next_int(2) == 0;
        for dx in -4..=1 {
            place_block(level, &bb, orient, mirror, rotation, sand.clone(), x + dx, y + 4, z + 4, box_);
        }
        place_block(level, &bb, orient, mirror, rotation, sand.clone(), x - 2, y + 3, z + 4, box_);
        place_block(
            level, &bb, orient, mirror, rotation,
            if random_boolean { sand.clone() } else { sandstone.clone() },
            x - 1, y + 3, z + 4, box_,
        );
        place_block(
            level, &bb, orient, mirror, rotation,
            if !random_boolean { sand.clone() } else { sandstone.clone() },
            x, y + 3, z + 4, box_,
        );
        place_block(level, &bb, orient, mirror, rotation, sand.clone(), x - 1, y + 2, z + 4, box_);
        place_block(level, &bb, orient, mirror, rotation, sandstone.clone(), x, y + 2, z + 4, box_);
        place_block(level, &bb, orient, mirror, rotation, sand, x, y + 1, z + 4, box_);
    }

    /// `addCellarRoom`.
    fn add_cellar_room(&mut self, level: &dyn LevelReader, pos: BlockPos, box_: &BoundingBox) {
        let (x, y, z) = (pos.x, pos.y, pos.z);
        let bb = self.bounding_box.clone();
        let orient = self.orientation;
        let mirror = self.mirror;
        let rotation = self.rotation;
        let cut = block_state("cut_sandstone");
        let chiseled = block_state("chiseled_sandstone");
        let sandstone = block_state("sandstone");
        let orange = block_state("orange_terracotta");
        let blue = block_state("blue_terracotta");
        let sand = block_state("sand");

        // Стены подвала (generateBox existing_only=true).
        generate_box(level, &bb, orient, mirror, rotation, box_, x - 3, y + 1, z - 3, x - 3, y + 1, z + 2, &cut, &cut, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x + 3, y + 1, z - 3, x + 3, y + 1, z + 2, &cut, &cut, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x - 3, y + 1, z - 3, x + 3, y + 1, z - 2, &cut, &cut, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x - 3, y + 1, z + 3, x + 3, y + 1, z + 3, &cut, &cut, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x - 3, y + 2, z - 3, x - 3, y + 2, z + 2, &chiseled, &chiseled, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x + 3, y + 2, z - 3, x + 3, y + 2, z + 2, &chiseled, &chiseled, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x - 3, y + 2, z - 3, x + 3, y + 2, z - 2, &chiseled, &chiseled, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x - 3, y + 2, z + 3, x + 3, y + 2, z + 3, &chiseled, &chiseled, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x - 3, -1, z - 3, x - 3, -1, z + 2, &cut, &cut, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x + 3, -1, z - 3, x + 3, -1, z + 2, &cut, &cut, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x - 3, -1, z - 3, x + 3, -1, z - 2, &cut, &cut, true);
        generate_box(level, &bb, orient, mirror, rotation, box_, x - 3, -1, z + 3, x + 3, -1, z + 3, &cut, &cut, true);

        self.place_sand_box(x - 2, y + 1, z - 2, x + 2, y + 3, z + 2);
        self.place_collapsed_roof(level, box_, x - 2, y + 4, z - 2, x + 2, z + 2);

        place_block(level, &bb, orient, mirror, rotation, blue.clone(), x, y, z, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x + 1, y, z - 1, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x + 1, y, z + 1, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x - 1, y, z - 1, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x - 1, y, z + 1, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x + 2, y, z, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x - 2, y, z, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x, y, z + 2, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x, y, z - 2, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x + 3, y, z, box_);
        self.place_sand(x + 3, y + 1, z);
        self.place_sand(x + 3, y + 2, z);
        place_block(level, &bb, orient, mirror, rotation, cut.clone(), x + 4, y + 1, z, box_);
        place_block(level, &bb, orient, mirror, rotation, chiseled.clone(), x + 4, y + 2, z, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x - 3, y, z, box_);
        self.place_sand(x - 3, y + 1, z);
        self.place_sand(x - 3, y + 2, z);
        place_block(level, &bb, orient, mirror, rotation, cut.clone(), x - 4, y + 1, z, box_);
        place_block(level, &bb, orient, mirror, rotation, chiseled.clone(), x - 4, y + 2, z, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x, y, z + 3, box_);
        self.place_sand(x, y + 1, z + 3);
        self.place_sand(x, y + 2, z + 3);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), x, y, z - 3, box_);
        self.place_sand(x, y + 1, z - 3);
        self.place_sand(x, y + 2, z - 3);
        place_block(level, &bb, orient, mirror, rotation, sandstone.clone(), x, y + 1, z - 4, box_);
        place_block(level, &bb, orient, mirror, rotation, chiseled.clone(), x, -2, z - 4, box_);
        let _ = sand;
    }
}

impl StructurePiece for DesertPyramidPiece {
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
        piece_set_orientation(orientation, &mut self.mirror, &mut self.rotation);
    }

    fn rotation(&self) -> Rotation {
        self.rotation
    }

    fn mirror(&self) -> Mirror {
        self.mirror
    }

    fn piece_type_id(&self) -> Identifier {
        Identifier::minecraft("desert_pyramid")
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        self.scattered.save(tag);
        tag.put("hasPlacedChest0", Tag::Byte(if self.has_placed_chest[0] { 1 } else { 0 }));
        tag.put("hasPlacedChest1", Tag::Byte(if self.has_placed_chest[1] { 1 } else { 0 }));
        tag.put("hasPlacedChest2", Tag::Byte(if self.has_placed_chest[2] { 1 } else { 0 }));
        tag.put("hasPlacedChest3", Tag::Byte(if self.has_placed_chest[3] { 1 } else { 0 }));
    }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.scattered = ScatteredFeaturePieceData::load(tag);
        self.has_placed_chest = [
            tag.get_bool("hasPlacedChest0").unwrap_or(false),
            tag.get_bool("hasPlacedChest1").unwrap_or(false),
            tag.get_bool("hasPlacedChest2").unwrap_or(false),
            tag.get_bool("hasPlacedChest3").unwrap_or(false),
        ];
    }

    fn add_children(
        &mut self,
        _piece: &mut dyn StructurePiece,
        _pieces: &mut dyn StructurePieceAccessor,
        _random: &mut dyn RandomSource,
    ) {
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    #[allow(clippy::too_many_arguments)]
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
        let bounds = self.bounding_box.clone();
        if !self
            .scattered
            .update_height_position_to_lowest_ground_height(
                level,
                -random.next_int_bound(3),
                &mut self.bounding_box,
            )
        {
            return;
        }
        let bb = self.bounding_box.clone();
        let orient = self.orientation;
        let mirror = self.mirror;
        let rotation = self.rotation;
        let width = self.scattered.width;
        let depth = self.scattered.depth;

        let sandstone = block_state("sandstone");
        let air = BlockState::air();
        let cut = block_state("cut_sandstone");
        let chiseled = block_state("chiseled_sandstone");
        let orange = block_state("orange_terracotta");
        let blue = block_state("blue_terracotta");
        let sand = block_state("sand");
        let slab = block_state("sandstone_slab");
        let tnt = block_state("tnt");
        let pressure_plate = block_state("stone_pressure_plate");

        let stairs_north = with_string_property(block_state("sandstone_stairs"), "facing", "north");
        let stairs_south = with_string_property(block_state("sandstone_stairs"), "facing", "south");
        let stairs_east = with_string_property(block_state("sandstone_stairs"), "facing", "east");
        let stairs_west = with_string_property(block_state("sandstone_stairs"), "facing", "west");

        // Основание и ярусы пирамиды.
        generate_box(level, &bb, orient, mirror, rotation, box_, 0, -4, 0, width - 1, 0, depth - 1, &sandstone, &sandstone, false);
        for i in 1..=9 {
            generate_box(level, &bb, orient, mirror, rotation, box_, i, i, i, width - 1 - i, i, depth - 1 - i, &sandstone, &sandstone, false);
            generate_box(level, &bb, orient, mirror, rotation, box_, i + 1, i, i + 1, width - 2 - i, i, depth - 2 - i, &air, &air, false);
        }

        // Столбы sandstone до min_build_height+1.
        for z in 0..depth {
            for x in 0..width {
                fill_column_down(level, &bb, orient, mirror, rotation, box_, &sandstone, x, -5, z);
            }
        }

        // Верхние выступы по углам.
        generate_box(level, &bb, orient, mirror, rotation, box_, 0, 0, 0, 4, 9, 4, &sandstone, &air, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 1, 10, 1, 3, 10, 3, &sandstone, &sandstone, false);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 2, 10, 0, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_south.clone(), 2, 10, 4, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_east.clone(), 0, 10, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_west.clone(), 4, 10, 2, box_);
        generate_box(level, &bb, orient, mirror, rotation, box_, width - 5, 0, 0, width - 1, 9, 4, &sandstone, &air, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, width - 4, 10, 1, width - 2, 10, 3, &sandstone, &sandstone, false);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), width - 3, 10, 0, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_south.clone(), width - 3, 10, 4, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_east.clone(), width - 5, 10, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_west.clone(), width - 1, 10, 2, box_);

        // Входы (8..12) и коридоры.
        generate_box(level, &bb, orient, mirror, rotation, box_, 8, 0, 0, 12, 4, 4, &sandstone, &air, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 9, 1, 0, 11, 3, 4, &air, &air, false);
        for (px, py, pz) in [(9, 1, 1), (9, 2, 1), (9, 3, 1), (10, 3, 1), (11, 3, 1), (11, 2, 1), (11, 1, 1)] {
            place_block(level, &bb, orient, mirror, rotation, cut.clone(), px, py, pz, box_);
        }
        generate_box(level, &bb, orient, mirror, rotation, box_, 4, 1, 1, 8, 3, 3, &sandstone, &air, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 4, 1, 2, 8, 2, 2, &air, &air, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 12, 1, 1, 16, 3, 3, &sandstone, &air, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 12, 1, 2, 16, 2, 2, &air, &air, false);

        // Плита этажа 5 и колонны.
        generate_box(level, &bb, orient, mirror, rotation, box_, 5, 4, 5, width - 6, 4, depth - 6, &sandstone, &sandstone, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 9, 4, 9, 11, 4, 11, &air, &air, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 8, 1, 8, 8, 3, 8, &cut, &cut, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 12, 1, 8, 12, 3, 8, &cut, &cut, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 8, 1, 12, 8, 3, 12, &cut, &cut, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 12, 1, 12, 12, 3, 12, &cut, &cut, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 1, 1, 5, 4, 4, 11, &sandstone, &sandstone, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, width - 5, 1, 5, width - 2, 4, 11, &sandstone, &sandstone, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 6, 7, 9, 6, 7, 11, &sandstone, &sandstone, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, width - 7, 7, 9, width - 7, 7, 11, &sandstone, &sandstone, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 5, 5, 9, 5, 7, 11, &cut, &cut, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, width - 6, 5, 9, width - 6, 7, 11, &cut, &cut, false);
        for (px, py, pz) in [(5, 5, 10), (5, 6, 10), (6, 6, 10), (width - 6, 5, 10), (width - 6, 6, 10), (width - 7, 6, 10)] {
            place_block(level, &bb, orient, mirror, rotation, air.clone(), px, py, pz, box_);
        }
        generate_box(level, &bb, orient, mirror, rotation, box_, 2, 4, 4, 2, 6, 4, &air, &air, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, width - 3, 4, 4, width - 3, 6, 4, &air, &air, false);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 2, 4, 5, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 2, 3, 4, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), width - 3, 4, 5, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), width - 3, 3, 4, box_);

        // Боковые блоки и плиты у входа.
        generate_box(level, &bb, orient, mirror, rotation, box_, 1, 1, 3, 2, 2, 3, &sandstone, &sandstone, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, width - 3, 1, 3, width - 2, 2, 3, &sandstone, &sandstone, false);
        place_block(level, &bb, orient, mirror, rotation, sandstone.clone(), 1, 1, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, sandstone.clone(), width - 2, 1, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, slab.clone(), 1, 2, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, slab.clone(), width - 2, 2, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_west.clone(), 2, 1, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_east.clone(), width - 3, 1, 2, box_);

        // Коридоры вдоль стен.
        generate_box(level, &bb, orient, mirror, rotation, box_, 4, 3, 5, 4, 3, 17, &sandstone, &sandstone, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, width - 5, 3, 5, width - 5, 3, 17, &sandstone, &sandstone, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 3, 1, 5, 4, 2, 16, &air, &air, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, width - 6, 1, 5, width - 5, 2, 16, &air, &air, false);
        let mut i3 = 5;
        while i3 <= 17 {
            place_block(level, &bb, orient, mirror, rotation, cut.clone(), 4, 1, i3, box_);
            place_block(level, &bb, orient, mirror, rotation, chiseled.clone(), 4, 2, i3, box_);
            place_block(level, &bb, orient, mirror, rotation, cut.clone(), width - 5, 1, i3, box_);
            place_block(level, &bb, orient, mirror, rotation, chiseled.clone(), width - 5, 2, i3, box_);
            i3 += 2;
        }

        // Мозаика пола.
        for (px, pz) in [(10, 7), (10, 8), (9, 9), (11, 9), (8, 10), (12, 10), (7, 10), (13, 10), (9, 11), (11, 11), (10, 12), (10, 13)] {
            place_block(level, &bb, orient, mirror, rotation, orange.clone(), px, 0, pz, box_);
        }
        place_block(level, &bb, orient, mirror, rotation, blue.clone(), 10, 0, 10, box_);

        // Фасады с терракотой (торцы i3 ∈ {0, width-1}, z=1..3).
        // Раскладка по y: 2/3/5 → (cut, orange, cut);
        // 4/6 → (orange, chiseled, orange); 7 → все orange; 8 → все cut.
        let mut i3 = 0;
        while i3 <= width - 1 {
            for (y, states) in [
                (2, (cut.clone(), orange.clone(), cut.clone())),
                (3, (cut.clone(), orange.clone(), cut.clone())),
                (4, (orange.clone(), chiseled.clone(), orange.clone())),
                (5, (cut.clone(), orange.clone(), cut.clone())),
                (6, (orange.clone(), chiseled.clone(), orange.clone())),
                (7, (orange.clone(), orange.clone(), orange.clone())),
                (8, (cut.clone(), cut.clone(), cut.clone())),
            ] {
                place_block(level, &bb, orient, mirror, rotation, states.0, i3, y, 1, box_);
                place_block(level, &bb, orient, mirror, rotation, states.1, i3, y, 2, box_);
                place_block(level, &bb, orient, mirror, rotation, states.2, i3, y, 3, box_);
            }
            i3 += width - 1;
        }
        // Фасады z=0 (i3 ∈ {2, width-3}): три блока по x на каждый y.
        let mut i3 = 2;
        while i3 <= width - 3 {
            for (y, states) in [
                (2, (cut.clone(), orange.clone(), cut.clone())),
                (3, (cut.clone(), orange.clone(), cut.clone())),
                (4, (orange.clone(), chiseled.clone(), orange.clone())),
                (5, (cut.clone(), orange.clone(), cut.clone())),
                (6, (orange.clone(), chiseled.clone(), orange.clone())),
                (7, (orange.clone(), orange.clone(), orange.clone())),
                (8, (cut.clone(), cut.clone(), cut.clone())),
            ] {
                place_block(level, &bb, orient, mirror, rotation, states.0, i3 - 1, y, 0, box_);
                place_block(level, &bb, orient, mirror, rotation, states.1, i3, y, 0, box_);
                place_block(level, &bb, orient, mirror, rotation, states.2, i3 + 1, y, 0, box_);
            }
            i3 += width - 3 - 2;
        }
        generate_box(level, &bb, orient, mirror, rotation, box_, 8, 4, 0, 12, 6, 0, &cut, &cut, false);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 8, 6, 0, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 12, 6, 0, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), 9, 5, 0, box_);
        place_block(level, &bb, orient, mirror, rotation, chiseled.clone(), 10, 5, 0, box_);
        place_block(level, &bb, orient, mirror, rotation, orange.clone(), 11, 5, 0, box_);

        // Подвал: стены, пол, TNT, давление.
        generate_box(level, &bb, orient, mirror, rotation, box_, 8, -14, 8, 12, -11, 12, &cut, &cut, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 8, -10, 8, 12, -10, 12, &chiseled, &chiseled, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 8, -9, 8, 12, -9, 12, &cut, &cut, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 8, -8, 8, 12, -1, 12, &sandstone, &sandstone, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 9, -11, 9, 11, -1, 11, &air, &air, false);
        place_block(level, &bb, orient, mirror, rotation, pressure_plate.clone(), 10, -11, 10, box_);
        generate_box(level, &bb, orient, mirror, rotation, box_, 9, -13, 9, 11, -13, 11, &tnt, &air, false);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 8, -11, 10, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 8, -10, 10, box_);
        place_block(level, &bb, orient, mirror, rotation, chiseled.clone(), 7, -10, 10, box_);
        place_block(level, &bb, orient, mirror, rotation, cut.clone(), 7, -11, 10, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 12, -11, 10, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 12, -10, 10, box_);
        place_block(level, &bb, orient, mirror, rotation, chiseled.clone(), 13, -10, 10, box_);
        place_block(level, &bb, orient, mirror, rotation, cut.clone(), 13, -11, 10, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 10, -11, 8, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 10, -10, 8, box_);
        place_block(level, &bb, orient, mirror, rotation, chiseled.clone(), 10, -10, 7, box_);
        place_block(level, &bb, orient, mirror, rotation, cut.clone(), 10, -11, 7, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 10, -11, 12, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 10, -10, 12, box_);
        place_block(level, &bb, orient, mirror, rotation, chiseled.clone(), 10, -10, 13, box_);
        place_block(level, &bb, orient, mirror, rotation, cut.clone(), 10, -11, 13, box_);

        // Сундуки по 4 направлениям.
        let loot = Identifier::minecraft("chests/desert_pyramid");
        for direction in Plane::HORIZONTAL.faces() {
            let idx = direction.get_2d_data_value() as usize;
            if !self.has_placed_chest[idx] {
                let i4 = direction.get_step_x() * 2;
                let i5 = direction.get_step_z() * 2;
                self.has_placed_chest[idx] = create_chest(
                    level, &bb, orient, mirror, rotation, box_, random,
                    10 + i4, -11, 10 + i5, &loot,
                );
            }
        }

        self.add_cellar(level, box_);
    }
}

/// `DesertPyramidStructure` — `SinglePieceStructure(21, 21)` +
/// `afterPlace` (подозрительный песок).
pub fn desert_pyramid_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
) -> Structure {
    let mut structure = single_piece_structure(
        id,
        settings,
        structure_type,
        21,
        21,
        Arc::new(|random: &mut dyn RandomSource, x: i32, z: i32| {
            Box::new(DesertPyramidPiece::new(random, x, z)) as Box<dyn StructurePiece>
        }),
    );
    structure.after_place = Some(Arc::new(
        |level: &dyn LevelReader,
         _structure_manager: &dyn StructureManagerAccess,
         random: &mut dyn RandomSource,
         bounding_box: &BoundingBox,
         _chunk_pos: ChunkPos,
         pieces: &PiecesContainer| {
            // SortedArraySet(Vec3i::compareTo) — BTreeSet дедупит+сортирует.
            let mut set: BTreeSet<BlockPos> = BTreeSet::new();
            let mut collapsed_roof_pos: Vec<BlockPos> = Vec::new();
            for piece in pieces.pieces() {
                if let Some(p) = piece.as_any().and_then(|a| a.downcast_ref::<DesertPyramidPiece>()) {
                    set.extend(p.potential_suspicious_sand_world_positions.iter().copied());
                    collapsed_roof_pos.push(p.random_collapsed_roof_pos);
                }
            }
            for pos in &collapsed_roof_pos {
                place_suspicious_sand(bounding_box, level, *pos);
            }

            let center = pieces
                .calculate_bounding_box()
                .map(|bb| bb.get_center())
                .unwrap_or(BlockPos::ZERO);
            // `RandomSource.create(seed).forkPositional().at(center)`.
            let seed = level.world_seed();
            let mut random_source = crate::random::legacy::LegacyRandomSource::new(seed)
                .fork_positional()
                .at(center.x, center.y, center.z);
            let mut list: Vec<BlockPos> = set.into_iter().collect();
            shuffle(&mut list, random_source.as_mut());
            let mut min = list.len().min((random_source.next_int_bound(4) + 5) as usize);
            for pos in &list {
                if min > 0 {
                    min -= 1;
                    place_suspicious_sand(bounding_box, level, *pos);
                } else if bounding_box.is_inside_pos(*pos) {
                    let sand = block_state("sand");
                    level.set_block_state(*pos, &sand);
                }
            }
            let _ = random;
        },
    ));
    structure
}

/// `placeSuspiciousSand` — suspicious_sand + loot table
/// `DESERT_PYRAMID_ARCHAEOLOGY` на block entity.
fn place_suspicious_sand(bounding_box: &BoundingBox, level: &dyn LevelReader, pos: BlockPos) {
    if !bounding_box.is_inside_pos(pos) {
        return;
    }
    let suspicious = block_state("suspicious_sand");
    level.set_block_state(pos, &suspicious);
    level.place_loot_container(
        pos,
        &Identifier::minecraft("archaeology/desert_pyramid"),
        pos.as_long(),
    );
}
