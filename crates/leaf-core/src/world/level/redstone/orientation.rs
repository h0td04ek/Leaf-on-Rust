//! `net.minecraft.world.level.redstone.Orientation` — ориентация
//! редстоуна (48 комбинаций up/front/sideBias).
//!
//! Связи `withFront`/`withUp`/`withSideBias` хранятся как индексы
//! в глобальном массиве `ORIENTATIONS` (self-referential в Java —
//! здесь через `&'static` после инициализации).

use std::sync::OnceLock;

use crate::core::direction::{Axis, Direction};
use crate::random::RandomSource;

/// `Orientation.SideBias`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SideBias {
    /// `LEFT`.
    Left,
    /// `RIGHT`.
    Right,
}

impl SideBias {
    /// `values()`.
    pub const VALUES: [SideBias; 2] = [SideBias::Left, SideBias::Right];

    /// `getOpposite()`.
    pub fn opposite(&self) -> Self {
        match self {
            SideBias::Left => SideBias::Right,
            SideBias::Right => SideBias::Left,
        }
    }
}

/// `Orientation`.
#[derive(Clone, Debug)]
pub struct Orientation {
    up: Direction,
    front: Direction,
    side: Direction,
    side_bias: SideBias,
    index: usize,
    neighbors: Vec<Direction>,
    horizontal_neighbors: Vec<Direction>,
    vertical_neighbors: Vec<Direction>,
    /// `withFront[d.ordinal()]` → индекс в `ORIENTATIONS`.
    with_front: [u8; 6],
    /// `withUp[d.ordinal()]` → индекс.
    with_up: [u8; 6],
    /// `withSideBias[sb.ordinal()]` → индекс.
    with_side_bias: [u8; 2],
}

fn dir_ord(d: Direction) -> usize {
    d as usize
}

/// Java `Direction.AxisDirection.ordinal()` — порядок
/// DOWN,EAST,NORTH,SOUTH,UP,WEST.
fn axis_direction_ordinal(d: Direction) -> usize {
    match d {
        Direction::DOWN => 0,
        Direction::EAST => 1,
        Direction::NORTH => 2,
        Direction::SOUTH => 3,
        Direction::UP => 4,
        Direction::WEST => 5,
    }
}

/// `generateIndex(up, front, sideBias)`.
fn generate_index(up: Direction, front: Direction, side_bias: SideBias) -> usize {
    if up.get_axis() == front.get_axis() {
        panic!("Up-vector and front-vector can not be on the same axis");
    }
    let i = if up.get_axis() == Axis::Y {
        if front.get_axis() == Axis::X { 1 } else { 0 }
    } else if front.get_axis() == Axis::Y {
        1
    } else {
        0
    };
    let i1 = (i << 1) | axis_direction_ordinal(front);
    ((dir_ord(up) << 2) + i1 << 1) + side_bias as usize
}

impl Orientation {
    fn new(up: Direction, front: Direction, side_bias: SideBias) -> Self {
        let index = generate_index(up, front, side_bias);
        let vec = front.get_unit_vec3i().cross(&up.get_unit_vec3i());
        let nearest = Direction::get_nearest_vec3i(&vec, None)
            .expect("nearest direction");
        let side = if side_bias == SideBias::Right {
            nearest
        } else {
            nearest.get_opposite()
        };

        let neighbors = vec![
            front.get_opposite(),
            front,
            side,
            side.get_opposite(),
            up.get_opposite(),
            up,
        ];
        let up_axis = up.get_axis();
        let horizontal_neighbors: Vec<Direction> = neighbors
            .iter()
            .copied()
            .filter(|d| d.get_axis() != up_axis)
            .collect();
        let vertical_neighbors: Vec<Direction> = neighbors
            .iter()
            .copied()
            .filter(|d| d.get_axis() == up_axis)
            .collect();

        Self {
            up,
            front,
            side,
            side_bias,
            index,
            neighbors,
            horizontal_neighbors,
            vertical_neighbors,
            with_front: [0; 6],
            with_up: [0; 6],
            with_side_bias: [0; 2],
        }
    }

    fn resolve(&self, idx: u8) -> &'static Orientation {
        Orientation::from_index(idx as usize)
    }

    /// `of(up, front, sideBias)`.
    pub fn of(up: Direction, front: Direction, side_bias: SideBias) -> &'static Orientation {
        Orientation::from_index(generate_index(up, front, side_bias))
    }

    /// `withUp(up)`.
    pub fn with_up(&self, up: Direction) -> &'static Orientation {
        self.resolve(self.with_up[dir_ord(up)])
    }

    /// `withFront(front)`.
    pub fn with_front(&self, front: Direction) -> &'static Orientation {
        self.resolve(self.with_front[dir_ord(front)])
    }

    /// `withFrontPreserveUp(front)`.
    pub fn with_front_preserve_up(&self, front: Direction) -> &'static Orientation {
        if front.get_axis() == self.up.get_axis() {
            // возвращаем себя — нужен 'static self; self уже 'static
            // у вызывающих (Orientation всегда из ORIENTATIONS).
            unsafe { &*(self as *const Orientation) }
        } else {
            self.with_front(front)
        }
    }

    /// `withFrontAdjustSideBias(front)`.
    pub fn with_front_adjust_side_bias(&self, front: Direction) -> &'static Orientation {
        let orientation = self.with_front(front);
        if self.front == orientation.side {
            orientation.with_mirror()
        } else {
            orientation
        }
    }

    /// `withSideBias(sideBias)`.
    pub fn with_side_bias(&self, side_bias: SideBias) -> &'static Orientation {
        self.resolve(self.with_side_bias[side_bias as usize])
    }

    /// `withMirror()`.
    pub fn with_mirror(&self) -> &'static Orientation {
        self.with_side_bias(self.side_bias.opposite())
    }

    /// `getFront()`.
    pub fn front(&self) -> Direction {
        self.front
    }

    /// `getUp()`.
    pub fn up(&self) -> Direction {
        self.up
    }

    /// `getSide()`.
    pub fn side(&self) -> Direction {
        self.side
    }

    /// `getSideBias()`.
    pub fn side_bias(&self) -> SideBias {
        self.side_bias
    }

    /// `getDirections()`.
    pub fn directions(&self) -> &[Direction] {
        &self.neighbors
    }

    /// `getHorizontalDirections()`.
    pub fn horizontal_directions(&self) -> &[Direction] {
        &self.horizontal_neighbors
    }

    /// `getVerticalDirections()`.
    pub fn vertical_directions(&self) -> &[Direction] {
        &self.vertical_neighbors
    }

    /// `getIndex()`.
    pub fn index(&self) -> usize {
        self.index
    }

    /// `fromIndex(index)`.
    pub fn from_index(index: usize) -> &'static Orientation {
        &orientations()[index]
    }

    /// `random(random)`.
    pub fn random(random: &mut dyn RandomSource) -> &'static Orientation {
        let o = orientations();
        &o[random.next_int_bound(o.len() as i32) as usize]
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[up={},front={},sideBias={:?}]",
            self.up.get_name(),
            self.front.get_name(),
            self.side_bias
        )
    }
}

/// Все 48 ориентаций (`ORIENTATIONS`) + связи.
fn orientations() -> &'static [Orientation; 48] {
    static ORIENTATIONS: OnceLock<[Orientation; 48]> = OnceLock::new();
    ORIENTATIONS.get_or_init(|| {
        // Собираем все комбинации (up != axis(front)) — ровно 48.
        let mut all: Vec<Orientation> = Vec::with_capacity(48);
        for up in Direction::VALUES {
            for front in Direction::VALUES {
                if up.get_axis() == front.get_axis() {
                    continue;
                }
                for sb in SideBias::VALUES {
                    all.push(Orientation::new(up, front, sb));
                }
            }
        }
        debug_assert_eq!(all.len(), 48);
        // Раскладываем по index.
        let mut arr: Vec<Option<Orientation>> = (0..48).map(|_| None).collect();
        for o in all {
            let idx = o.index;
            arr[idx] = Some(o);
        }
        let mut out: Vec<Orientation> = Vec::with_capacity(48);
        for o in arr {
            out.push(o.expect("orientation missing"));
        }
        // Заполняем связи.
        for i in 0..48 {
            let up = out[i].up;
            let front = out[i].front;
            let sb = out[i].side_bias;
            for d in Direction::VALUES {
                // withFront(d)
                let direction1 = if d == up {
                    front.get_opposite()
                } else if d == up.get_opposite() {
                    front
                } else {
                    up
                };
                out[i].with_front[dir_ord(d)] =
                    generate_index(direction1, d, sb) as u8;
                // withUp(d)
                let direction1x = if d == front {
                    up.get_opposite()
                } else if d == front.get_opposite() {
                    up
                } else {
                    front
                };
                out[i].with_up[dir_ord(d)] =
                    generate_index(d, direction1x, sb) as u8;
            }
            for sb2 in SideBias::VALUES {
                out[i].with_side_bias[sb2 as usize] =
                    generate_index(up, front, sb2) as u8;
            }
        }
        out.try_into().unwrap()
    })
}
