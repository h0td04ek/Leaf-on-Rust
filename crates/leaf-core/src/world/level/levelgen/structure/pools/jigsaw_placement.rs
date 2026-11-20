//! Перенос `JigsawPlacement` — генератор jigsaw-структур
//! (addPieces + Placer::tryPlacingChildren).
//!
//! Отличия от Java:
//! - `ChunkGenerator`/`RegistryAccess`/`LevelHeightAccessor` —
//!   швы `PoolGeneratorAccess`/`PoolRegistryAccess`/`LevelReader`.
//! - `MutableObject<VoxelShape>` — `Rc<RefCell<Option<VoxelShape>>>`.
//! - `BooleanOp.ONLY_FIRST/ONLY_SECOND` — семантика разности
//!   (`a \ b` / `b \ a`) через `BooleanOp::Not` (см. helpers).

use super::empty_pool_element::empty_element_rc;
use super::structure_pool_element::PoolGeneratorAccess;
use super::structure_template_pool::{JigsawJunction, Projection, StructureTemplatePool};
use super::structure_pool_element::StructurePoolElement;
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::holder::Holder;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::world::level::block::jigsaw_block::JigsawBlock;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::structure::pool_element_structure_piece::PoolElementStructurePiece;
use crate::world::level::levelgen::structure::structure::GenerationStub;
use crate::world::level::levelgen::structure::structure_piece::StructurePiece;
use crate::world::level::levelgen::structure::templatesystem::liquid_settings::LiquidSettings;
use crate::world::level::levelgen::structure::templatesystem::structure_template::{
    JigsawBlockInfo, StructureBlockInfo, StructureTemplate,
};
use crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::shapes::{BooleanOp, VoxelShape};
use std::cell::RefCell;
use std::rc::Rc;

/// `UNSET_HEIGHT`.
const UNSET_HEIGHT: i32 = i32::MIN;

/// `JigsawStructure.MaxDistance` — макс. расстояние от центра.
#[derive(Clone, Copy, Debug)]
pub struct JigsawMaxDistance {
    /// `horizontal`.
    pub horizontal: i32,
    /// `vertical`.
    pub vertical: i32,
}

impl JigsawMaxDistance {
    /// `new`.
    pub fn new(horizontal: i32, vertical: i32) -> Self {
        Self { horizontal, vertical }
    }
}

/// `DimensionPadding` — отступ от границ измерения.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DimensionPadding {
    /// `bottom`.
    pub bottom: i32,
    /// `top`.
    pub top: i32,
}

impl DimensionPadding {
    /// `ZERO`.
    pub const ZERO: Self = Self { bottom: 0, top: 0 };
    /// `new(padding)` — равные top/bottom.
    pub fn uniform(padding: i32) -> Self {
        Self { bottom: padding, top: padding }
    }
}

/// `PoolAliasLookup` — резолв pool-ключей (alias-подсистема).
pub trait PoolAliasLookup {
    /// `lookup(key)` → фактический ключ пула.
    fn lookup(
        &self,
        key: ResourceKey<Rc<StructureTemplatePool>>,
    ) -> ResourceKey<Rc<StructureTemplatePool>>;
}

/// `PoolAliasLookup.EMPTY` — identity.
pub struct EmptyPoolAliasLookup;

impl PoolAliasLookup for EmptyPoolAliasLookup {
    fn lookup(
        &self,
        key: ResourceKey<Rc<StructureTemplatePool>>,
    ) -> ResourceKey<Rc<StructureTemplatePool>> {
        key
    }
}

/// Реестр `Registry<StructureTemplatePool>` — доступ к пулам
/// по ключу (реализация — с registry-слоем).
pub trait PoolRegistryAccess {
    /// `registry.getOptional(key)` → holder на пул.
    fn get_pool(
        &self,
        key: &ResourceKey<Rc<StructureTemplatePool>>,
    ) -> Option<Holder<Rc<StructureTemplatePool>>>;
}

/// Ключ реестра `worldgen/template_pool`.
pub fn template_pool_registry_key() -> ResourceKey<crate::core::registry::Registry<Rc<StructureTemplatePool>>> {
    ResourceKey::create_registry_key(Identifier::minecraft("worldgen/template_pool"))
}

/// `ResourceKey` для id пула (`jigsaw.pool` → key).
pub fn pool_key(id: &str) -> Option<ResourceKey<Rc<StructureTemplatePool>>> {
    let identifier = Identifier::parse(id).ok()?;
    Some(ResourceKey::create_with_registry_name(
        Identifier::minecraft("worldgen/template_pool"),
        identifier,
    ))
}

/// `JigsawContext` — контекст `addPieces` (швы вместо
/// GenerationContext/ChunkGenerator/RegistryAccess).
pub struct JigsawContext<'a> {
    /// `level` — LevelHeightAccessor/WorldGenLevel.
    pub level: &'a dyn LevelReader,
    /// `chunkGenerator`.
    pub generator: &'a dyn PoolGeneratorAccess,
    /// `structureTemplateManager` (shared — pieces держат Arc).
    pub template_manager: std::sync::Arc<StructureTemplateManager>,
    /// `pools` — registry template_pool.
    pub pools: &'a dyn PoolRegistryAccess,
    /// `random`.
    pub random: &'a mut dyn RandomSource,
    /// `chunkPos`.
    pub chunk_pos: ChunkPos,
}

/// `AABB.of(BoundingBox)`.
fn aabb_of_bb(bb: &crate::world::level::levelgen::structure::bounding_box::BoundingBox) -> Aabb {
    Aabb::new(
        bb.min_x as f64,
        bb.min_y as f64,
        bb.min_z as f64,
        (bb.max_x + 1) as f64,
        (bb.max_y + 1) as f64,
        (bb.max_z + 1) as f64,
    )
}

/// `Shapes.create(AABB)` → VoxelShape.
fn shape_of_aabb(aabb: &Aabb) -> VoxelShape {
    VoxelShape::of(aabb.clone())
}

/// `ONLY_FIRST(a,b)` = `a \ b` (vanilla семантика: части первого
/// вне второго).
fn join_only_first(a: &VoxelShape, b: &VoxelShape) -> VoxelShape {
    a.join(b, BooleanOp::Not)
}

/// `ONLY_SECOND(a,b)` = `b \ a`.
fn join_only_second(a: &VoxelShape, b: &VoxelShape) -> VoxelShape {
    b.join(a, BooleanOp::Not)
}

/// `Shapes.joinIsNotEmpty(a, b, ONLY_SECOND)`.
fn join_is_not_empty_only_second(a: &VoxelShape, b: &VoxelShape) -> bool {
    !join_only_second(a, b).is_empty()
}

/// `VoxelShape.deflate(v)`.
fn deflate_shape(shape: &VoxelShape, v: f64) -> VoxelShape {
    shape.deflate(v)
}

/// `Shapes.create(AABB.of(bb).deflate(0.25))`.
fn deflated_bb_shape(bb: &crate::world::level::levelgen::structure::bounding_box::BoundingBox) -> VoxelShape {
    deflate_shape(&shape_of_aabb(&aabb_of_bb(bb)), 0.25)
}

/// `JigsawPlacement.addPieces` — точка входа генерации.
///
/// В Java генерация кусков откладывается до `GenerationStub`-
/// builder'а; у нас `GenerationStub` требует `'static`, поэтому
/// placer запускается eagerly (детерминировано — тот же
/// random-поток, та же последовательность).
#[allow(clippy::too_many_arguments)]
pub fn add_pieces(
    context: &mut JigsawContext,
    start_pool: Holder<Rc<StructureTemplatePool>>,
    start_jigsaw_name: Option<Identifier>,
    max_depth: i32,
    pos: BlockPos,
    use_expansion_hack: bool,
    project_start_to_heightmap: Option<HeightmapType>,
    max_distance_from_center: JigsawMaxDistance,
    alias_lookup: &dyn PoolAliasLookup,
    dimension_padding: DimensionPadding,
    liquid_settings: LiquidSettings,
) -> Option<GenerationStub> {
    let rotation = Rotation::get_random(context.random);
    let start_key = start_pool.key().cloned();
    let resolved_pool = match start_key {
        Some(key) => context
            .pools
            .get_pool(&alias_lookup.lookup(key))
            .unwrap_or_else(|| start_pool.clone()),
        None => start_pool.clone(),
    };
    let pool_rc = resolved_pool.value().cloned()?;
    let random_template = pool_rc.get_random_template(context.random);
    if random_template.is_empty_element() {
        return None;
    }

    let block_pos = if let Some(identifier) = start_jigsaw_name {
        match get_random_named_jigsaw(
            random_template.as_ref(),
            &identifier,
            pos,
            rotation,
            &context.template_manager,
            context.random,
        ) {
            Some(p) => p,
            None => return None,
        }
    } else {
        pos
    };

    let vec3i = block_pos.to_vec3i().subtract(&pos.to_vec3i());
    let block_pos1 = pos.subtract(&vec3i);
    let start_piece = Rc::new(RefCell::new(PoolElementStructurePiece::new(
        context.template_manager.clone(),
        random_template.clone(),
        block_pos1,
        random_template.get_ground_level_delta(),
        rotation,
        random_template.get_bounding_box(&context.template_manager, block_pos1, rotation),
        liquid_settings,
    )));
    let bounding_box = start_piece.borrow().bounding_box();
    let i = (bounding_box.max_x + bounding_box.min_x) / 2;
    let i1 = (bounding_box.max_z + bounding_box.min_z) / 2;
    let i2 = match project_start_to_heightmap {
        None => block_pos1.y,
        Some(hm) => {
            pos.y + context.generator.get_first_free_height(
                hm,
                i,
                i1,
                context.level,
            )
        }
    };
    let i3 = bounding_box.min_y + start_piece.borrow().ground_level_delta;
    start_piece.borrow_mut().move_by(0, i2 - i3, 0);
    if is_start_too_close_to_world_height_limits(
        context.level,
        dimension_padding,
        &start_piece.borrow().bounding_box(),
    ) {
        return None;
    }
    let i4 = i2 + vec3i.y;

    let mut pieces = Vec::new();
    if max_depth > 0 {
        let level = context.level;
        let min_y = level.min_build_height();
        let max_y = min_y + level.height();
        let bb = start_piece.borrow().bounding_box();
        let aabb = Aabb::new(
            (i - max_distance_from_center.horizontal) as f64,
            (i4 - max_distance_from_center.vertical).max(min_y + dimension_padding.bottom) as f64,
            (i1 - max_distance_from_center.horizontal) as f64,
            (i + max_distance_from_center.horizontal + 1) as f64,
            (i4 + max_distance_from_center.vertical + 1).min(max_y + 1 - dimension_padding.top) as f64,
            (i1 + max_distance_from_center.horizontal + 1) as f64,
        );
        // free = region ONLY_FIRST startBox = region \ startBox.
        let region = shape_of_aabb(&aabb);
        let start_shape = shape_of_aabb(&aabb_of_bb(&bb));
        let initial_free = join_only_first(&region, &start_shape);

        let generated = run_placer(
            context,
            start_piece.clone(),
            initial_free,
            max_depth,
            use_expansion_hack,
            alias_lookup,
            liquid_settings,
        );
        pieces = generated;
    } else {
        pieces.push(start_piece.clone());
    }

    Some(GenerationStub::new(
        BlockPos::new(i, i4, i1),
        move |out: &mut Vec<Box<dyn StructurePiece>>| {
            for piece in pieces {
                out.push(Box::new(RcPoolPiece(piece)));
            }
        },
    ))
}

/// `isStartTooCloseToWorldHeightLimits`.
fn is_start_too_close_to_world_height_limits(
    level: &dyn LevelReader,
    padding: DimensionPadding,
    bounding_box: &crate::world::level::levelgen::structure::bounding_box::BoundingBox,
) -> bool {
    if padding == DimensionPadding::ZERO {
        return false;
    }
    let i = level.min_build_height() + padding.bottom;
    let i1 = level.min_build_height() + level.height() - 1 - padding.top;
    bounding_box.min_y < i || bounding_box.max_y > i1
}

/// `getRandomNamedJigsaw`.
fn get_random_named_jigsaw(
    element: &dyn StructurePoolElement,
    start_jigsaw_name: &Identifier,
    pos: BlockPos,
    rotation: Rotation,
    structure_template_manager: &StructureTemplateManager,
    random: &mut dyn RandomSource,
) -> Option<BlockPos> {
    for jigsaw in element.get_shuffled_jigsaw_blocks(
        structure_template_manager,
        pos,
        rotation,
        random,
    ) {
        if jigsaw.name == start_jigsaw_name.path
            || jigsaw.name == start_jigsaw_name.to_string()
        {
            return Some(jigsaw.info.pos);
        }
    }
    None
}

/// `PieceState` — состояние placer'а.
struct PieceState {
    piece: Rc<RefCell<PoolElementStructurePiece>>,
    free: Rc<RefCell<Option<VoxelShape>>>,
    depth: i32,
}

/// `Placer.placing` — priority-итератор (priority desc, seq asc).
struct PlacingEntry {
    priority: i32,
    seq: u64,
    state: PieceState,
}

/// `JigsawPlacement.Placer`.
struct Placer<'a> {
    pools: &'a dyn PoolRegistryAccess,
    max_depth: i32,
    generator: &'a dyn PoolGeneratorAccess,
    template_manager: std::sync::Arc<StructureTemplateManager>,
    pieces: Vec<Rc<RefCell<PoolElementStructurePiece>>>,
    random: &'a mut dyn RandomSource,
    placing: Vec<PlacingEntry>,
    seq: u64,
}

impl<'a> Placer<'a> {
    /// `placing.add(state, priority)` — вставка по приоритету.
    fn placing_add(&mut self, state: PieceState, priority: i32) {
        let entry = PlacingEntry {
            priority,
            seq: self.seq,
            state,
        };
        self.seq += 1;
        let pos = self
            .placing
            .iter()
            .position(|e| e.priority < priority)
            .unwrap_or(self.placing.len());
        self.placing.insert(pos, entry);
    }

    /// `placing.next()` — извлечь элемент с наивысшим приоритетом.
    fn placing_next(&mut self) -> Option<PieceState> {
        if self.placing.is_empty() {
            None
        } else {
            Some(self.placing.remove(0).state)
        }
    }

    /// `tryPlacingChildren`.
    #[allow(clippy::too_many_arguments)]
    fn try_placing_children(
        &mut self,
        piece: Rc<RefCell<PoolElementStructurePiece>>,
        free: Rc<RefCell<Option<VoxelShape>>>,
        depth: i32,
        use_expansion_hack: bool,
        level: &dyn LevelReader,
        alias_lookup: &dyn PoolAliasLookup,
        liquid_settings: LiquidSettings,
    ) {
        let (element, position, rotation, projection, bounding_box, min_y, ground_delta) = {
            let p = piece.borrow();
            (
                p.element.clone(),
                p.position,
                p.rotation,
                p.element.get_projection(),
                p.bounding_box(),
                p.bounding_box().min_y,
                p.ground_level_delta,
            )
        };
        let flag = projection == Projection::Rigid;
        let mutable_object: Rc<RefCell<Option<VoxelShape>>> =
            Rc::new(RefCell::new(None));
        let mut height_for_level = UNSET_HEIGHT;

        'jigsaw_loop: for jigsaw_block_info in element.get_shuffled_jigsaw_blocks(
            &*self.template_manager,
            position,
            rotation,
            self.random,
        ) {
            let structure_block_info = &jigsaw_block_info.info;
            let front_facing = JigsawBlock::get_front_facing(&structure_block_info.state);
            let block_pos = structure_block_info.pos;
            let block_pos1 = block_pos.relative(front_facing);
            let i = block_pos.y - min_y;
            let Some(key) = jigsaw_block_info
                .pool
                .as_deref()
                .and_then(pool_key)
                .map(|k| alias_lookup.lookup(k))
            else {
                continue;
            };
            let Some(holder) = self.pools.get_pool(&key) else {
                continue;
            };
            let Some(pool) = holder.value().cloned() else {
                continue;
            };
            if pool.size() == 0 && !is_empty_pool_holder(&holder) {
                continue;
            }
            let fallback = pool.get_fallback().clone();
            let Some(fallback_pool) = fallback.value().cloned() else {
                continue;
            };
            if fallback_pool.size() == 0 && !is_empty_pool_holder(&fallback) {
                continue;
            }

            let is_inside = bounding_box.is_inside_pos(block_pos1);
            let mutable_object1: Rc<RefCell<Option<VoxelShape>>> = if is_inside {
                {
                    let mut mo = mutable_object.borrow_mut();
                    if mo.is_none() {
                        *mo = Some(shape_of_aabb(&aabb_of_bb(&bounding_box)));
                    }
                }
                mutable_object.clone()
            } else {
                free.clone()
            };

            let mut list: Vec<Rc<dyn StructurePoolElement>> = Vec::new();
            if depth != self.max_depth {
                list.extend(pool.get_shuffled_templates(self.random));
            }
            list.extend(fallback_pool.get_shuffled_templates(self.random));
            let placement_priority = jigsaw_block_info
                .info
                .nbt
                .as_ref()
                .and_then(|n| n.get_int("placement_priority"))
                .unwrap_or(0);

            for structure_pool_element in list {
                if structure_pool_element.is_empty_element() {
                    break;
                }
                for rotation1 in Rotation::get_shuffled(self.random) {
                    let shuffled_jigsaw_blocks = structure_pool_element
                        .get_shuffled_jigsaw_blocks(
                            &*self.template_manager,
                            BlockPos::ZERO,
                            rotation1,
                            self.random,
                        );
                    let bounding_box1 = structure_pool_element.get_bounding_box(
                        &*self.template_manager,
                        BlockPos::ZERO,
                        rotation1,
                    );
                    let mut i2 = 0;
                    if use_expansion_hack && bounding_box1.get_y_span() <= 16 {
                        i2 = shuffled_jigsaw_blocks
                            .iter()
                            .map(|info| {
                                let sbi = &info.info;
                                let front =
                                    JigsawBlock::get_front_facing(&sbi.state);
                                if !bounding_box1
                                    .is_inside_pos(sbi.pos.relative(front))
                                {
                                    return 0;
                                }
                                let Some(key1) = info
                                    .pool
                                    .as_deref()
                                    .and_then(pool_key)
                                    .map(|k| alias_lookup.lookup(k))
                                else {
                                    return 0;
                                };
                                let pool1 = self.pools.get_pool(&key1);
                                let fallback1 = pool1
                                    .as_ref()
                                    .and_then(|h| h.value())
                                    .map(|p| p.get_fallback().clone());
                                let i8 = pool1
                                    .as_ref()
                                    .and_then(|h| h.value())
                                    .map(|p| {
                                        p.get_max_size(&*self.template_manager)
                                    })
                                    .unwrap_or(0);
                                let i9 = fallback1
                                    .as_ref()
                                    .and_then(|h| h.value())
                                    .map(|p| {
                                        p.get_max_size(&*self.template_manager)
                                    })
                                    .unwrap_or(0);
                                i8.max(i9)
                            })
                            .max()
                            .unwrap_or(0);
                    }

                    for jigsaw_block_info1 in &shuffled_jigsaw_blocks {
                        if !JigsawBlock::can_attach(&jigsaw_block_info, jigsaw_block_info1) {
                            continue;
                        }
                        let block_pos2 = jigsaw_block_info1.info.pos;
                        let block_pos3 = block_pos1.subtract(&block_pos2.to_vec3i());
                        let bounding_box2 = structure_pool_element.get_bounding_box(
                            &*self.template_manager,
                            block_pos3,
                            rotation1,
                        );
                        let min_y1 = bounding_box2.min_y;
                        let projection1 = structure_pool_element.get_projection();
                        let flag1 = projection1 == Projection::Rigid;
                        let y = block_pos2.y;
                        let front_step_y =
                            front_facing.get_step_y();
                        let i3 = i - y + front_step_y;
                        let i4 = if flag && flag1 {
                            min_y + i3
                        } else {
                            if height_for_level == UNSET_HEIGHT {
                                height_for_level = self.generator.get_first_free_height(
                                    HeightmapType::WorldSurfaceWg,
                                    block_pos.x,
                                    block_pos.z,
                                    level,
                                );
                            }
                            height_for_level - y
                        };
                        let i5 = i4 - min_y1;
                        let mut bounding_box3 = bounding_box2.moved(0, i5, 0);
                        let block_pos4 = block_pos3.offset(0, i5, 0);
                        if i2 > 0 {
                            let max = (i2 + 1).max(bounding_box3.get_y_span());
                            bounding_box3 = bounding_box3.encapsulate_pos(BlockPos::new(
                                bounding_box3.min_x,
                                bounding_box3.min_y + max,
                                bounding_box3.min_z,
                            ));
                        }

                        // joinIsNotEmpty(free, bb3.deflate(0.25), ONLY_SECOND)
                        // → кандидат должен быть полностью внутри free.
                        let candidate_inside = {
                            let free_shape = mutable_object1.borrow();
                            match free_shape.as_ref() {
                                Some(free) => !join_is_not_empty_only_second(
                                    free,
                                    &deflated_bb_shape(&bounding_box3),
                                ),
                                None => true,
                            }
                        };
                        if !candidate_inside {
                            continue;
                        }

                        // free = free \ bb3 (ONLY_FIRST).
                        {
                            let mut cell = mutable_object1.borrow_mut();
                            let new_free = match cell.as_ref() {
                                Some(free) => join_only_first(
                                    free,
                                    &shape_of_aabb(&aabb_of_bb(&bounding_box3)),
                                ),
                                None => shape_of_aabb(&aabb_of_bb(&bounding_box3)),
                            };
                            *cell = Some(new_free);
                        }

                        let max = ground_delta;
                        let i6 = if flag1 {
                            max - i3
                        } else {
                            structure_pool_element.get_ground_level_delta()
                        };
                        let new_piece = Rc::new(RefCell::new(
                            PoolElementStructurePiece::new(
                                self.template_manager.clone(),
                                structure_pool_element.clone(),
                                block_pos4,
                                i6,
                                rotation1,
                                bounding_box3.clone(),
                                liquid_settings,
                            ),
                        ));
                        let i7 = if flag {
                            min_y + i
                        } else if flag1 {
                            i4 + y
                        } else {
                            if height_for_level == UNSET_HEIGHT {
                                height_for_level = self.generator.get_first_free_height(
                                    HeightmapType::WorldSurfaceWg,
                                    block_pos.x,
                                    block_pos.z,
                                    level,
                                );
                            }
                            height_for_level + i3 / 2
                        };
                        piece.borrow_mut().add_junction(JigsawJunction::new(
                            block_pos1.x,
                            i7 - i + max,
                            block_pos1.z,
                            i3,
                            projection1,
                        ));
                        new_piece.borrow_mut().add_junction(JigsawJunction::new(
                            block_pos.x,
                            i7 - y + i6,
                            block_pos.z,
                            -i3,
                            projection,
                        ));
                        self.pieces.push(new_piece.clone());
                        if depth + 1 <= self.max_depth {
                            self.placing_add(
                                PieceState {
                                    piece: new_piece,
                                    free: mutable_object1.clone(),
                                    depth: depth + 1,
                                },
                                placement_priority,
                            );
                        }
                        continue 'jigsaw_loop;
                    }
                }
            }
        }
    }
}

/// `holder.is(Pools.EMPTY)`-проверка — empty pool.
fn is_empty_pool_holder(
    holder: &Holder<Rc<StructureTemplatePool>>,
) -> bool {
    holder
        .value()
        .map(|p| p.size() == 0)
        .unwrap_or(false)
}

/// `addPieces` — внутренняя версия (после add_pieces-обёртки):
/// запускает Placer и возвращает список кусков.
#[allow(clippy::too_many_arguments)]
pub fn run_placer(
    context: &mut JigsawContext,
    start_piece: Rc<RefCell<PoolElementStructurePiece>>,
    initial_free: VoxelShape,
    max_depth: i32,
    use_expansion_hack: bool,
    alias_lookup: &dyn PoolAliasLookup,
    liquid_settings: LiquidSettings,
) -> Vec<Rc<RefCell<PoolElementStructurePiece>>> {
    let mut placer = Placer {
        pools: context.pools,
        max_depth,
        generator: context.generator,
        template_manager: context.template_manager.clone(),
        pieces: vec![start_piece.clone()],
        random: context.random,
        placing: Vec::new(),
        seq: 0,
    };
    let free_cell = Rc::new(RefCell::new(Some(initial_free)));
    placer.try_placing_children(
        start_piece,
        free_cell.clone(),
        0,
        use_expansion_hack,
        context.level,
        alias_lookup,
        liquid_settings,
    );
    while let Some(state) = placer.placing_next() {
        placer.try_placing_children(
            state.piece,
            state.free,
            state.depth,
            use_expansion_hack,
            context.level,
            alias_lookup,
            liquid_settings,
        );
    }
    placer.pieces
}

/// `Rc<RefCell<PoolElementStructurePiece>>` → `Box<dyn
/// StructurePiece>` (для StructurePiecesBuilder).
pub struct RcPoolPiece(pub Rc<RefCell<PoolElementStructurePiece>>);

impl StructurePiece for RcPoolPiece {
    fn bounding_box(&self) -> crate::world::level::levelgen::structure::bounding_box::BoundingBox {
        self.0.borrow().bounding_box()
    }
    fn gen_depth(&self) -> i32 {
        self.0.borrow().gen_depth()
    }
    fn set_gen_depth(&mut self, gen_depth: i32) {
        self.0.borrow_mut().set_gen_depth(gen_depth);
    }
    fn orientation(&self) -> Option<Direction> {
        self.0.borrow().orientation()
    }
    fn set_orientation(&mut self, orientation: Option<Direction>) {
        self.0.borrow_mut().set_orientation(orientation);
    }
    fn rotation(&self) -> Rotation {
        self.0.borrow().rotation()
    }
    fn mirror(&self) -> crate::world::level::block::mirror::Mirror {
        self.0.borrow().mirror()
    }
    fn piece_type_id(&self) -> Identifier {
        self.0.borrow().piece_type_id()
    }
    fn add_additional_save_data(&self, tag: &mut crate::nbt::compound::CompoundTag) {
        self.0.borrow().add_additional_save_data(tag);
    }
    fn load_additional_save_data(&mut self, tag: &crate::nbt::compound::CompoundTag) {
        self.0.borrow_mut().load_additional_save_data(tag);
    }
    fn add_children(
        &mut self,
        piece: &mut dyn StructurePiece,
        pieces: &mut dyn crate::world::level::levelgen::structure::structure_piece::StructurePieceAccessor,
        random: &mut dyn RandomSource,
    ) {
        self.0.borrow_mut().add_children(piece, pieces, random);
    }
    fn post_process(
        &mut self,
        level: &dyn LevelReader,
        structure_manager: &dyn crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess,
        generator: Option<&dyn PoolGeneratorAccess>,
        random: &mut dyn RandomSource,
        box_: &crate::world::level::levelgen::structure::bounding_box::BoundingBox,
        chunk_pos: ChunkPos,
        pos: BlockPos,
    ) {
        self.0.borrow_mut().post_process(
            level, structure_manager, generator, random, box_, chunk_pos, pos,
        );
    }
}
