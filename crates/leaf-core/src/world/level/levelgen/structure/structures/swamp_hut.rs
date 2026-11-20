//! Перенос `structures/SwampHutStructure.java` + `SwampHutPiece.java`.

use super::super::bounding_box::BoundingBox;
use super::super::scattered_feature_piece::ScatteredFeaturePieceData;
use super::super::structure::{
    on_top_of_chunk_center, GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_piece::{
    fill_column_down, generate_box, place_block, piece_set_orientation, StructurePiece,
    StructurePieceAccessor,
};
use super::super::structure_type::StructureType;
use super::{block_state, with_string_property};
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::entity::entity_type::entity_type_by_identifier;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::entity::entity_spawn_reason::EntitySpawnReason;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess;
use crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess;
use std::sync::Arc;

/// `SwampHutPiece` — кусок хижины ведьмы.
pub struct SwampHutPiece {
    scattered: ScatteredFeaturePieceData,
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    direction: Direction,
    mirror: Mirror,
    rotation: Rotation,
    spawned_witch: bool,
    spawned_cat: bool,
}

impl SwampHutPiece {
    /// `new(random, x, z)` — `ScatteredFeaturePiece(SWAMPLAND_HUT,
    /// x, 64, z, 7, 7, 9, randomHorizontalDirection)`.
    pub fn new(random: &mut dyn RandomSource, x: i32, z: i32) -> Self {
        let direction = super::super::structure_piece::get_random_horizontal_direction(random);
        let (scattered, bounding_box) =
            ScatteredFeaturePieceData::new(x, 64, z, 7, 7, 9, direction);
        Self {
            scattered,
            bounding_box,
            gen_depth: 0,
            orientation: Some(direction),
            direction,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            spawned_witch: false,
            spawned_cat: false,
        }
    }

    /// `new(tag)` — загрузка из NBT.
    pub fn from_nbt(tag: &CompoundTag) -> Self {
        let scattered = ScatteredFeaturePieceData::load(tag);
        // Java: orientation из StructurePiece-базового тега.
        let direction = Direction::by_name(tag.get_string("O").unwrap_or("north"))
            .unwrap_or(Direction::NORTH);
        let bounding_box = BoundingBox::new(
            tag.get_int("TPX").unwrap_or(0),
            tag.get_int("TPY").unwrap_or(0),
            tag.get_int("TPZ").unwrap_or(0),
            tag.get_int("TPX").unwrap_or(0) + scattered.width - 1,
            tag.get_int("TPY").unwrap_or(0) + scattered.height - 1,
            tag.get_int("TPZ").unwrap_or(0) + scattered.depth - 1,
        );
        let mut piece = Self {
            scattered,
            bounding_box,
            gen_depth: 0,
            orientation: Some(direction),
            direction,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            spawned_witch: tag.get_bool("Witch").unwrap_or(false),
            spawned_cat: tag.get_bool("Cat").unwrap_or(false),
        };
        piece_set_orientation(Some(direction), &mut piece.mirror, &mut piece.rotation);
        piece
    }

    /// `getWorldPos(x, y, z)` — через PieceTransform.
    fn world_pos(&self, x: i32, y: i32, z: i32) -> BlockPos {
        super::super::structure_piece::PieceTransform {
            bb: &self.bounding_box,
            orientation: self.orientation,
        }
        .world_pos(x, y, z)
    }

    /// `spawnWitch`-логика — создаёт Witch через level-хуки.
    fn spawn_mob(
        &mut self,
        level: &dyn LevelReader,
        box_: &BoundingBox,
        spawned: &mut bool,
        entity_name: &str,
    ) {
        if *spawned {
            return;
        }
        let world_pos = self.world_pos(2, 2, 5);
        if !box_.is_inside_pos(world_pos) {
            return;
        }
        *spawned = true;
        let Some(entity_type) =
            entity_type_by_identifier(&Identifier::minecraft(entity_name))
        else {
            return;
        };
        let Some(mut mob) = level.create_mob_for_spawn(entity_type.as_ref()) else {
            return;
        };
        mob.set_persistence_required(true);
        mob.living.entity.snap_to(
            world_pos.x as f64 + 0.5,
            world_pos.y as f64,
            world_pos.z as f64 + 0.5,
            0.0,
            0.0,
        );
        let difficulty = level.current_difficulty_at(world_pos);
        mob.finalize_spawn(
            level,
            difficulty as i32 as f32,
            EntitySpawnReason::Structure,
            None,
        );
        level.try_add_fresh_entity_with_passengers(mob.living.entity.clone());
    }
}

impl StructurePiece for SwampHutPiece {
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
        Identifier::minecraft("swamp_hut")
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        self.scattered.save(tag);
        tag.put("Witch", Tag::Byte(if self.spawned_witch { 1 } else { 0 }));
        tag.put("Cat", Tag::Byte(if self.spawned_cat { 1 } else { 0 }));
    }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.scattered = ScatteredFeaturePieceData::load(tag);
        self.spawned_witch = tag.get_bool("Witch").unwrap_or(false);
        self.spawned_cat = tag.get_bool("Cat").unwrap_or(false);
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
        _random: &mut dyn RandomSource,
        box_: &BoundingBox,
        _chunk_pos: ChunkPos,
        _pos: BlockPos,
    ) {
        let bounds = box_.clone();
        if !self
            .scattered
            .update_average_ground_height(level, &bounds, 0, &mut self.bounding_box)
        {
            return;
        }
        let bb = self.bounding_box.clone();
        let orient = self.orientation;
        let mirror = self.mirror;
        let rotation = self.rotation;
        let spruce_planks = block_state("spruce_planks");
        let oak_log = block_state("oak_log");
        let air = BlockState::air();
        let oak_fence = block_state("oak_fence");
        let potted_mushroom = block_state("potted_red_mushroom");
        let crafting_table = block_state("crafting_table");
        let cauldron = block_state("cauldron");

        // Крыша и пол (vanilla generateBox: boundary=inside=state,
        // existing_only=false → внутри тоже state т.к. граничные
        // слои совпадают).
        generate_box(level, &bb, orient, mirror, rotation, box_, 1, 1, 1, 5, 1, 7, &spruce_planks, &spruce_planks, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 1, 4, 2, 5, 4, 7, &spruce_planks, &spruce_planks, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 2, 1, 0, 4, 1, 0, &spruce_planks, &spruce_planks, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 2, 2, 2, 3, 3, 2, &spruce_planks, &spruce_planks, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 1, 2, 3, 1, 3, 6, &spruce_planks, &spruce_planks, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 5, 2, 3, 5, 3, 6, &spruce_planks, &spruce_planks, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 2, 2, 7, 4, 3, 7, &spruce_planks, &spruce_planks, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 1, 0, 2, 1, 3, 2, &oak_log, &oak_log, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 5, 0, 2, 5, 3, 2, &oak_log, &oak_log, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 1, 0, 7, 1, 3, 7, &oak_log, &oak_log, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 5, 0, 7, 5, 3, 7, &oak_log, &oak_log, false);

        place_block(level, &bb, orient, mirror, rotation, oak_fence.clone(), 2, 3, 2, box_);
        place_block(level, &bb, orient, mirror, rotation, oak_fence.clone(), 3, 3, 7, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 1, 3, 4, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 5, 3, 4, box_);
        place_block(level, &bb, orient, mirror, rotation, air.clone(), 5, 3, 5, box_);
        place_block(level, &bb, orient, mirror, rotation, potted_mushroom.clone(), 1, 3, 5, box_);
        place_block(level, &bb, orient, mirror, rotation, crafting_table.clone(), 3, 2, 6, box_);
        place_block(level, &bb, orient, mirror, rotation, cauldron.clone(), 4, 2, 6, box_);
        place_block(level, &bb, orient, mirror, rotation, oak_fence.clone(), 1, 2, 1, box_);
        place_block(level, &bb, orient, mirror, rotation, oak_fence.clone(), 5, 2, 1, box_);

        // Ступени: facing NORTH/EAST/WEST/SOUTH + OUTER shape.
        let stairs_north = with_string_property(
            with_string_property(block_state("spruce_stairs"), "facing", "north"),
            "shape",
            "straight",
        );
        let stairs_east = with_string_property(
            with_string_property(block_state("spruce_stairs"), "facing", "east"),
            "shape",
            "straight",
        );
        let stairs_west = with_string_property(
            with_string_property(block_state("spruce_stairs"), "facing", "west"),
            "shape",
            "straight",
        );
        let stairs_south = with_string_property(
            with_string_property(block_state("spruce_stairs"), "facing", "south"),
            "shape",
            "straight",
        );
        generate_box(level, &bb, orient, mirror, rotation, box_, 0, 4, 1, 6, 4, 1, &stairs_north, &stairs_north, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 0, 4, 2, 0, 4, 7, &stairs_east, &stairs_east, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 6, 4, 2, 6, 4, 7, &stairs_west, &stairs_west, false);
        generate_box(level, &bb, orient, mirror, rotation, box_, 0, 4, 8, 6, 4, 8, &stairs_south, &stairs_south, false);

        let outer_right_n = with_string_property(
            with_string_property(block_state("spruce_stairs"), "facing", "north"),
            "shape",
            "outer_right",
        );
        let outer_left_n = with_string_property(
            with_string_property(block_state("spruce_stairs"), "facing", "north"),
            "shape",
            "outer_left",
        );
        let outer_left_s = with_string_property(
            with_string_property(block_state("spruce_stairs"), "facing", "south"),
            "shape",
            "outer_left",
        );
        let outer_right_s = with_string_property(
            with_string_property(block_state("spruce_stairs"), "facing", "south"),
            "shape",
            "outer_right",
        );
        place_block(level, &bb, orient, mirror, rotation, outer_right_n, 0, 4, 1, box_);
        place_block(level, &bb, orient, mirror, rotation, outer_left_n, 6, 4, 1, box_);
        place_block(level, &bb, orient, mirror, rotation, outer_left_s, 0, 4, 8, box_);
        place_block(level, &bb, orient, mirror, rotation, outer_right_s, 6, 4, 8, box_);

        for i in [2, 7] {
            for i1 in [1, 5] {
                fill_column_down(level, &bb, orient, mirror, rotation, box_, &oak_log, i1, -1, i);
            }
        }

        self.spawn_mob(level, box_, &mut self.spawned_witch, "witch");
        self.spawn_mob(level, box_, &mut self.spawned_cat, "cat");
    }
}

/// `SwampHutStructure` — фабрика.
pub fn swamp_hut_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
) -> Structure {
    let mut structure = Structure::new(id, settings, structure_type);
    structure.find_generation_point =
        Some(Arc::new(|context: &mut GenerationContext| -> Option<GenerationStub> {
            let chunk_pos = context.chunk_pos;
            Some(on_top_of_chunk_center(
                context,
                HeightmapType::WorldSurfaceWg,
                move |pieces, context| {
                    // Java: `SwampHutPiece::new(context.random(), ...)`.
                    let min_x = chunk_pos.get_min_block_x();
                    let min_z = chunk_pos.get_min_block_z();
                    pieces.push(Box::new(SwampHutPiece::new(
                        &mut context.random,
                        min_x,
                        min_z,
                    )));
                },
            ))
        }));
    structure
}
