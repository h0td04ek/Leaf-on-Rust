//! `Level.clip(ClipContext)` — рейкаст по блокам и жидкостям.
//! Свободные функции (не метод `LevelReader` — dyn-compat).

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::hit_result::BlockHitResult;
use crate::world::phys::Vec3;

/// `ClipContext.Block` — режим клипа по блокам.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipBlock {
    /// `OUTLINE` — `getShape(OUTLINE)`.
    Outline,
    /// `COLLIDER` — `getCollisionShape`.
    Collider,
    /// `VISUAL` — визуальная форма (= outline у нас).
    Visual,
    /// `FALLING` — форма для falling-block (= collider).
    Falling,
    /// `NO_COLLISION` — без проверки блоков.
    NoCollision,
}

/// `ClipContext.Fluid` — режим клипа по жидкостям.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipFluid {
    /// `NONE`.
    None,
    /// `SOURCE_ONLY` — только источники жидкости.
    SourceOnly,
    /// `ANY` — любая жидкость.
    Any,
}

/// `Level.clip(from, to, clipBlock, clipFluid)` — DDA-обход
/// вокселей + clip collision/outline-шейпов. MISS →
/// `BlockHitResult::miss(to)`.
pub fn clip(
    level: &dyn LevelReader,
    from: Vec3,
    to: Vec3,
    clip_block: ClipBlock,
    clip_fluid: ClipFluid,
) -> BlockHitResult {
    let dir = Vec3::new(to.x - from.x, to.y - from.y, to.z - from.z);
    let start = BlockPos::containing(from.x, from.y, from.z);
    let end = BlockPos::containing(to.x, to.y, to.z);
    if start == end {
        return clip_in_cell(level, &from, &to, start, clip_block, clip_fluid);
    }
    // Amanatides–Woo DDA.
    let step = |a: i32, b: i32| -> i32 {
        if b > a { 1 } else if b < a { -1 } else { 0 }
    };
    let step_x = step(start.x, end.x);
    let step_y = step(start.y, end.y);
    let step_z = step(start.z, end.z);
    let t_delta = |d: f64, s: i32| -> f64 {
        if s == 0 { f64::INFINITY } else { 1.0 / d.abs() }
    };
    let t_max = |pos: i32, d: f64, s: i32, from_c: f64| -> f64 {
        if s == 0 {
            f64::INFINITY
        } else {
            let boundary = if s > 0 { (pos + 1) as f64 } else { pos as f64 };
            (boundary - from_c) / d
        }
    };
    let mut tx = t_max(start.x, dir.x, step_x, from.x);
    let mut ty = t_max(start.y, dir.y, step_y, from.y);
    let mut tz = t_max(start.z, dir.z, step_z, from.z);
    let d_tx = t_delta(dir.x, step_x);
    let d_ty = t_delta(dir.y, step_y);
    let d_tz = t_delta(dir.z, step_z);
    let mut cell = start;
    let mut first_fluid: Option<(BlockPos, f64)> = None;
    let mut guard = 0i32;
    loop {
        guard += 1;
        if guard > 4096 {
            break;
        }
        let (hit, fluid_hit) = clip_cell_hit(level, &from, &to, cell, clip_block, clip_fluid);
        if let Some((loc, face, inside)) = hit {
            return BlockHitResult::new(loc, face, cell, inside);
        }
        if fluid_hit && first_fluid.is_none() {
            let t_enter = tx.min(ty).min(tz).clamp(0.0, 1.0);
            first_fluid = Some((cell, t_enter));
        }
        if cell == end {
            break;
        }
        if tx <= ty && tx <= tz {
            cell = BlockPos::new(cell.x + step_x, cell.y, cell.z);
            tx += d_tx;
        } else if ty <= tz {
            cell = BlockPos::new(cell.x, cell.y + step_y, cell.z);
            ty += d_ty;
        } else {
            cell = BlockPos::new(cell.x, cell.y, cell.z + step_z);
            tz += d_tz;
        }
        if tx.min(ty).min(tz) > 1.0 {
            break;
        }
    }
    if let Some((fluid_pos, t)) = first_fluid {
        if clip_fluid != ClipFluid::None {
            let loc = Vec3::new(
                from.x + (to.x - from.x) * t,
                from.y + (to.y - from.y) * t,
                from.z + (to.z - from.z) * t,
            );
            let face = Direction::get_approximate_nearest_vec(&dir);
            return BlockHitResult::new(loc, face, fluid_pos, false);
        }
    }
    BlockHitResult::miss(to, Direction::get_approximate_nearest_vec(&dir), end)
}

/// `clip` для одной ячейки (start == end).
fn clip_in_cell(
    level: &dyn LevelReader,
    from: &Vec3,
    to: &Vec3,
    cell: BlockPos,
    clip_block: ClipBlock,
    clip_fluid: ClipFluid,
) -> BlockHitResult {
    let (hit, fluid_hit) = clip_cell_hit(level, from, to, cell, clip_block, clip_fluid);
    if let Some((loc, face, inside)) = hit {
        return BlockHitResult::new(loc, face, cell, inside);
    }
    let dir = Vec3::new(to.x - from.x, to.y - from.y, to.z - from.z);
    if fluid_hit && clip_fluid != ClipFluid::None {
        return BlockHitResult::new(to.clone(), Direction::get_approximate_nearest_vec(&dir), cell, false);
    }
    BlockHitResult::miss(to.clone(), Direction::get_approximate_nearest_vec(&dir), cell)
}

/// Проверка одной воксельной ячейки при clip.
/// Возвращает `(block_hit, has_fluid)`:
/// `block_hit` — `(location, face, inside)` при попадании.
fn clip_cell_hit(
    level: &dyn LevelReader,
    from: &Vec3,
    to: &Vec3,
    cell: BlockPos,
    clip_block: ClipBlock,
    clip_fluid: ClipFluid,
) -> (
    Option<(Vec3, Direction, bool)>,
    bool,
) {
    let offset = Vec3::new(cell.x as f64, cell.y as f64, cell.z as f64);
    let mut fluid_hit = false;
    if let Some(state) = level.get_block_state(cell) {
        if clip_fluid != ClipFluid::None {
            let fluid = state.get_fluid_state();
            if !fluid.is_empty()
                && (clip_fluid == ClipFluid::Any || fluid.fluid.is_source)
            {
                fluid_hit = true;
            }
        }
        if clip_block != ClipBlock::NoCollision {
            let shape = match clip_block {
                ClipBlock::Collider | ClipBlock::Falling => {
                    state.get_collision_shape(level, cell)
                }
                _ => state.get_shape(level, cell),
            };
            if !shape.is_empty() {
                let local = Vec3::new(
                    from.x - offset.x,
                    from.y - offset.y,
                    from.z - offset.z,
                );
                let inside = shape.contains(local.x, local.y, local.z);
                let mut best: Option<(Vec3, f64)> = None;
                for aabb in shape.to_aabbs() {
                    let moved = aabb.move_vec(&offset);
                    if let Some(loc) = moved.clip(from, to) {
                        let d = loc.distance_to_sqr(from);
                        if best.as_ref().map(|(_, bd)| d < *bd).unwrap_or(true) {
                            best = Some((loc, d));
                        }
                    }
                }
                if let Some((loc, _)) = best {
                    let face = Direction::get_approximate_nearest_vec(&Vec3::new(
                        loc.x - offset.x - 0.5,
                        loc.y - offset.y - 0.5,
                        loc.z - offset.z - 0.5,
                    ));
                    return (Some((loc, face, inside)), fluid_hit);
                }
            }
        }
    }
    (None, fluid_hit)
}
