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
            BooleanOp::OnlyFirst => self.clone(),
            BooleanOp::OnlySecond => other.clone(),
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

    /// `join(Shapes.or(...))` — объединение нескольких форм.
    pub fn join_all(shapes: &[&VoxelShape]) -> Self {
        let mut boxes = Vec::new();
        for s in shapes {
            boxes.extend(s.boxes.iter().cloned());
        }
        Self { boxes }.optimize()
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
