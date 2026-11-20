//! Перенос `structures/JungleTempleStructure.java` +
//! `structures/JungleTemplePiece.java` — храм в джунглях
//! (12×10×15, ScatteredFeaturePiece, ловушки, прячущиеся
//! сундуки, рычаги/поршни).

use super::super::bounding_box::BoundingBox;
use super::super::scattered_feature_piece::ScatteredFeaturePieceData;
use super::super::single_piece_structure::single_piece_structure;
use super::super::structure::{Structure, StructureSettings};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{
    create_chest, create_dispenser, generate_air_box, generate_box_selector,
    piece_set_orientation, place_block, BlockSelector, StructurePiece,
    StructurePieceAccessor,
};
use super::super::structure_type::StructureType;
use super::{block_state, with_bool_property, with_string_property};
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess;
use std::sync::Arc;

/// `JungleTemplePiece.MossStoneSelector` — cobblestone 40%,
/// иначе mossy_cobblestone.
pub struct MossStoneSelector {
    next: BlockState,
}

impl MossStoneSelector {
    /// `new()` — `next` инициализируется при первом `next()`
    /// (в Java null до первого вызова; у нас air-заглушка).
    pub fn new() -> Self {
        Self {
            next: BlockState::air(),
        }
    }
}

impl BlockSelector for MossStoneSelector {
    fn next(&mut self, random: &mut dyn RandomSource, _x: i32, _y: i32, _z: i32, _wall: bool) {
        self.next = if random.next_float() < 0.4 {
            block_state("cobblestone")
        } else {
            block_state("mossy_cobblestone")
        };
    }

    fn get_next(&self) -> &BlockState {
        &self.next
    }
}

/// `JungleTemplePiece` — `ScatteredFeaturePiece(JUNGLE_PYRAMID_PIECE,
/// x, 64, z, 12, 10, 15, randomHorizontalDirection)`.
pub struct JungleTemplePiece {
    scattered: ScatteredFeaturePieceData,
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    placed_main_chest: bool,
    placed_hidden_chest: bool,
    placed_trap1: bool,
    placed_trap2: bool,
}

impl JungleTemplePiece {
    /// `new(random, x, z)`.
    pub fn new(random: &mut dyn RandomSource, x: i32, z: i32) -> Self {
        let direction = super::super::structure_piece::get_random_horizontal_direction(random);
        let (scattered, bounding_box) =
            ScatteredFeaturePieceData::new(x, 64, z, 12, 10, 15, direction);
        Self {
            scattered,
            bounding_box,
            gen_depth: 0,
            orientation: Some(direction),
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            placed_main_chest: false,
            placed_hidden_chest: false,
            placed_trap1: false,
            placed_trap2: false,
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
                tag.get_int("TPX").unwrap_or(0) + 12 - 1,
                tag.get_int("TPY").unwrap_or(0) + 10 - 1,
                tag.get_int("TPZ").unwrap_or(0) + 15 - 1,
            ),
            gen_depth: 0,
            orientation: Some(direction),
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            placed_main_chest: tag.get_bool("placedMainChest").unwrap_or(false),
            placed_hidden_chest: tag.get_bool("placedHiddenChest").unwrap_or(false),
            placed_trap1: tag.get_bool("placedTrap1").unwrap_or(false),
            placed_trap2: tag.get_bool("placedTrap2").unwrap_or(false),
        };
        piece_set_orientation(Some(direction), &mut piece.mirror, &mut piece.rotation);
        piece
    }

    /// `generateBox(..., false, random, STONE_SELECTOR)` —
    /// обёртка с кусковыми параметрами.
    #[allow(clippy::too_many_arguments)]
    fn gen_stone(
        level: &dyn LevelReader,
        bb: &BoundingBox,
        orient: Option<Direction>,
        mirror: Mirror,
        rotation: Rotation,
        box_: &BoundingBox,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        random: &mut dyn RandomSource,
    ) {
        generate_box_selector(
            level, bb, orient, mirror, rotation, box_,
            min_x, min_y, min_z, max_x, max_y, max_z,
            false, random, &mut MossStoneSelector::new(),
        );
    }
}

impl StructurePiece for JungleTemplePiece {
    fn bounding_box(&self) -> BoundingBox {
        self.bounding_box.clone()
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
        Identifier::minecraft("jungle_pyramid")
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        self.scattered.save(tag);
        tag.put("placedMainChest", Tag::Byte(if self.placed_main_chest { 1 } else { 0 }));
        tag.put("placedHiddenChest", Tag::Byte(if self.placed_hidden_chest { 1 } else { 0 }));
        tag.put("placedTrap1", Tag::Byte(if self.placed_trap1 { 1 } else { 0 }));
        tag.put("placedTrap2", Tag::Byte(if self.placed_trap2 { 1 } else { 0 }));
    }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.scattered = ScatteredFeaturePieceData::load(tag);
        self.placed_main_chest = tag.get_bool("placedMainChest").unwrap_or(false);
        self.placed_hidden_chest = tag.get_bool("placedHiddenChest").unwrap_or(false);
        self.placed_trap1 = tag.get_bool("placedTrap1").unwrap_or(false);
        self.placed_trap2 = tag.get_bool("placedTrap2").unwrap_or(false);
    }

    fn add_children(
        &mut self,
        _piece: &mut dyn StructurePiece,
        _pieces: &mut dyn StructurePieceAccessor,
        _random: &mut dyn RandomSource,
    ) {
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
        // `updateAverageGroundHeight(level, box, 0)` — bounds = box.
        if !self
            .scattered
            .update_average_ground_height(level, box_, 0, &mut self.bounding_box)
        {
            return;
        }
        let bb = self.bounding_box.clone();
        let orient = self.orientation;
        let mirror = self.mirror;
        let rotation = self.rotation;
        let width = self.scattered.width;
        let depth = self.scattered.depth;

        macro_rules! stone {
            ($min_x:expr, $min_y:expr, $min_z:expr, $max_x:expr, $max_y:expr, $max_z:expr) => {
                Self::gen_stone(level, &bb, orient, mirror, rotation, box_,
                    $min_x, $min_y, $min_z, $max_x, $max_y, $max_z, random)
            };
        }
        macro_rules! air {
            ($min_x:expr, $min_y:expr, $min_z:expr, $max_x:expr, $max_y:expr, $max_z:expr) => {
                generate_air_box(level, &bb, orient, mirror, rotation, box_,
                    $min_x, $min_y, $min_z, $max_x, $max_y, $max_z)
            };
        }

        stone!(0, -4, 0, width - 1, 0, depth - 1);
        stone!(2, 1, 2, 9, 2, 2);
        stone!(2, 1, 12, 9, 2, 12);
        stone!(2, 1, 3, 2, 2, 11);
        stone!(9, 1, 3, 9, 2, 11);
        stone!(1, 3, 1, 10, 6, 1);
        stone!(1, 3, 13, 10, 6, 13);
        stone!(1, 3, 2, 1, 6, 12);
        stone!(10, 3, 2, 10, 6, 12);
        stone!(2, 3, 2, 9, 3, 12);
        stone!(2, 6, 2, 9, 6, 12);
        stone!(3, 7, 3, 8, 7, 11);
        stone!(4, 8, 4, 7, 8, 10);
        air!(3, 1, 3, 8, 2, 11);
        air!(4, 3, 6, 7, 3, 9);
        air!(2, 4, 2, 9, 5, 12);
        air!(4, 6, 5, 7, 6, 9);
        air!(5, 7, 6, 6, 7, 8);
        air!(5, 1, 2, 6, 2, 2);
        air!(5, 2, 12, 6, 2, 12);
        air!(5, 5, 1, 6, 5, 1);
        air!(5, 5, 13, 6, 5, 13);

        let air_state = BlockState::air();
        for (x, z) in [(1, 5), (10, 5), (1, 9), (10, 9)] {
            place_block(level, &bb, orient, mirror, rotation, air_state.clone(), x, 5, z, box_);
        }

        for i in [0, 14] {
            for (x1, x2) in [(2, 2), (4, 4), (7, 7), (9, 9)] {
                stone!(x1, 4, i, x2, 5, i);
            }
        }
        stone!(5, 6, 0, 6, 6, 0);

        for i in [0, 11] {
            let mut i1 = 2;
            while i1 <= 12 {
                stone!(i, 4, i1, i, 5, i1);
                i1 += 2;
            }
            stone!(i, 6, 5, i, 6, 5);
            stone!(i, 6, 9, i, 6, 9);
        }

        stone!(2, 7, 2, 2, 9, 2);
        stone!(9, 7, 2, 9, 9, 2);
        stone!(2, 7, 12, 2, 9, 12);
        stone!(9, 7, 12, 9, 9, 12);
        stone!(4, 9, 4, 4, 9, 4);
        stone!(7, 9, 4, 7, 9, 4);
        stone!(4, 9, 10, 4, 9, 10);
        stone!(7, 9, 10, 7, 9, 10);
        stone!(5, 9, 7, 6, 9, 7);

        let stairs_east = with_string_property(block_state("cobblestone_stairs"), "facing", "east");
        let stairs_west = with_string_property(block_state("cobblestone_stairs"), "facing", "west");
        let stairs_south = with_string_property(block_state("cobblestone_stairs"), "facing", "south");
        let stairs_north = with_string_property(block_state("cobblestone_stairs"), "facing", "north");
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 5, 9, 6, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 6, 9, 6, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_south.clone(), 5, 9, 8, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_south.clone(), 6, 9, 8, box_);
        for x in 4..=7 {
            place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), x, 0, 0, box_);
        }
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 4, 1, 8, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 4, 2, 9, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 4, 3, 10, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 7, 1, 8, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 7, 2, 9, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_north.clone(), 7, 3, 10, box_);
        stone!(4, 1, 9, 4, 1, 9);
        stone!(7, 1, 9, 7, 1, 9);
        stone!(4, 1, 10, 7, 2, 10);
        stone!(5, 4, 5, 6, 4, 5);
        place_block(level, &bb, orient, mirror, rotation, stairs_east.clone(), 4, 4, 5, box_);
        place_block(level, &bb, orient, mirror, rotation, stairs_west.clone(), 7, 4, 5, box_);

        for i2 in 0..4 {
            place_block(level, &bb, orient, mirror, rotation, stairs_south.clone(), 5, -i2, 6 + i2, box_);
            place_block(level, &bb, orient, mirror, rotation, stairs_south.clone(), 6, -i2, 6 + i2, box_);
            air!(5, -i2, 7 + i2, 6, -i2, 9 + i2);
        }

        air!(1, -3, 12, 10, -1, 13);
        air!(1, -3, 1, 3, -1, 13);
        air!(1, -3, 1, 9, -1, 5);

        let mut i2 = 1;
        while i2 <= 13 {
            stone!(1, -3, i2, 1, -2, i2);
            i2 += 2;
        }
        let mut i2 = 2;
        while i2 <= 12 {
            stone!(1, -1, i2, 3, -1, i2);
            i2 += 2;
        }
        stone!(2, -2, 1, 5, -2, 1);
        stone!(7, -2, 1, 9, -2, 1);
        stone!(6, -3, 1, 6, -3, 1);
        stone!(6, -1, 1, 6, -1, 1);

        // Tripwire hooks + провода.
        let mossy = block_state("mossy_cobblestone");
        let hook_east = with_bool_property(
            with_string_property(block_state("tripwire_hook"), "facing", "east"),
            "attached", true,
        );
        let hook_west = with_bool_property(
            with_string_property(block_state("tripwire_hook"), "facing", "west"),
            "attached", true,
        );
        let hook_north = with_bool_property(
            with_string_property(block_state("tripwire_hook"), "facing", "north"),
            "attached", true,
        );
        let hook_south = with_bool_property(
            with_string_property(block_state("tripwire_hook"), "facing", "south"),
            "attached", true,
        );
        let wire_ew = with_bool_property(
            with_bool_property(
                with_bool_property(block_state("tripwire"), "east", true),
                "west", true,
            ),
            "attached", true,
        );
        let wire_ns = with_bool_property(
            with_bool_property(
                with_bool_property(block_state("tripwire"), "north", true),
                "south", true,
            ),
            "attached", true,
        );

        place_block(level, &bb, orient, mirror, rotation, hook_east.clone(), 1, -3, 8, box_);
        place_block(level, &bb, orient, mirror, rotation, hook_west.clone(), 4, -3, 8, box_);
        place_block(level, &bb, orient, mirror, rotation, wire_ew.clone(), 2, -3, 8, box_);
        place_block(level, &bb, orient, mirror, rotation, wire_ew.clone(), 3, -3, 8, box_);

        // Redstone wire: side-соединения.
        let wire_ns_side = with_string_property(
            with_string_property(block_state("redstone_wire"), "north", "side"),
            "south", "side",
        );
        for z in 2..=7 {
            place_block(level, &bb, orient, mirror, rotation, wire_ns_side.clone(), 5, -3, z, box_);
        }
        let wire_west_side = with_string_property(
            with_string_property(block_state("redstone_wire"), "north", "side"),
            "west", "side",
        );
        place_block(level, &bb, orient, mirror, rotation, wire_west_side.clone(), 5, -3, 1, box_);
        let wire_ew_side = with_string_property(
            with_string_property(block_state("redstone_wire"), "east", "side"),
            "west", "side",
        );
        place_block(level, &bb, orient, mirror, rotation, wire_ew_side.clone(), 4, -3, 1, box_);
        place_block(level, &bb, orient, mirror, rotation, mossy.clone(), 3, -3, 1, box_);

        if !self.placed_trap1 {
            self.placed_trap1 = create_dispenser(
                level, &bb, orient, mirror, rotation, box_, random,
                3, -2, 1, Direction::NORTH,
                &Identifier::minecraft("chests/jungle_temple_dispenser"),
            );
        }
        place_block(level, &bb, orient, mirror, rotation, with_bool_property(block_state("vine"), "south", true), 3, -2, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, hook_north.clone(), 7, -3, 1, box_);
        place_block(level, &bb, orient, mirror, rotation, hook_south.clone(), 7, -3, 5, box_);
        place_block(level, &bb, orient, mirror, rotation, wire_ns.clone(), 7, -3, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, wire_ns.clone(), 7, -3, 3, box_);
        place_block(level, &bb, orient, mirror, rotation, wire_ns.clone(), 7, -3, 4, box_);
        let wire_sw = with_string_property(
            with_string_property(block_state("redstone_wire"), "east", "side"),
            "west", "side",
        );
        place_block(level, &bb, orient, mirror, rotation, wire_sw.clone(), 8, -3, 6, box_);
        let wire_w_s = with_string_property(
            with_string_property(block_state("redstone_wire"), "west", "side"),
            "south", "side",
        );
        place_block(level, &bb, orient, mirror, rotation, wire_w_s.clone(), 9, -3, 6, box_);
        let wire_ns_up = with_string_property(
            with_string_property(block_state("redstone_wire"), "north", "side"),
            "south", "up",
        );
        place_block(level, &bb, orient, mirror, rotation, wire_ns_up.clone(), 9, -3, 5, box_);
        place_block(level, &bb, orient, mirror, rotation, mossy.clone(), 9, -3, 4, box_);
        place_block(level, &bb, orient, mirror, rotation, wire_ns_side.clone(), 9, -2, 4, box_);

        if !self.placed_trap2 {
            self.placed_trap2 = create_dispenser(
                level, &bb, orient, mirror, rotation, box_, random,
                9, -2, 3, Direction::WEST,
                &Identifier::minecraft("chests/jungle_temple_dispenser"),
            );
        }
        place_block(level, &bb, orient, mirror, rotation, with_bool_property(block_state("vine"), "east", true), 8, -1, 3, box_);
        place_block(level, &bb, orient, mirror, rotation, with_bool_property(block_state("vine"), "east", true), 8, -2, 3, box_);

        let jungle_loot = Identifier::minecraft("chests/jungle_temple");
        if !self.placed_main_chest {
            self.placed_main_chest = create_chest(
                level, &bb, orient, mirror, rotation, box_, random,
                8, -3, 3, &jungle_loot,
            );
        }

        place_block(level, &bb, orient, mirror, rotation, mossy.clone(), 9, -3, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, mossy.clone(), 8, -3, 1, box_);
        place_block(level, &bb, orient, mirror, rotation, mossy.clone(), 4, -3, 5, box_);
        for (x, y) in [(5, -2), (5, -1), (6, -3), (7, -2), (7, -1), (8, -3)] {
            place_block(level, &bb, orient, mirror, rotation, mossy.clone(), x, y, 5, box_);
        }
        stone!(9, -1, 1, 9, -1, 5);
        air!(8, -3, 8, 10, -1, 10);
        place_block(level, &bb, orient, mirror, rotation, block_state("chiseled_stone_bricks"), 8, -2, 11, box_);
        place_block(level, &bb, orient, mirror, rotation, block_state("chiseled_stone_bricks"), 9, -2, 11, box_);
        place_block(level, &bb, orient, mirror, rotation, block_state("chiseled_stone_bricks"), 10, -2, 11, box_);
        let lever = with_string_property(
            with_string_property(block_state("lever"), "facing", "north"),
            "face", "wall",
        );
        place_block(level, &bb, orient, mirror, rotation, lever.clone(), 8, -2, 12, box_);
        place_block(level, &bb, orient, mirror, rotation, lever.clone(), 9, -2, 12, box_);
        place_block(level, &bb, orient, mirror, rotation, lever.clone(), 10, -2, 12, box_);
        stone!(8, -3, 8, 8, -3, 10);
        stone!(10, -3, 8, 10, -3, 10);
        place_block(level, &bb, orient, mirror, rotation, mossy.clone(), 10, -2, 9, box_);
        place_block(level, &bb, orient, mirror, rotation, wire_ns_side.clone(), 8, -2, 9, box_);
        place_block(level, &bb, orient, mirror, rotation, wire_ns_side.clone(), 8, -2, 10, box_);
        let wire_all = with_string_property(
            with_string_property(
                with_string_property(block_state("redstone_wire"), "north", "side"),
                "south", "side",
            ),
            "east", "side",
        );
        let wire_all = with_string_property(wire_all, "west", "side");
        place_block(level, &bb, orient, mirror, rotation, wire_all, 10, -1, 9, box_);
        place_block(
            level, &bb, orient, mirror, rotation,
            with_string_property(block_state("sticky_piston"), "facing", "up"),
            9, -2, 8, box_,
        );
        place_block(
            level, &bb, orient, mirror, rotation,
            with_string_property(block_state("sticky_piston"), "facing", "west"),
            10, -2, 8, box_,
        );
        place_block(
            level, &bb, orient, mirror, rotation,
            with_string_property(block_state("sticky_piston"), "facing", "west"),
            10, -1, 8, box_,
        );
        place_block(
            level, &bb, orient, mirror, rotation,
            with_string_property(block_state("repeater"), "facing", "north"),
            10, -2, 10, box_,
        );
        if !self.placed_hidden_chest {
            self.placed_hidden_chest = create_chest(
                level, &bb, orient, mirror, rotation, box_, random,
                9, -3, 10, &jungle_loot,
            );
        }
    }
}

/// `JungleTempleStructure` — `SinglePieceStructure(12, 15)`.
pub fn jungle_temple_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
) -> Structure {
    single_piece_structure(
        id,
        settings,
        structure_type,
        12,
        15,
        Arc::new(|random: &mut dyn RandomSource, x: i32, z: i32| {
            Box::new(JungleTemplePiece::new(random, x, z)) as Box<dyn StructurePiece>
        }),
    )
}
