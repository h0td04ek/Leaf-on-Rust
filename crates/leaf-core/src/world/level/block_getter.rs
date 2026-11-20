//! `net.minecraft.world.level.BlockGetter` — чтение блоков/жидкостей/
//! block entities + рейкасты (`clip`, `isBlockInLine`,
//! `forEachBlockIntersectedBetween`, `traverseBlocks`).
//!
//! В Java `LevelReader extends BlockAndTintGetter extends
//! BlockGetter`. Наш `LevelReader` — отдельный мега-трейт;
//! `BlockGetter` портируется как самостоятельная иерархия,
//! а `Level` будет реализовывать оба. Методы, которым нужны
//! `BlockState`-формы (`&dyn LevelReader`), получают уровень
//! через `as_level_reader()` (default `None` — для
//! `EmptyBlockGetter`, где все состояния air/пустые).

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::direction::{Axis, Direction};
use crate::core::holder_set::HolderSet;
use crate::resources::identifier::Identifier;
use crate::util::mth;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::clip_block_state_context::ClipBlockStateContext;
use crate::world::level::clip_context::{ClipBlockMode, ClipContext, ClipFluidMode};
use crate::world::level::level_height_accessor::LevelHeightAccessor;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::hit_result::BlockHitResult;
use crate::world::phys::shapes::voxel_shape::VoxelShape;
use crate::world::phys::vec3::Vec3;

/// `BlockGetter`.
pub trait BlockGetter: LevelHeightAccessor {
    /// `getBlockEntity(pos)`.
    fn get_block_entity(
        &self,
        pos: BlockPos,
    ) -> Option<Arc<dyn crate::world::level::block::entity::block_entity::BlockEntityAccess>>;

    /// `getBlockEntity(pos, type)` — default → фильтр по типу.
    fn get_block_entity_typed(
        &self,
        pos: BlockPos,
        type_: &crate::world::level::block::entity::block_entity_type::BlockEntityType,
    ) -> Option<crate::world::level::block::entity::block_entity::BlockEntityRef> {
        self.get_block_entity(pos)
            .filter(|be| be.block_entity_type_id() == &type_.id)
    }

    /// `getBlockState(pos)` — `None` = блок не загружен
    /// (Paper `@Nullable` семантика).
    fn get_block_state(&self, pos: BlockPos) -> Option<BlockState>;

    /// `getBlockStateIfLoaded(pos)` (Paper) — без загрузки чанка.
    /// Default → `getBlockState`.
    fn get_block_state_if_loaded(&self, pos: BlockPos) -> Option<BlockState> {
        self.get_block_state(pos)
    }

    /// `getBlockIfLoaded(pos)` — `Option<Arc<Block>>`.
    fn get_block_if_loaded(&self, pos: BlockPos) -> Option<Arc<crate::world::level::block::Block>> {
        self.get_block_state_if_loaded(pos).map(|s| s.block.clone())
    }

    /// `getFluidIfLoaded(pos)` (Paper).
    fn get_fluid_if_loaded(&self, pos: BlockPos) -> Option<FluidState> {
        self.get_fluid_state(pos)
    }

    /// `getFluidState(pos)`.
    fn get_fluid_state(&self, pos: BlockPos) -> Option<FluidState>;

    /// Доступ к `LevelReader` для shape/clip вызовов
    /// (`BlockState::get_shape` принимает `&dyn LevelReader`).
    /// Default `None` — реализации без LevelReader
    /// (`EmptyBlockGetter`) не достигают shape-логики.
    fn as_level_reader(&self) -> Option<&dyn LevelReader> {
        None
    }

    /// `getLightEmission(pos)`.
    fn get_light_emission(&self, pos: BlockPos) -> u32 {
        self.get_block_state(pos)
            .map(|s| s.get_light_emission() as u32)
            .unwrap_or(0)
    }

    /// `getBlockStates(AABB)` — все состояния в области.
    fn get_block_states(&self, area: &Aabb) -> Vec<BlockState> {
        BlockPos::between_closed_aabb(area)
            .filter_map(|pos| self.get_block_state(pos))
            .collect()
    }

    /// `isBlockInLine(context)` — `traverseBlocks` до первого
    /// состояния, проходящего предикат.
    fn is_block_in_line(&self, context: &ClipBlockStateContext) -> BlockHitResult {
        let from = context.from().clone();
        let to = context.to().clone();
        traverse_blocks(
            &from,
            &to,
            context,
            &|ctx: &ClipBlockStateContext, pos: BlockPos| {
                let state = self.get_block_state(pos)?;
                let vec = ctx.from().subtract_vec(&ctx.to());
                if ctx.is_target_block()(&state) {
                    Some(BlockHitResult::new(
                        ctx.to().clone(),
                        Direction::get_approximate_nearest_vec(&vec),
                        BlockPos::containing(ctx.to().x, ctx.to().y, ctx.to().z),
                        false,
                    ))
                } else {
                    None
                }
            },
            &|ctx: &ClipBlockStateContext| {
                let vec = ctx.from().subtract_vec(&ctx.to());
                BlockHitResult::miss(
                    ctx.to().clone(),
                    Direction::get_approximate_nearest_vec(&vec),
                    BlockPos::containing(ctx.to().x, ctx.to().y, ctx.to().z),
                )
            },
        )
    }

    /// `clip(context, pos)` (CraftBukkit) —
    /// `Option<BlockHitResult>` (`None` = продолжить обход).
    #[allow(clippy::too_many_arguments)]
    fn clip_at(
        &self,
        context: &ClipContext,
        pos: BlockPos,
        can_collide: Option<&dyn Fn(&BlockState) -> bool>,
    ) -> Option<BlockHitResult> {
        let state = self.get_block_state_if_loaded(pos)?;
        if state.is_air() {
            return None;
        }
        // Paper `canCollide` — предикат по bukkit Block; наш
        // эквивалент — предикат по BlockState (вызывается только
        // при наличии LevelAccessor — здесь применяем всегда,
        // если предикат задан).
        if let Some(pred) = can_collide {
            if !pred(&state) {
                return None;
            }
        }
        let level = self.as_level_reader()?;
        let block_hit = {
            let shape = context.block_shape(&state, level, pos);
            self.clip_with_interaction_override(
                context.from(),
                context.to(),
                pos,
                &shape,
                &state,
            )
        };
        let fluid_state = state.get_fluid_state();
        let fluid_shape = context.fluid_shape(&fluid_state, level, pos);
        let fluid_hit = fluid_shape.clip(context.from(), context.to(), pos);
        let d_block = block_hit
            .as_ref()
            .map(|h| h.location.distance_to_sqr(context.from()))
            .unwrap_or(f64::MAX);
        let d_fluid = fluid_hit
            .as_ref()
            .map(|h| h.location.distance_to_sqr(context.from()))
            .unwrap_or(f64::MAX);
        Some(if d_block <= d_fluid {
            block_hit?
        } else {
            fluid_hit?
        })
    }

    /// `clip(context)` — `traverseBlocks` от `from` к `to`.
    fn clip(&self, context: &ClipContext) -> BlockHitResult {
        self.clip_with_predicate(context, None)
    }

    /// `clip(context, canCollide)`.
    fn clip_with_predicate(
        &self,
        context: &ClipContext,
        can_collide: Option<&dyn Fn(&BlockState) -> bool>,
    ) -> BlockHitResult {
        struct ClipCtx<'a> {
            context: &'a ClipContext,
            can_collide: Option<&'a dyn Fn(&BlockState) -> bool>,
        }
        let from = context.from().clone();
        let to = context.to().clone();
        let ctx = ClipCtx { context, can_collide };
        traverse_blocks(
            &from,
            &to,
            &ctx,
            &|c: &ClipCtx, pos: BlockPos| self.clip_at(c.context, pos, c.can_collide),
            &|_| {
                let dir = Vec3::new(
                    context.from().x - context.to().x,
                    context.from().y - context.to().y,
                    context.from().z - context.to().z,
                );
                BlockHitResult::miss(
                    context.to().clone(),
                    Direction::get_approximate_nearest_vec(&dir),
                    BlockPos::containing(context.to().x, context.to().y, context.to().z),
                )
            },
        )
    }

    /// `clipWithInteractionOverride(startVec, endVec, pos, shape, state)`.
    fn clip_with_interaction_override(
        &self,
        start_vec: &Vec3,
        end_vec: &Vec3,
        pos: BlockPos,
        shape: &VoxelShape,
        state: &BlockState,
    ) -> Option<BlockHitResult> {
        let block_hit = shape.clip(start_vec, end_vec, pos)?;
        let level = self.as_level_reader()?;
        let interaction = state.get_interaction_shape(level, pos);
        if let Some(hit1) = interaction.clip(start_vec, end_vec, pos) {
            if hit1
                .location
                .subtract_vec(start_vec)
                .length_sqr()
                < block_hit.location.subtract_vec(start_vec).length_sqr()
            {
                return Some(block_hit.with_direction(hit1.direction));
            }
        }
        Some(block_hit)
    }

    /// `getBlockFloorHeight(shape, belowShapeSupplier)`.
    fn get_block_floor_height_with(
        &self,
        shape: &VoxelShape,
        below_shape: &VoxelShape,
    ) -> f64 {
        if !shape.is_empty() {
            shape.max(Axis::Y)
        } else {
            let d = below_shape.max(Axis::Y);
            if d >= 1.0 {
                d - 1.0
            } else {
                f64::NEG_INFINITY
            }
        }
    }

    /// `getBlockFloorHeight(pos)`.
    fn get_block_floor_height(&self, pos: BlockPos) -> f64 {
        let level = match self.as_level_reader() {
            Some(l) => l,
            None => return f64::NEG_INFINITY,
        };
        let shape = self
            .get_block_state(pos)
            .map(|s| s.get_collision_shape(level, pos))
            .unwrap_or_else(VoxelShape::empty);
        let below_pos = pos.below();
        let below_shape = self
            .get_block_state(below_pos)
            .map(|s| s.get_collision_shape(level, below_pos))
            .unwrap_or_else(VoxelShape::empty);
        self.get_block_floor_height_with(&shape, &below_shape)
    }
}

/// `traverseBlocks(from, to, context, tester, onFail)` —
/// DDA-обход вокселей (free-функция, как static в Java).
pub fn traverse_blocks<T, C>(
    from: &Vec3,
    to: &Vec3,
    context: &C,
    tester: &dyn Fn(&C, BlockPos) -> Option<T>,
    on_fail: &dyn Fn(&C) -> T,
) -> T {
    if from == to {
        return on_fail(context);
    }
    // Лерп -1e-7 от `to` к `from` — стартовая точка чуть
    // «раньше» цели (как в Java).
    let d = mth::lerp_f64(-1.0e-7, to.x, from.x);
    let d1 = mth::lerp_f64(-1.0e-7, to.y, from.y);
    let d2 = mth::lerp_f64(-1.0e-7, to.z, from.z);
    let d3 = mth::lerp_f64(-1.0e-7, from.x, to.x);
    let d4 = mth::lerp_f64(-1.0e-7, from.y, to.y);
    let d5 = mth::lerp_f64(-1.0e-7, from.z, to.z);
    let mut floor = mth::floor(d3);
    let mut floor1 = mth::floor(d4);
    let mut floor2 = mth::floor(d5);
    if let Some(obj) = tester(context, BlockPos::new(floor, floor1, floor2)) {
        return obj;
    }
    let d6 = d - d3;
    let d7 = d1 - d4;
    let d8 = d2 - d5;
    let i = mth::sign(d6);
    let i1 = mth::sign(d7);
    let i2 = mth::sign(d8);
    let d9 = if i == 0 { f64::MAX } else { i as f64 / d6 };
    let d10 = if i1 == 0 { f64::MAX } else { i1 as f64 / d7 };
    let d11 = if i2 == 0 { f64::MAX } else { i2 as f64 / d8 };
    let mut d12 = d9 * if i > 0 { 1.0 - mth::frac_f64(d3) } else { mth::frac_f64(d3) };
    let mut d13 = d10 * if i1 > 0 { 1.0 - mth::frac_f64(d4) } else { mth::frac_f64(d4) };
    let mut d14 = d11 * if i2 > 0 { 1.0 - mth::frac_f64(d5) } else { mth::frac_f64(d5) };

    while d12 <= 1.0 || d13 <= 1.0 || d14 <= 1.0 {
        if d12 < d13 {
            if d12 < d14 {
                floor += i;
                d12 += d9;
            } else {
                floor2 += i2;
                d14 += d11;
            }
        } else if d13 < d14 {
            floor1 += i1;
            d13 += d10;
        } else {
            floor2 += i2;
            d14 += d11;
        }
        if let Some(obj) = tester(context, BlockPos::new(floor, floor1, floor2)) {
            return obj;
        }
    }
    on_fail(context)
}

/// `BlockStepVisitor`.
pub trait BlockStepVisitor {
    /// `visit(pos, index)` — `false` = прервать обход.
    fn visit(&mut self, pos: BlockPos, index: i32) -> bool;
}

/// `forEachBlockIntersectedBetween(from, to, boundingBox, visitor)`.
pub fn for_each_block_intersected_between(
    from: &Vec3,
    to: &Vec3,
    bounding_box: &Aabb,
    visitor: &mut dyn BlockStepVisitor,
) -> bool {
    let vec3 = to.subtract_vec(from);
    if vec3.length_sqr() < mth::square_f64(1.0e-5) {
        for pos in crate::util::list::SimpleBlockPosIterator::iterable(bounding_box) {
            if !visitor.visit(pos, 0) {
                return false;
            }
        }
        return true;
    }
    let x_zero = vec3.x == 0.0;
    let y_zero = vec3.y == 0.0;
    let z_zero = vec3.z == 0.0;
    if (x_zero && y_zero) || (y_zero && z_zero) || (x_zero && z_zero) {
        // Leaf-оптимизация: движение вдоль одной оси.
        let mut block_index = 0i32;
        for pos in crate::util::list::SimpleBlockPosIterator::traverse_bounds_in_direction(
            &vec3,
            bounding_box,
            16.0,
        ) {
            if !visitor.visit(pos, block_index) {
                return false;
            }
            block_index += 1;
        }
        return true;
    }
    // Общий путь: LongSet + addCollisionsAlongTravel.
    let mut set = std::collections::HashSet::new();
    let moved = bounding_box.move_vec(&vec3.scale(-1.0));
    for pos in BlockPos::between_corners_in_direction_aabb(&moved, &vec3) {
        if !visitor.visit(pos, 0) {
            return false;
        }
        set.insert(pos.as_long());
    }
    let i = match add_collisions_along_travel(&mut set, &vec3, bounding_box, visitor) {
        Some(i) => i,
        None => return false,
    };
    for pos in crate::util::list::SimpleBlockPosIterator::iterable(bounding_box) {
        if set.insert(pos.as_long()) && !visitor.visit(pos, i + 1) {
            return false;
        }
    }
    true
}

/// `addCollisionsAlongTravel` — `None` = visitor прервал (-1 в Java).
fn add_collisions_along_travel(
    output: &mut std::collections::HashSet<i64>,
    travel_vector: &Vec3,
    bounding_box: &Aabb,
    step_visitor: &mut dyn BlockStepVisitor,
) -> Option<i32> {
    let xsize = bounding_box.get_xsize();
    let ysize = bounding_box.get_ysize();
    let zsize = bounding_box.get_zsize();
    let furthest_corner = get_furthest_corner(travel_vector);
    let center = bounding_box.get_center();
    let vec3 = Vec3::new(
        center.x + xsize * 0.5 * furthest_corner.get_x() as f64,
        center.y + ysize * 0.5 * furthest_corner.get_y() as f64,
        center.z + zsize * 0.5 * furthest_corner.get_z() as f64,
    );
    let vec31 = vec3.subtract_vec(travel_vector);
    let mut floor = mth::floor(vec31.x);
    let mut floor1 = mth::floor(vec31.y);
    let mut floor2 = mth::floor(vec31.z);
    let i = mth::sign(travel_vector.x);
    let i1 = mth::sign(travel_vector.y);
    let i2 = mth::sign(travel_vector.z);
    let d = if i == 0 { f64::MAX } else { i as f64 / travel_vector.x };
    let d1 = if i1 == 0 { f64::MAX } else { i1 as f64 / travel_vector.y };
    let d2 = if i2 == 0 { f64::MAX } else { i2 as f64 / travel_vector.z };
    let mut d3 = d * if i > 0 { 1.0 - mth::frac_f64(vec31.x) } else { mth::frac_f64(vec31.x) };
    let mut d4 = d1 * if i1 > 0 { 1.0 - mth::frac_f64(vec31.y) } else { mth::frac_f64(vec31.y) };
    let mut d5 = d2 * if i2 > 0 { 1.0 - mth::frac_f64(vec31.z) } else { mth::frac_f64(vec31.z) };
    let mut i3 = 0i32;

    while d3 <= 1.0 || d4 <= 1.0 || d5 <= 1.0 {
        if d3 < d4 {
            if d3 < d5 {
                floor += i;
                d3 += d;
            } else {
                floor2 += i2;
                d5 += d2;
            }
        } else if d4 < d5 {
            floor1 += i1;
            d4 += d1;
        } else {
            floor2 += i2;
            d5 += d2;
        }

        // `AABB.clip(floor,floor1,floor2, +1,+1,+1, vec31, vec3)` —
        // пересечение отрезка [vec31→vec3] с единичным блоком.
        let cell_min = Vec3::new(floor as f64, floor1 as f64, floor2 as f64);
        let cell_max = Vec3::new(
            (floor + 1) as f64,
            (floor1 + 1) as f64,
            (floor2 + 1) as f64,
        );
        if let Some(optional) = Aabb::clip_coords(
            cell_min.x, cell_min.y, cell_min.z,
            cell_max.x, cell_max.y, cell_max.z,
            &vec31, &vec3,
        ) {
            i3 += 1;
            let vec32 = optional;
            let d6 = mth::clamp_f64(
                vec32.x,
                floor as f64 + 1.0e-5,
                floor as f64 + 1.0 - 1.0e-5,
            );
            let d7 = mth::clamp_f64(
                vec32.y,
                floor1 as f64 + 1.0e-5,
                floor1 as f64 + 1.0 - 1.0e-5,
            );
            let d8 = mth::clamp_f64(
                vec32.z,
                floor2 as f64 + 1.0e-5,
                floor2 as f64 + 1.0 - 1.0e-5,
            );
            let floor3 = mth::floor(d6 - xsize * furthest_corner.get_x() as f64);
            let floor4 = mth::floor(d7 - ysize * furthest_corner.get_y() as f64);
            let floor5 = mth::floor(d8 - zsize * furthest_corner.get_z() as f64);
            let i4 = i3;

            for pos in BlockPos::between_corners_in_direction_coords(
                floor, floor1, floor2, floor3, floor4, floor5, travel_vector,
            ) {
                if output.insert(pos.as_long()) && !step_visitor.visit(pos, i4) {
                    return None;
                }
            }
        }
    }
    Some(i3)
}

/// `getFurthestCorner(vector)`.
fn get_furthest_corner(vector: &Vec3) -> crate::core::vec3i::Vec3i {
    let abs = Vec3::X_AXIS.dot_vec(vector).abs();
    let abs1 = Vec3::Y_AXIS.dot_vec(vector).abs();
    let abs2 = Vec3::Z_AXIS.dot_vec(vector).abs();
    let i = if vector.x >= 0.0 { 1 } else { -1 };
    let i1 = if vector.y >= 0.0 { 1 } else { -1 };
    let i2 = if vector.z >= 0.0 { 1 } else { -1 };
    if abs <= abs1 && abs <= abs2 {
        crate::core::vec3i::Vec3i::new(-i, -i2, i1)
    } else if abs1 <= abs2 {
        crate::core::vec3i::Vec3i::new(i2, -i1, -i)
    } else {
        crate::core::vec3i::Vec3i::new(-i1, i, -i2)
    }
}
