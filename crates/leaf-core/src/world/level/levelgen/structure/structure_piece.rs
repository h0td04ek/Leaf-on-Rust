//! Перенос `net.minecraft.world.level.levelgen.structure.StructurePiece`
//! + `StructurePieceAccessor` + `BlockSelector`.
//!
//! `StructurePiece` — трейт (Java abstract class). Мировые
//! координаты/повороты — свободные функции `piece_*`, т.к.
//! наследники хранят `boundingBox`/`orientation`/`mirror`/`rotation`
//! по-разному (ScatteredFeaturePiece и др.).

use crate::core::block_pos::BlockPos;
use crate::core::direction::{Direction, Plane};
use crate::core::vec3i::Vec3i;
use crate::nbt::compound::CompoundTag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;

/// `StructurePieceAccessor` — коллектор частей структуры.
pub trait StructurePieceAccessor {
    /// `addPiece(piece)`.
    fn add_piece(&mut self, piece: Box<dyn StructurePiece>);
    /// `findCollisionPiece(box)` — часть, пересекающая box.
    fn find_collision_piece(&self, box_: &BoundingBox) -> Option<&dyn StructurePiece>;
}

/// Реализация `StructurePieceAccessor` для `Vec<Box<dyn
/// StructurePiece>>` (аналог `StructurePiecesBuilder`).
impl StructurePieceAccessor for Vec<Box<dyn StructurePiece>> {
    fn add_piece(&mut self, piece: Box<dyn StructurePiece>) {
        self.push(piece);
    }
    fn find_collision_piece(&self, box_: &BoundingBox) -> Option<&dyn StructurePiece> {
        self.iter()
            .map(|b| b.as_ref())
            .find(|p| p.bounding_box().intersects(box_))
    }
}

/// `StructurePiece` — часть структуры (trait-аналог abstract class).
pub trait StructurePiece {
    /// `boundingBox`.
    fn bounding_box(&self) -> BoundingBox;
    /// `genDepth`.
    fn gen_depth(&self) -> i32;
    /// `setGenDepth`.
    fn set_gen_depth(&mut self, gen_depth: i32);
    /// `getOrientation`.
    fn orientation(&self) -> Option<Direction>;
    /// `setOrientation` — также выставляет mirror/rotation
    /// (см. `piece_set_orientation`).
    fn set_orientation(&mut self, orientation: Option<Direction>);
    /// `getRotation`.
    fn rotation(&self) -> Rotation;
    /// `getMirror`.
    fn mirror(&self) -> Mirror;
    /// `getType` — id типа части (`StructurePieceType`).
    fn piece_type_id(&self) -> Identifier;
    /// `addAdditionalSaveData`.
    fn add_additional_save_data(&self, tag: &mut CompoundTag);
    /// `loadAdditionalSaveData` (ctor из NBT — части сами
    /// реализуют через `from_nbt` в реестре типов).
    fn load_additional_save_data(&mut self, tag: &CompoundTag);
    /// `addChildren(piece, pieces, random)`.
    fn add_children(
        &mut self,
        piece: &mut dyn StructurePiece,
        pieces: &mut dyn StructurePieceAccessor,
        random: &mut dyn RandomSource,
    );
    /// `postProcess(level, structureManager, generator, random,
    /// box, chunkPos, pos)`.
    fn post_process(
        &mut self,
        level: &dyn LevelReader,
        structure_manager: &dyn crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess,
        random: &mut dyn RandomSource,
        box_: &BoundingBox,
        chunk_pos: ChunkPos,
        pos: BlockPos,
    );
}

/// `makeBoundingBox(x, y, z, direction, offsetX, offsetY, offsetZ)`.
pub fn make_bounding_box(
    x: i32,
    y: i32,
    z: i32,
    direction: Direction,
    offset_x: i32,
    offset_y: i32,
    offset_z: i32,
) -> BoundingBox {
    if direction.get_axis() == crate::core::direction::Axis::Z {
        BoundingBox::new(x, y, z, x + offset_x - 1, y + offset_y - 1, z + offset_z - 1)
    } else {
        BoundingBox::new(x, y, z, x + offset_z - 1, y + offset_y - 1, x + offset_x - 1)
    }
}

/// `getRandomHorizontalDirection(random)`.
pub fn get_random_horizontal_direction(random: &mut dyn RandomSource) -> Direction {
    use crate::core::direction::{Direction, Plane};
    const H: [Direction; 4] = [Direction::NORTH, Direction::EAST, Direction::SOUTH, Direction::WEST];
    H[random.next_int_bound(4) as usize]
}

/// `setOrientation` — выставляет orientation + mirror/rotation
/// (таблица из Java switch).
pub fn piece_set_orientation(
    orientation: Option<Direction>,
    mirror: &mut Mirror,
    rotation: &mut Rotation,
) {
    match orientation {
        None => {
            *mirror = Mirror::NONE;
            *rotation = Rotation::NONE;
        }
        Some(Direction::SOUTH) => {
            *mirror = Mirror::LEFT_RIGHT;
            *rotation = Rotation::NONE;
        }
        Some(Direction::WEST) => {
            *mirror = Mirror::LEFT_RIGHT;
            *rotation = Rotation::CLOCKWISE_90;
        }
        Some(Direction::EAST) => {
            *mirror = Mirror::NONE;
            *rotation = Rotation::CLOCKWISE_90;
        }
        _ => {
            *mirror = Mirror::NONE;
            *rotation = Rotation::NONE;
        }
    }
}

/// Трансформ части — bounding box + orientation (mirror/rotation
/// хранятся у наследника, передаются параметрами).
pub struct PieceTransform<'a> {
    /// `boundingBox`.
    pub bb: &'a BoundingBox,
    /// `orientation`.
    pub orientation: Option<Direction>,
}

impl<'a> PieceTransform<'a> {
    /// `getWorldX(x, z)`.
    pub fn world_x(&self, x: i32, z: i32) -> i32 {
        match self.orientation {
            None => x,
            Some(Direction::NORTH) | Some(Direction::SOUTH) => self.bb.min_x + x,
            Some(Direction::WEST) => self.bb.max_x - z,
            Some(Direction::EAST) => self.bb.min_x + z,
            _ => x,
        }
    }

    /// `getWorldY(y)`.
    pub fn world_y(&self, y: i32) -> i32 {
        match self.orientation {
            None => y,
            Some(_) => y + self.bb.min_y,
        }
    }

    /// `getWorldZ(x, z)`.
    pub fn world_z(&self, x: i32, z: i32) -> i32 {
        match self.orientation {
            None => z,
            Some(Direction::NORTH) => self.bb.max_z - z,
            Some(Direction::SOUTH) => self.bb.min_z + z,
            Some(Direction::WEST) | Some(Direction::EAST) => self.bb.min_z + x,
            _ => z,
        }
    }

    /// `getWorldPos(x, y, z)`.
    pub fn world_pos(&self, x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos::new(self.world_x(x, z), self.world_y(y), self.world_z(x, z))
    }

    /// `getLocatorPosition`.
    pub fn locator_position(&self) -> BlockPos {
        BlockPos::new(
            self.bb.get_center().x,
            self.bb.min_y,
            self.bb.get_center().z,
        )
    }
}

/// `placeBlock(level, state, x, y, z, boundingBox)` —
/// mirror/rotation применяются к state.
#[allow(clippy::too_many_arguments)]
pub fn place_block(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    mut state: BlockState,
    x: i32,
    y: i32,
    z: i32,
    box_: &BoundingBox,
) {
    let t = PieceTransform { bb, orientation };
    let world_pos = t.world_pos(x, y, z);
    if box_.is_inside_pos(world_pos) && can_be_replaced_default(level, x, y, z, bb, t) {
        if mirror != Mirror::NONE {
            state = state_mirror(state, mirror);
        }
        if rotation != Rotation::NONE {
            state = state_rotate(state, rotation);
        }
        level.set_block_state(world_pos, &state);
        let fluid = state.get_fluid_state();
        if !fluid.is_empty() {
            level.schedule_tick(world_pos, &fluid.fluid.id.clone(), 0);
        }
        if is_shape_check_block(&state) {
            level.mark_pos_for_postprocessing(world_pos);
        }
    }
}

/// `SHAPE_CHECK_BLOCKS` — fence/torch/ladder/iron_bars.
pub fn is_shape_check_block(state: &BlockState) -> bool {
    const IDS: [&str; 11] = [
        "nether_brick_fence", "torch", "wall_torch", "oak_fence", "spruce_fence",
        "dark_oak_fence", "pale_oak_fence", "acacia_fence", "birch_fence",
        "jungle_fence", "ladder",
    ];
    // iron_bars тоже в наборе — 12-й элемент; держим отдельно.
    IDS.iter().any(|id| state.block.id.path == **id)
        || state.block.id.path == "iron_bars"
}

/// `state.mirror(mirror)` — поворот значения `facing` для
/// горизонтальных направлений (полная OctahedralGroup — позже;
/// для StructurePiece-достаточно facing).
pub fn state_mirror(mut state: BlockState, mirror: Mirror) -> BlockState {
    if let Some(StateValue::Str(s)) = state.values.get_by_name("facing") {
        if let Some(dir) = Direction::by_name(&s) {
            let mirrored = match mirror {
                Mirror::NONE => dir,
                Mirror::LEFT_RIGHT => match dir {
                    Direction::NORTH => Direction::SOUTH,
                    Direction::SOUTH => Direction::NORTH,
                    Direction::EAST => Direction::WEST,
                    Direction::WEST => Direction::EAST,
                    other => other,
                },
                Mirror::FRONT_BACK => match dir {
                    Direction::EAST => Direction::WEST,
                    Direction::WEST => Direction::EAST,
                    Direction::NORTH => Direction::SOUTH,
                    Direction::SOUTH => Direction::NORTH,
                    other => other,
                },
            };
            state.values.set_erased("facing", StateValue::Str(mirrored.get_name().to_string()));
        }
    }
    state
}

/// `state.rotate(rotation)` — вращение `facing`.
pub fn state_rotate(mut state: BlockState, rotation: Rotation) -> BlockState {
    if let Some(StateValue::Str(s)) = state.values.get_by_name("facing") {
        if let Some(dir) = Direction::by_name(&s) {
            let rotated = match rotation {
                Rotation::NONE => dir,
                Rotation::CLOCKWISE_90 => dir.get_clock_wise(),
                Rotation::CLOCKWISE_180 => dir.get_clock_wise().get_clock_wise(),
                Rotation::COUNTERCLOCKWISE_90 => dir.get_opposite().get_clock_wise(),
            };
            state.values.set_erased("facing", StateValue::Str(rotated.get_name().to_string()));
        }
    }
    state
}

use crate::world::level::block::state::property::StateValue;

/// `canBeReplaced` — дефолт true (наследники переопределяют).
pub fn can_be_replaced_default(
    _level: &dyn LevelReader,
    _x: i32,
    _y: i32,
    _z: i32,
    _bb: &BoundingBox,
    _t: PieceTransform,
) -> bool {
    true
}

/// `getBlock(level, x, y, z, box)` — air вне box.
pub fn get_block(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    x: i32,
    y: i32,
    z: i32,
    box_: &BoundingBox,
) -> BlockState {
    let t = PieceTransform { bb, orientation };
    let world_pos = t.world_pos(x, y, z);
    if !box_.is_inside_pos(world_pos) {
        BlockState::air()
    } else {
        level.get_block_state(world_pos).unwrap_or_else(BlockState::air)
    }
}

/// `isInterior(level, x, y, z, box)` — ниже OCEAN_FLOOR_WG.
pub fn is_interior(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    x: i32,
    y: i32,
    z: i32,
    box_: &BoundingBox,
) -> bool {
    let t = PieceTransform { bb, orientation };
    let world_pos = t.world_pos(x, y + 1, z);
    box_.is_inside_pos(world_pos)
        && world_pos.y
            < level.get_height(
                crate::world::level::levelgen::heightmap::HeightmapType::OceanFloorWg,
                world_pos.x,
                world_pos.z,
            )
}

/// `generateAirBox`.
#[allow(clippy::too_many_arguments)]
pub fn generate_air_box(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
) {
    for i in min_y..=max_y {
        for i1 in min_x..=max_x {
            for i2 in min_z..=max_z {
                place_block(level, bb, orientation, mirror, rotation, BlockState::air(), i1, i, i2, box_);
            }
        }
    }
}

/// `generateBox` (boundary/inside варианты).
#[allow(clippy::too_many_arguments)]
pub fn generate_box(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    boundary: &BlockState,
    inside: &BlockState,
    existing_only: bool,
) {
    for i in min_y..=max_y {
        for i1 in min_x..=max_x {
            for i2 in min_z..=max_z {
                if !existing_only
                    || !get_block(level, bb, orientation, i1, i, i2, box_).is_air()
                {
                    let interior = i != min_y && i != max_y && i1 != min_x && i1 != max_x && i2 != min_z && i2 != max_z;
                    let state = if interior { inside } else { boundary };
                    place_block(level, bb, orientation, mirror, rotation, state.clone(), i1, i, i2, box_);
                }
            }
        }
    }
}

/// `generateBox` по BoundingBox-аргументу.
#[allow(clippy::too_many_arguments)]
pub fn generate_box_bb(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    inner: &BoundingBox,
    boundary: &BlockState,
    inside: &BlockState,
    existing_only: bool,
) {
    generate_box(
        level, bb, orientation, mirror, rotation, box_,
        inner.min_x, inner.min_y, inner.min_z,
        inner.max_x, inner.max_y, inner.max_z,
        boundary, inside, existing_only,
    );
}

/// `generateBox` (BlockSelector-вариант).
#[allow(clippy::too_many_arguments)]
pub fn generate_box_selector(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    always_replace: bool,
    random: &mut dyn RandomSource,
    block_selector: &mut dyn BlockSelector,
) {
    for i in min_y..=max_y {
        for i1 in min_x..=max_x {
            for i2 in min_z..=max_z {
                if !always_replace
                    || !get_block(level, bb, orientation, i1, i, i2, box_).is_air()
                {
                    let wall = i == min_y || i == max_y || i1 == min_x || i1 == max_x || i2 == min_z || i2 == max_z;
                    block_selector.next(random, i1, i, i2, wall);
                    place_block(
                        level, bb, orientation, mirror, rotation,
                        block_selector.get_next().clone(), i1, i, i2, box_,
                    );
                }
            }
        }
    }
}

/// `generateBox` selector-вариант по BoundingBox.
#[allow(clippy::too_many_arguments)]
pub fn generate_box_selector_bb(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    inner: &BoundingBox,
    always_replace: bool,
    random: &mut dyn RandomSource,
    block_selector: &mut dyn BlockSelector,
) {
    generate_box_selector(
        level, bb, orientation, mirror, rotation, box_,
        inner.min_x, inner.min_y, inner.min_z,
        inner.max_x, inner.max_y, inner.max_z,
        always_replace, random, block_selector,
    );
}

/// `generateMaybeBox`.
#[allow(clippy::too_many_arguments)]
pub fn generate_maybe_box(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    random: &mut dyn RandomSource,
    chance: f32,
    x1: i32,
    y1: i32,
    z1: i32,
    x2: i32,
    y2: i32,
    z2: i32,
    edge_state: &BlockState,
    state: &BlockState,
    require_non_air: bool,
    require_skylight: bool,
) {
    for i in y1..=y2 {
        for i1 in x1..=x2 {
            for i2 in z1..=z2 {
                if random.next_float() <= chance
                    && (!require_non_air || !get_block(level, bb, orientation, i1, i, i2, box_).is_air())
                    && (!require_skylight || is_interior(level, bb, orientation, i1, i, i2, box_))
                {
                    let interior = i != y1 && i != y2 && i1 != x1 && i1 != x2 && i2 != z1 && i2 != z2;
                    let s = if interior { state } else { edge_state };
                    place_block(level, bb, orientation, mirror, rotation, s.clone(), i1, i, i2, box_);
                }
            }
        }
    }
}

/// `maybeGenerateBlock`.
#[allow(clippy::too_many_arguments)]
pub fn maybe_generate_block(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    random: &mut dyn RandomSource,
    chance: f32,
    x: i32,
    y: i32,
    z: i32,
    state: &BlockState,
) {
    if random.next_float() < chance {
        place_block(level, bb, orientation, mirror, rotation, state.clone(), x, y, z, box_);
    }
}

/// `generateUpperHalfSphere`.
#[allow(clippy::too_many_arguments)]
pub fn generate_upper_half_sphere(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    state: &BlockState,
    exclude_air: bool,
) {
    let f = (max_x - min_x + 1) as f32;
    let f1 = (max_y - min_y + 1) as f32;
    let f2 = (max_z - min_z + 1) as f32;
    let f3 = min_x as f32 + f / 2.0;
    let f4 = min_z as f32 + f2 / 2.0;
    for i in min_y..=max_y {
        let f5 = (i - min_y) as f32 / f1;
        for i1 in min_x..=max_x {
            let f6 = (i1 as f32 - f3) / (f * 0.5);
            for i2 in min_z..=max_z {
                let f7 = (i2 as f32 - f4) / (f2 * 0.5);
                if !exclude_air
                    || !get_block(level, bb, orientation, i1, i, i2, box_).is_air()
                {
                    let f8 = f6 * f6 + f5 * f5 + f7 * f7;
                    if f8 <= 1.05 {
                        place_block(level, bb, orientation, mirror, rotation, state.clone(), i1, i, i2, box_);
                    }
                }
            }
        }
    }
}

/// `fillColumnDown`.
#[allow(clippy::too_many_arguments)]
pub fn fill_column_down(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    state: &BlockState,
    x: i32,
    y: i32,
    z: i32,
) {
    let t = PieceTransform { bb, orientation };
    let mut world_pos = t.world_pos(x, y, z);
    if box_.is_inside_pos(world_pos) {
        while is_replaceable_by_structures(
            &level.get_block_state(world_pos).unwrap_or_else(BlockState::air),
        ) && world_pos.y > level.min_build_height() + 1
        {
            level.set_block_state(world_pos, state);
            world_pos = world_pos.below();
        }
    }
}

/// `isReplaceableByStructures`.
pub fn is_replaceable_by_structures(state: &BlockState) -> bool {
    state.is_air()
        || !state.get_fluid_state().is_empty()
        || state.block.id.path == "glow_lichen"
        || state.block.id.path == "seagrass"
        || state.block.id.path == "tall_seagrass"
}

/// `createChest(level, box, random, x, y, z, lootTable)`.
#[allow(clippy::too_many_arguments)]
pub fn create_chest(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    random: &mut dyn RandomSource,
    x: i32,
    y: i32,
    z: i32,
    loot_table: &Identifier,
) -> bool {
    let t = PieceTransform { bb, orientation };
    let world_pos = t.world_pos(x, y, z);
    if box_.is_inside_pos(world_pos)
        && !level
            .get_block_state(world_pos)
            .map(|s| s.block.id.path == "chest")
            .unwrap_or(false)
    {
        let state = reorient(level, world_pos, BlockState::air());
        level.set_block_state(world_pos, &state);
        level.place_loot_container(world_pos, loot_table, random.next_long());
        true
    } else {
        false
    }
}

/// `createChest(level, box, random, pos, lootTable, state)` —
/// вариант с явной позицией.
pub fn create_chest_at(
    level: &dyn LevelReader,
    box_: &BoundingBox,
    random: &mut dyn RandomSource,
    pos: BlockPos,
    loot_table: &Identifier,
    state: Option<&BlockState>,
) -> bool {
    if box_.is_inside_pos(pos)
        && !level
            .get_block_state(pos)
            .map(|s| s.block.id.path == "chest")
            .unwrap_or(false)
    {
        let state = state.cloned().unwrap_or_else(|| reorient(level, pos, BlockState::air()));
        level.set_block_state(pos, &state);
        level.place_loot_container(pos, loot_table, random.next_long());
        true
    } else {
        false
    }
}

/// `reorient(level, pos, state)` — поворот `facing` относительно
/// соседних твёрдых блоков (для сундуков).
pub fn reorient(level: &dyn LevelReader, pos: BlockPos, mut state: BlockState) -> BlockState {
    let mut direction: Option<Direction> = None;
    for d in Plane::FACES_HORIZONTAL {
        let npos = pos.relative(d);
        let solid = level
            .get_block_state(npos)
            .map(|s| s.is_solid_render(level, npos))
            .unwrap_or(false);
        if level
            .get_block_state(npos)
            .map(|s| s.block.id.path == "chest")
            .unwrap_or(false)
        {
            return state;
        }
        if solid {
            if direction.is_some() {
                direction = None;
                break;
            }
            direction = Some(d);
        }
    }
    let facing = if let Some(d) = direction {
        d.get_opposite()
    } else {
        let mut d2 = level
            .get_block_state(pos)
            .and_then(|s| s.values.get_by_name("facing"))
            .and_then(|v| match v {
                StateValue::Str(s) => Direction::by_name(&s),
                _ => None,
            })
            .unwrap_or(Direction::NORTH);
        let mut p = pos.relative(d2);
        let solid = |lv: &dyn LevelReader, pp: BlockPos| {
            lv.get_block_state(pp).map(|s| s.is_solid_render(lv, pp)).unwrap_or(false)
        };
        if solid(level, p) {
            d2 = d2.get_opposite();
            p = pos.relative(d2);
        }
        if solid(level, p) {
            d2 = d2.get_clock_wise();
            p = pos.relative(d2);
        }
        if solid(level, p) {
            d2 = d2.get_opposite();
            p = pos.relative(d2);
        }
        let _ = p;
        d2
    };
    state.values.set_erased("facing", StateValue::Str(facing.get_name().to_string()));
    state
}

/// `createDispenser`.
#[allow(clippy::too_many_arguments)]
pub fn create_dispenser(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    random: &mut dyn RandomSource,
    x: i32,
    y: i32,
    z: i32,
    facing: Direction,
    loot_table: &Identifier,
) -> bool {
    let t = PieceTransform { bb, orientation };
    let world_pos = t.world_pos(x, y, z);
    if box_.is_inside_pos(world_pos)
        && !level
            .get_block_state(world_pos)
            .map(|s| s.block.id.path == "dispenser")
            .unwrap_or(false)
    {
        let mut state = level
            .get_block_state(world_pos)
            .unwrap_or_else(BlockState::air);
        state.values.set_erased("facing", StateValue::Str(facing.get_name().to_string()));
        place_block(level, bb, orientation, mirror, rotation, state, x, y, z, box_);
        level.place_loot_container(world_pos, loot_table, random.next_long());
        true
    } else {
        false
    }
}

/// `StructurePiece.BlockSelector`.
pub trait BlockSelector {
    /// `next(random, x, y, z, wall)`.
    fn next(&mut self, random: &mut dyn RandomSource, x: i32, y: i32, z: i32, wall: bool);
    /// `getNext`.
    fn get_next(&self) -> &BlockState;
}

/// `move(x, y, z)` — сдвиг bounding box.
pub fn piece_move(bb: &mut BoundingBox, x: i32, y: i32, z: i32) {
    bb.move_(x, y, z);
}

/// `createBoundingBox(pieces)` — объединяющий бокс кусков.
pub fn create_bounding_box(pieces: &[&dyn StructurePiece]) -> BoundingBox {
    BoundingBox::encapsulating_boxes(pieces.iter().map(|p| p.bounding_box()).collect::<Vec<_>>().iter())
        .expect("Unable to calculate boundingbox without pieces")
}

/// `findCollisionPiece(pieces, boundingBox)`.
pub fn find_collision_piece<'a>(
    pieces: &'a [Box<dyn StructurePiece>],
    bounding_box: &BoundingBox,
) -> Option<&'a dyn StructurePiece> {
    pieces
        .iter()
        .map(|b| b.as_ref())
        .find(|p| p.bounding_box().intersects(bounding_box))
}

/// `isCloseToChunk(chunkPos, distance)`.
pub fn is_close_to_chunk(bb: &BoundingBox, chunk_pos: ChunkPos, distance: i32) -> bool {
    let min_x = chunk_pos.get_min_block_x();
    let min_z = chunk_pos.get_min_block_z();
    bb.intersects_2d(min_x - distance, min_z - distance, min_x + 15 + distance, min_z + 15 + distance)
}
