//! Перенос `net.minecraft.world.phys.shapes.VoxelShape`.
//!
//! Java-реализация хранит форму как дерево бинарных срезов
//! (`DiscreteVoxelShape`/`IndexMerger`). Здесь форма представлена
//! списком непересекающихся `Aabb`-объёмов, вычисленных
//! алгебраическими операциями — семантика `bounds`, `isEmpty`,
//! `max(axis)`, `join`/`BooleanOp` совпадает с ванильной.
//!
//! Покрыты операции, нужные ядру: `of`, `empty`, `block`,
//! `cube`, `join` (Or/OnlyFirst/OnlySecond/Not/And/XNor),
//! `optimize`, `bounds`, `isEmpty`, `max`/`min(axis)`,
//! `toAabbs`, `move`, `scale`.

use crate::core::direction::Axis;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::vec3::Vec3;

/// `BooleanOp` — операции над объёмами (`Shapes.join`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BooleanOp {
    /// `Shapes.or` — объединение.
    Or,
    /// `Shapes.and` — пересечение.
    And,
    /// `Shapes.not` — разность `a \ b`.
    Not,
    /// `Shapes.onlyFirst`.
    OnlyFirst,
    /// `Shapes.onlySecond`.
    OnlySecond,
    /// `Shapes.xnor` — `(a ∪ b) \ (a ∩ b)`.
    XNor,
}

/// `VoxelShape` — объём формы блока.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxelShape {
    /// Непересекающиеся AABB-части (у пустой формы — пусто).
    boxes: Vec<Aabb>,
}

impl VoxelShape {
    /// `VoxelShape.empty()`.
    pub fn empty() -> Self {
        Self { boxes: Vec::new() }
    }

    /// `VoxelShapes.block()` — полный куб блока (0..1).
    pub fn block() -> Self {
        Self::of(Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0))
    }

    /// `VoxelShape.of(aabb)`.
    pub fn of(aabb: Aabb) -> Self {
        let mut shape = Self { boxes: Vec::new() };
        shape.boxes.push(aabb);
        shape
    }

    /// `VoxelShape.cube(size)` — куб со стороной `size`.
    pub fn cube(size: f64) -> Self {
        Self::of(Aabb::new(0.0, 0.0, 0.0, size, size, size))
    }

    /// `isEmpty` — форма пуста.
    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    /// `bounds` — ограничивающий объём (null у пустой — в Java
    /// возвращает `Optional.empty()`; здесь `None`).
    pub fn bounds(&self) -> Option<Aabb> {
        let mut it = self.boxes.iter();
        let first = it.next()?;
        let mut b = first.clone();
        for box_ in it {
            b = b.minmax(box_);
        }
        Some(b)
    }

    /// `max(axis)` — максимальная координата по оси.
    /// Для пустой формы возвращает `f64::NEG_INFINITY`
    /// (в Java `Double.NEGATIVE_INFINITY`).
    pub fn max(&self, axis: Axis) -> f64 {
        self.boxes
            .iter()
            .map(|b| b.max_axis(axis))
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// `min(axis)` — минимальная координата по оси.
    pub fn min(&self, axis: Axis) -> f64 {
        self.boxes
            .iter()
            .map(|b| b.min_axis(axis))
            .fold(f64::INFINITY, f64::min)
    }

    /// `toAabbs` — список объёмов (для итерации по AABB).
    pub fn to_aabbs(&self) -> &[Aabb] {
        &self.boxes
    }

    /// `deflate(d)` — сжатие всех AABB на d со всех сторон.
    pub fn deflate(&self, d: f64) -> Self {
        Self {
            boxes: self.boxes.iter().map(|b| b.deflate_all(d)).collect(),
        }
        .optimize()
    }

    /// `optimize` — удалить вырожденные/дублирующиеся части.
    /// (Vanilla объединяет соседние срезы в индексе.)
    pub fn optimize(&self) -> Self {
        let mut result = self.clone();
        // Удаляем нулевые по объёму части и дубликаты.
        result.boxes.retain(|b| {
            let _ = b;
            b.max_x > b.min_x && b.max_y > b.min_y && b.max_z > b.min_z
        });
        result.boxes.sort_by(|a, b| {
            a.min_x
                .partial_cmp(&b.min_x)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    a.min_y
                        .partial_cmp(&b.min_y)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then(
                    a.min_z
                        .partial_cmp(&b.min_z)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
        });
        result.boxes.dedup_by(|a, b| {
            (a.min_x - b.min_x).abs() < 1e-9
                && (a.min_y - b.min_y).abs() < 1e-9
                && (a.min_z - b.min_z).abs() < 1e-9
                && (a.max_x - b.max_x).abs() < 1e-9
                && (a.max_y - b.max_y).abs() < 1e-9
                && (a.max_z - b.max_z).abs() < 1e-9
        });
        result
    }

    /// `move(x, y, z)` — сдвинуть форму.
    pub fn move_shape(&self, x: f64, y: f64, z: f64) -> Self {
        Self {
            boxes: self
                .boxes
                .iter()
                .map(|b| b.move_xyz(x, y, z))
                .collect(),
        }
    }

    /// `move(Vec3)` — сдвинуть на вектор.
    pub fn move_vec(&self, vec: &Vec3) -> Self {
        self.move_shape(vec.x, vec.y, vec.z)
    }

    /// `join(other, op)` — булева операция над формами.
    pub fn join(&self, other: &VoxelShape, op: BooleanOp) -> Self {
        match op {
            BooleanOp::Or => Self {
                boxes: [self.boxes.clone(), other.boxes.clone()].concat(),
            },
            // ONLY_FIRST = `first && !second` — разность `a \ b`
            // (совпадает с `Not` в нашей AABB-модели).
            BooleanOp::OnlyFirst => self.join(other, BooleanOp::Not),
            // ONLY_SECOND = `!first && second` — `b \ a`.
            BooleanOp::OnlySecond => other.join(self, BooleanOp::Not),
            BooleanOp::Not => {
                // self \ other — вычитание каждого бокса other
                // из каждого бокса self.
                let mut result = Vec::new();
                for a in &self.boxes {
                    let mut pieces = vec![a.clone()];
                    for b in &other.boxes {
                        pieces = pieces
                            .iter()
                            .flat_map(|p| subtract_aabb(p, b))
                            .collect();
                    }
                    result.extend(pieces);
                }
                Self { boxes: result }.optimize()
            }
            BooleanOp::And => {
                let mut result = Vec::new();
                for a in &self.boxes {
                    for b in &other.boxes {
                        if a.intersects(b) {
                            let i = a.intersect(b);
                            if i.max_x > i.min_x
                                && i.max_y > i.min_y
                                && i.max_z > i.min_z
                            {
                                result.push(i);
                            }
                        }
                    }
                }
                Self { boxes: result }.optimize()
            }
            BooleanOp::XNor => {
                // (a ∪ b) \ (a ∩ b) — симметрическая разность.
                let union = self.join(other, BooleanOp::Or);
                let inter = self.join(other, BooleanOp::And);
                union.join(&inter, BooleanOp::Not)
            }
        }
    }

    /// `closestPointTo(pos)` — ближайшая к `pos` точка формы.
    /// `None` для пустой формы.
    pub fn closest_point_to(&self, pos: &crate::world::phys::vec3::Vec3) -> Option<crate::world::phys::vec3::Vec3> {
        let mut best: Option<(f64, crate::world::phys::vec3::Vec3)> = None;
        for b in &self.boxes {
            let x = pos.x.clamp(b.min_x, b.max_x);
            let y = pos.y.clamp(b.min_y, b.max_y);
            let z = pos.z.clamp(b.min_z, b.max_z);
            let dx = pos.x - x;
            let dy = pos.y - y;
            let dz = pos.z - z;
            let d = dx * dx + dy * dy + dz * dz;
            if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                best = Some((d, crate::world::phys::vec3::Vec3::new(x, y, z)));
            }
        }
        best.map(|(_, v)| v)
    }

    /// `join(Shapes.or(...))` — объединение нескольких форм.
    pub fn join_all(shapes: &[&VoxelShape]) -> Self {
        let mut boxes = Vec::new();
        for s in shapes {
            boxes.extend(s.boxes.iter().cloned());
        }
        Self { boxes }.optimize()
    }

    /// `clip(from, to, pos)` — ближайшее пересечение луча
    /// `[from,to]` с формой в мировых координатах
    /// (`pos` — позиция блока). `None` — нет пересечения.
    pub fn clip(
        &self,
        from: &crate::world::phys::vec3::Vec3,
        to: &crate::world::phys::vec3::Vec3,
        pos: crate::core::block_pos::BlockPos,
    ) -> Option<crate::world::phys::hit_result::BlockHitResult> {
        if self.is_empty() {
            return None;
        }
        let offset = crate::world::phys::vec3::Vec3::new(
            pos.get_x() as f64,
            pos.get_y() as f64,
            pos.get_z() as f64,
        );
        let local = crate::world::phys::vec3::Vec3::new(
            from.x - offset.x,
            from.y - offset.y,
            from.z - offset.z,
        );
        let mut best: Option<(crate::world::phys::vec3::Vec3, f64)> = None;
        for aabb in &self.boxes {
            if let Some(loc) = aabb.clip(from, to) {
                let d = loc.distance_to_sqr(from);
                if best.as_ref().map(|(_, bd)| d < *bd).unwrap_or(true) {
                    best = Some((loc, d));
                }
            }
        }
        let (loc, _) = best?;
        let inside = self.contains(local.x, local.y, local.z);
        let face = crate::core::direction::Direction::get_approximate_nearest_vec(
            &crate::world::phys::vec3::Vec3::new(
                loc.x - offset.x - 0.5,
                loc.y - offset.y - 0.5,
                loc.z - offset.z - 0.5,
            ),
        );
        Some(crate::world::phys::hit_result::BlockHitResult::new(
            loc, face, pos, inside,
        ))
    }

    /// Проверка: содержит ли форма точку (block-local координаты).
    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        self.boxes.iter().any(|b| b.contains(x, y, z))
    }

    /// `get(Vec3i)`-эквивалент — точка внутри блока.
    pub fn contains_pos(&self, pos: &crate::core::block_pos::BlockPos) -> bool {
        self.contains(pos.x as f64, pos.y as f64, pos.z as f64)
    }
}

/// `COLLISION_EPSILON` (moonrise `CollisionUtil`).
pub const COLLISION_EPSILON: f64 = 1.0e-7;

/// `mergedMayOccludeBlock(shape1, shape2)` — объединённые границы
/// форм должны покрывать единичный куб с epsilon.
pub fn merged_may_occlude_block(shape1: &VoxelShape, shape2: &VoxelShape) -> bool {
    let (Some(b1), Some(b2)) = (shape1.bounds(), shape2.bounds()) else {
        return false;
    };
    let min_x = b1.min_x.min(b2.min_x);
    let min_y = b1.min_y.min(b2.min_y);
    let min_z = b1.min_z.min(b2.min_z);
    let max_x = b1.max_x.max(b2.max_x);
    let max_y = b1.max_y.max(b2.max_y);
    let max_z = b1.max_z.max(b2.max_z);
    min_x <= COLLISION_EPSILON
        && max_x >= 1.0 - COLLISION_EPSILON
        && min_y <= COLLISION_EPSILON
        && max_y >= 1.0 - COLLISION_EPSILON
        && min_z <= COLLISION_EPSILON
        && max_z >= 1.0 - COLLISION_EPSILON
}

/// `moonrise$occludesFullBlock` — форма покрывает единичный куб.
/// Точная проверка: разбиение по всем координатам граней брусов и
/// проверка покрытия каждой ячейки (union AABB покрывает куб тогда
/// и только тогда, когда покрыта каждая ячейка сетки).
fn occludes_full_block(shape: &VoxelShape) -> bool {
    if shape.is_empty() {
        return false;
    }
    let Some(b) = shape.bounds() else {
        return false;
    };
    if b.min_x > COLLISION_EPSILON
        || b.min_y > COLLISION_EPSILON
        || b.min_z > COLLISION_EPSILON
        || b.max_x < 1.0 - COLLISION_EPSILON
        || b.max_y < 1.0 - COLLISION_EPSILON
        || b.max_z < 1.0 - COLLISION_EPSILON
    {
        return false;
    }
    covers_unit_cube(&shape.boxes)
}

/// Покрывают ли AABB-ы весь единичный куб [0,1]³.
fn covers_unit_cube(boxes: &[Aabb]) -> bool {
    let mut xs: Vec<f64> = vec![0.0, 1.0];
    let mut ys: Vec<f64> = vec![0.0, 1.0];
    let mut zs: Vec<f64> = vec![0.0, 1.0];
    for b in boxes {
        for v in [b.min_x, b.max_x] {
            if v > 0.0 && v < 1.0 && !xs.contains(&v) {
                xs.push(v);
            }
        }
        for v in [b.min_y, b.max_y] {
            if v > 0.0 && v < 1.0 && !ys.contains(&v) {
                ys.push(v);
            }
        }
        for v in [b.min_z, b.max_z] {
            if v > 0.0 && v < 1.0 && !zs.contains(&v) {
                zs.push(v);
            }
        }
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    zs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for i in 0..xs.len() - 1 {
        for j in 0..ys.len() - 1 {
            for k in 0..zs.len() - 1 {
                let (x, y, z) = (
                    (xs[i] + xs[i + 1]) * 0.5,
                    (ys[j] + ys[j + 1]) * 0.5,
                    (zs[k] + zs[k + 1]) * 0.5,
                );
                if !boxes.iter().any(|b| b.contains(x, y, z)) {
                    return false;
                }
            }
        }
    }
    true
}

/// `Shapes.faceShapeOccludes(shape1, shape2)` — окклюзия между
/// соседними гранями блоков (используется движком освещения).
pub fn face_shape_occludes(shape1: &VoxelShape, shape2: &VoxelShape) -> bool {
    if occludes_full_block(shape1) || occludes_full_block(shape2) {
        return true;
    }

    let s1_empty = shape1.is_empty();
    let s2_empty = shape2.is_empty();
    if s1_empty && s2_empty {
        return false;
    }
    if s1_empty || s2_empty {
        return if s2_empty {
            occludes_full_block(shape1)
        } else {
            occludes_full_block(shape2)
        };
    }
    if shape1 == shape2 {
        return occludes_full_block(shape1);
    }

    merged_may_occlude_block(shape1, shape2)
        && occludes_full_block(&shape1.join(shape2, BooleanOp::Or))
}

/// `Shapes.mergedFaceOccludes(first, second, direction)` — окклюзия
/// между гранями блоков в направлении `direction` (используется
/// `LightEngine.getLightBlockInto`).
pub fn merged_face_occludes(
    first: &VoxelShape,
    second: &VoxelShape,
    direction: crate::core::direction::Direction,
) -> bool {
    if occludes_full_block(first) || occludes_full_block(second) {
        return true;
    }
    if first.is_empty() && second.is_empty() {
        return false;
    }

    let first_rects = face_rects(first, direction);
    let second_rects = face_rects(second, direction.get_opposite());

    let first_empty = first_rects.is_empty();
    let second_empty = second_rects.is_empty();
    if first_empty && second_empty {
        return false;
    }
    if first_empty || second_empty {
        return face_rects_cover(if second_empty { &first_rects } else { &second_rects });
    }

    let mut all = first_rects;
    all.extend(second_rects);
    face_rects_cover(&all)
}

/// 2D-прямоугольник на единичной грани: `(u_min, v_min, u_max, v_max)`.
type FaceRect = (f64, f64, f64, f64);

/// `moonrise$getFaceShapeClamped(direction)` в виде набора
/// прямоугольников: проекция брусов, касающихся грани в направлении
/// `direction`, на единичный квадрат грани.
fn face_rects(shape: &VoxelShape, direction: crate::core::direction::Direction) -> Vec<FaceRect> {
    use crate::core::direction::{Axis, AxisDirection};
    let axis = direction.get_axis();
    let positive = direction.get_axis_direction() == AxisDirection::POSITIVE;
    let (u_axis, v_axis) = match axis {
        Axis::X => (Axis::Z, Axis::Y),
        Axis::Y => (Axis::X, Axis::Z),
        Axis::Z => (Axis::X, Axis::Y),
    };
    let mut rects = Vec::new();
    for b in &shape.boxes {
        let (min_a, max_a) = axis_range(b, axis);
        let touches = if positive {
            max_a >= 1.0 - COLLISION_EPSILON
        } else {
            min_a <= COLLISION_EPSILON
        };
        if !touches {
            continue;
        }
        let (u_min, u_max) = axis_range_clamped(b, u_axis);
        let (v_min, v_max) = axis_range_clamped(b, v_axis);
        rects.push((u_min, v_min, u_max, v_max));
    }
    rects
}

fn axis_range(b: &Aabb, axis: crate::core::direction::Axis) -> (f64, f64) {
    use crate::core::direction::Axis;
    match axis {
        Axis::X => (b.min_x, b.max_x),
        Axis::Y => (b.min_y, b.max_y),
        Axis::Z => (b.min_z, b.max_z),
    }
}

fn axis_range_clamped(b: &Aabb, axis: crate::core::direction::Axis) -> (f64, f64) {
    use crate::core::direction::Axis;
    match axis {
        Axis::X => (b.min_x.clamp(0.0, 1.0), b.max_x.clamp(0.0, 1.0)),
        Axis::Y => (b.min_y.clamp(0.0, 1.0), b.max_y.clamp(0.0, 1.0)),
        Axis::Z => (b.min_z.clamp(0.0, 1.0), b.max_z.clamp(0.0, 1.0)),
    }
}

/// Покрывают ли прямоугольники грани весь единичный квадрат [0,1]².
fn face_rects_cover(rects: &[FaceRect]) -> bool {
    if rects.is_empty() {
        return false;
    }
    let mut xs: Vec<f64> = vec![0.0, 1.0];
    let mut ys: Vec<f64> = vec![0.0, 1.0];
    for r in rects {
        for v in [r.0, r.2] {
            if v > 0.0 && v < 1.0 && !xs.contains(&v) {
                xs.push(v);
            }
        }
        for v in [r.1, r.3] {
            if v > 0.0 && v < 1.0 && !ys.contains(&v) {
                ys.push(v);
            }
        }
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for i in 0..xs.len() - 1 {
        for j in 0..ys.len() - 1 {
            let (u, v) = ((xs[i] + xs[i + 1]) * 0.5, (ys[j] + ys[j + 1]) * 0.5);
            if !rects.iter().any(|r| u >= r.0 && u <= r.2 && v >= r.1 && v <= r.3) {
                return false;
            }
        }
    }
    true
}

/// `a \ b` — вычитание AABB `b` из AABB `a`, результат —
/// список непересекающихся боксов (стандартная декомпозиция
/// на до 6 частей по осям).
fn subtract_aabb(a: &Aabb, b: &Aabb) -> Vec<Aabb> {
    if !a.intersects(b) {
        return vec![a.clone()];
    }
    let mut result = Vec::new();
    // Пересечение
    let ix1 = a.min_x.max(b.min_x);
    let iy1 = a.min_y.max(b.min_y);
    let iz1 = a.min_z.max(b.min_z);
    let ix2 = a.max_x.min(b.max_x);
    let iy2 = a.max_y.min(b.max_y);
    let iz2 = a.max_z.min(b.max_z);

    // Часть ниже по X
    if ix1 > a.min_x {
        result.push(Aabb::new(a.min_x, a.min_y, a.min_z, ix1, a.max_y, a.max_z));
    }
    // Часть выше по X
    if ix2 < a.max_x {
        result.push(Aabb::new(ix2, a.min_y, a.min_z, a.max_x, a.max_y, a.max_z));
    }
    // Часть ниже по Y (в пределах X-пересечения)
    if iy1 > a.min_y {
        result.push(Aabb::new(ix1, a.min_y, iz1, ix2, iy1, iz2));
    }
    // Часть выше по Y
    if iy2 < a.max_y {
        result.push(Aabb::new(ix1, iy2, iz1, ix2, a.max_y, iz2));
    }
    // Часть ниже по Z
    if iz1 > a.min_z {
        result.push(Aabb::new(ix1, iy1, a.min_z, ix2, iy2, iz1));
    }
    // Часть выше по Z
    if iz2 < a.max_z {
        result.push(Aabb::new(ix1, iy1, iz2, ix2, iy2, a.max_z));
    }
    result
        .into_iter()
        .filter(|r| r.max_x > r.min_x && r.max_y > r.min_y && r.max_z > r.min_z)
        .collect()
}

/// `moonrise$getSingleAABBRepresentation` — Some(AABB), если форма
/// состоит ровно из одного бокса (moonrise `CollisionVoxelShape`).
pub fn single_aabb_representation(shape: &VoxelShape) -> Option<Aabb> {
    if shape.boxes.len() == 1 {
        Some(shape.boxes[0])
    } else {
        None
    }
}

/// `CollisionUtil.isEmpty(AABB)` — вырожденный бокс (< COLLISION_EPSILON
/// по какому-либо измерению).
pub fn collision_aabb_is_empty(aabb: &Aabb) -> bool {
    (aabb.max_x - aabb.min_x) < COLLISION_EPSILON
        || (aabb.max_y - aabb.min_y) < COLLISION_EPSILON
        || (aabb.max_z - aabb.min_z) < COLLISION_EPSILON
}

/// `CollisionUtil.voxelShapeIntersectNoEmpty(voxel, aabb)` —
/// пересечение формы с боксом (строгое, непустое).
pub fn voxel_shape_intersect_no_empty(shape: &VoxelShape, aabb: &Aabb) -> bool {
    !shape.is_empty()
        && shape
            .boxes
            .iter()
            .any(|b| b.intersects(aabb))
}

/// `Shapes.joinIsNotEmpty(shape1, shape2, op)` — результат
/// булевой операции непуст.
pub fn join_is_not_empty(shape1: &VoxelShape, shape2: &VoxelShape, op: BooleanOp) -> bool {
    !shape1.join(shape2, op).is_empty()
}
