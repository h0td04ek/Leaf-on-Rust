//! `net.minecraft.core.Cursor3D` — итератор по 3D-сетке блоков
//! с классификацией ячеек (inside/face/edge/corner).

/// Типы ячеек курсора.
pub const TYPE_INSIDE: i32 = 0;
pub const TYPE_FACE: i32 = 1;
pub const TYPE_EDGE: i32 = 2;
pub const TYPE_CORNER: i32 = 3;

/// `Cursor3D` — обходит все ячейки `[origin..end]` включительно,
/// x — самый быстрый индекс.
#[derive(Debug)]
pub struct Cursor3D {
    origin_x: i32,
    origin_y: i32,
    origin_z: i32,
    width: i32,
    height: i32,
    depth: i32,
    end: i64,
    index: i64,
    x: i32,
    y: i32,
    z: i32,
}

impl Cursor3D {
    /// `new Cursor3D(originX, originY, originZ, endX, endY, endZ)`.
    pub fn new(origin_x: i32, origin_y: i32, origin_z: i32, end_x: i32, end_y: i32, end_z: i32) -> Self {
        Self {
            origin_x,
            origin_y,
            origin_z,
            width: end_x - origin_x + 1,
            height: end_y - origin_y + 1,
            depth: end_z - origin_z + 1,
            end: (end_x - origin_x + 1) as i64 * (end_y - origin_y + 1) as i64 * (end_z - origin_z + 1) as i64,
            index: 0,
            x: 0,
            y: 0,
            z: 0,
        }
    }

    /// `advance()` — перейти к следующей ячейке. `false` — конец.
    pub fn advance(&mut self) -> bool {
        if self.index == self.end {
            return false;
        }
        self.x = (self.index % self.width as i64) as i32;
        let i = self.index / self.width as i64;
        self.y = (i % self.height as i64) as i32;
        self.z = (i / self.height as i64) as i32;
        self.index += 1;
        true
    }

    /// `nextX()`.
    pub fn next_x(&self) -> i32 {
        self.origin_x + self.x
    }

    /// `nextY()`.
    pub fn next_y(&self) -> i32 {
        self.origin_y + self.y
    }

    /// `nextZ()`.
    pub fn next_z(&self) -> i32 {
        self.origin_z + self.z
    }

    /// `getNextType()` — сколько граней ячейки лежит на границе
    /// диапазона (0=внутри, 1=грань, 2=ребро, 3=угол).
    pub fn get_next_type(&self) -> i32 {
        let mut i = 0;
        if self.x == 0 || self.x == self.width - 1 {
            i += 1;
        }
        if self.y == 0 || self.y == self.height - 1 {
            i += 1;
        }
        if self.z == 0 || self.z == self.depth - 1 {
            i += 1;
        }
        i
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor3d_order_and_type() {
        let mut c = Cursor3D::new(0, 0, 0, 1, 1, 0);
        // width=2,height=2,depth=1 -> 4 ячейки; все на границах.
        let mut cells = Vec::new();
        while c.advance() {
            cells.push((c.next_x(), c.next_y(), c.next_z(), c.get_next_type()));
        }
        assert_eq!(cells.len(), 4);
        // x меняется быстрее всего.
        assert_eq!(cells[0], (0, 0, 0, 3));
        assert_eq!(cells[1], (1, 0, 0, 3));
        assert_eq!(cells[2], (0, 1, 0, 3));
        assert_eq!(cells[3], (1, 1, 0, 3));
        assert!(!c.advance());
    }

    #[test]
    fn cursor3d_inside_type() {
        let mut c = Cursor3D::new(0, 0, 0, 2, 2, 2);
        let mut saw_inside = false;
        let mut saw_corner = false;
        while c.advance() {
            match c.get_next_type() {
                TYPE_INSIDE => saw_inside = true,
                TYPE_CORNER => saw_corner = true,
                _ => {}
            }
        }
        assert!(saw_inside && saw_corner);
    }
}
