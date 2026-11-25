use std::sync::OnceLock;

use leaf_core::core::direction::Direction;

const STATE_BYTES: usize = 7;
const STATES: &[u8; 29671 * STATE_BYTES] = include_bytes!("../data/fluid-state-faces-774.bin");

fn profile(state: i32) -> &'static [u8] {
    let index = state as usize * STATE_BYTES;
    &STATES[index..index + STATE_BYTES]
}

pub(super) fn can_waterlog(state: i32) -> bool {
    flags(state) & 1 != 0
}

pub(super) fn waterlogged(state: i32) -> bool {
    flags(state) & 2 != 0
}

pub(super) fn blocks_motion(state: i32) -> bool {
    flags(state) & 4 != 0
}

pub(super) fn supports_falling_current(state: i32, direction: Direction) -> bool {
    flags(state) & (1 << (direction.get_3d_data_value() + 1)) != 0
}

fn flags(state: i32) -> u8 {
    STATES[state as usize * STATE_BYTES]
}

pub(super) fn shape_changed(before: i32, after: i32) -> bool {
    profile(before) != profile(after)
}

pub(super) fn can_pass(from: i32, to: i32, direction: Direction) -> bool {
    let face = |state, direction: Direction| {
        usize::from(profile(state)[1 + direction.get_3d_data_value() as usize])
    };
    let a = face(from, direction);
    let b = face(to, direction.get_opposite());
    if a == 1 || b == 1 {
        return false;
    }
    if a == 0 && b == 0 {
        return true;
    }
    let table = FacePairs::get();
    !table.occludes[a * table.count + b]
}

type Rectangle = [f64; 4];

struct FacePairs {
    occludes: Vec<bool>,
    count: usize,
}

impl FacePairs {
    fn get() -> &'static Self {
        static TABLE: OnceLock<FacePairs> = OnceLock::new();
        TABLE.get_or_init(|| {
            let faces = include_str!("../data/fluid-faces-774.txt")
                .lines()
                .skip(1)
                .enumerate()
                .map(|(index, line)| {
                    let mut fields = line.split_ascii_whitespace();
                    assert_eq!(fields.next().unwrap().parse::<usize>().unwrap(), index);
                    fields
                        .map(|rectangle| {
                            let mut values = rectangle.split(',');
                            std::array::from_fn(|_| values.next().unwrap().parse().unwrap())
                        })
                        .collect::<Vec<Rectangle>>()
                })
                .collect::<Vec<_>>();
            let count = faces.len();
            let mut occludes = vec![false; count * count];
            for (a, first) in faces.iter().enumerate() {
                for (b, second) in faces.iter().enumerate().skip(a) {
                    let result = covers_face(first, second);
                    occludes[a * count + b] = result;
                    occludes[b * count + a] = result;
                }
            }
            Self { occludes, count }
        })
    }
}

/// Projected face rectangles cover the interface when their union leaves no
/// gap. Testing each interval between rectangle edges also handles thin
/// collision shapes whose coordinates are finer than 1/16 of a block.
fn covers_face(first: &[Rectangle], second: &[Rectangle]) -> bool {
    let rectangles = || first.iter().chain(second);
    let mut edges = vec![0.0, 1.0];
    for &[left, _, right, _] in rectangles() {
        edges.extend([left, right]);
    }
    edges.sort_by(f64::total_cmp);
    edges.dedup();
    let mut spans = Vec::new();
    for edges in edges.windows(2) {
        let middle = (edges[0] + edges[1]) * 0.5;
        spans.clear();
        spans.extend(rectangles().filter_map(|&[left, low, right, high]| {
            (left <= middle && middle < right).then_some((low, high))
        }));
        spans.sort_by(|(a, _), (b, _)| a.total_cmp(b));
        let mut covered: f64 = 0.0;
        for &(low, high) in &spans {
            if low > covered + 1e-7 {
                break;
            }
            covered = covered.max(high);
        }
        if covered < 1.0 - 1e-7 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::Blocks;

    #[test]
    fn opposing_partial_shapes_can_occlude_the_whole_interface() {
        let slab = Blocks::vanilla().named("oak_slab").unwrap();
        let bottom = slab.state(slab.default, &[("type", "bottom")]);
        let top = slab.state(slab.default, &[("type", "top")]);
        let double = slab.state(
            slab.default,
            &[("type", "double"), ("waterlogged", "false")],
        );
        assert!(can_waterlog(bottom) && can_waterlog(top));
        assert!(!can_waterlog(double));
        assert!(!can_pass(bottom, top, Direction::EAST));
        assert!(can_pass(bottom, bottom, Direction::EAST));
        assert!(!can_pass(bottom, 0, Direction::DOWN));
        assert!(!can_pass(top, 0, Direction::UP));
        assert!(can_pass(top, 0, Direction::DOWN));
    }
}
