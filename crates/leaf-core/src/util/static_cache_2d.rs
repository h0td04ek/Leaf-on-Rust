//! Перенос `net.minecraft.util.StaticCache2D<T>`.

/// `StaticCache2D<T>` — двумерный кэш с жёсткими границами [minX..maxX)×[minZ..maxZ).
pub struct StaticCache2D<T> {
    min_x: i32,
    min_z: i32,
    size_x: usize,
    size_z: usize,
    cache: Vec<T>,
}

/// `StaticCache2D.Initializer<T>`.
pub trait Initializer<T> {
    fn get(&self, x: i32, z: i32) -> T;
}

impl<T, F: Fn(i32, i32) -> T> Initializer<T> for F {
    fn get(&self, x: i32, z: i32) -> T {
        self(x, z)
    }
}

impl<T> StaticCache2D<T> {
    /// `create(centerX, centerZ, size, initializer)` — квадрат 2*size+1 вокруг центра.
    pub fn create(center_x: i32, center_z: i32, size: i32, initializer: impl Initializer<T>) -> Self {
        let i = center_x - size;
        let i1 = center_z - size;
        let i2 = (2 * size + 1) as usize;
        Self::new(i, i1, i2, i2, initializer)
    }

    fn new(min_x: i32, min_z: i32, size_x: usize, size_z: usize, initializer: impl Initializer<T>) -> Self {
        let mut cache = Vec::with_capacity(size_x * size_z);
        for x in min_x..min_x + size_x as i32 {
            for z in min_z..min_z + size_z as i32 {
                cache.push(initializer.get(x, z));
            }
        }
        Self { min_x, min_z, size_x, size_z, cache }
    }

    /// `forEach(action)` — порядок заполнения: x внешний, z внутренний.
    pub fn for_each(&self, mut action: impl FnMut(&T)) {
        for v in &self.cache {
            action(v);
        }
    }

    /// `get(x, z)` — вне диапазона → panic.
    pub fn get(&self, x: i32, z: i32) -> &T {
        assert!(self.contains(x, z), "Requested out of range value ({},{}) from {}", x, z, self);
        &self.cache[self.get_index(x, z)]
    }

    /// `get` с мутацией.
    pub fn get_mut(&mut self, x: i32, z: i32) -> &mut T {
        assert!(self.contains(x, z), "Requested out of range value ({},{}) from {}", x, z, self);
        let i = self.get_index(x, z);
        &mut self.cache[i]
    }

    pub fn contains(&self, x: i32, z: i32) -> bool {
        let i = x - self.min_x;
        let i1 = z - self.min_z;
        i >= 0 && (i as usize) < self.size_x && i1 >= 0 && (i1 as usize) < self.size_z
    }

    fn get_index(&self, x: i32, z: i32) -> usize {
        let i = (x - self.min_x) as usize;
        let i1 = (z - self.min_z) as usize;
        i * self.size_z + i1
    }
}

impl<T> std::fmt::Display for StaticCache2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StaticCache2D[{}, {}, {}, {}]",
            self.min_x,
            self.min_z,
            self.min_x + self.size_x as i32,
            self.min_z + self.size_z as i32
        )
    }
}
