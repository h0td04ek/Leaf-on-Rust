//! Перенос `net.minecraft.util.BitStorage`, `SimpleBitStorage`,
//! `ZeroBitStorage` — постраничное хранилище значений фиксированной
//! разрядности (битовые палитры чанков).

/// `BitStorage` — хранилище `size` значений по `bits` бит каждое.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BitStorage {
    /// `ZeroBitStorage` — все значения равны 0, биты не хранятся.
    Zero {
        /// `size`.
        size: usize,
    },
    /// `SimpleBitStorage` — плотная упаковка little-endian по `long`.
    Simple(SimpleBitStorage),
}

impl BitStorage {
    /// `getAndSet` — возвращает старое значение.
    pub fn get_and_set(&mut self, index: usize, value: i32) -> i32 {
        match self {
            BitStorage::Zero { size } => {
                assert!(index < *size, "Index {} out of bounds {}", index, size);
                0
            }
            BitStorage::Simple(s) => s.get_and_set(index, value),
        }
    }

    /// `set`.
    pub fn set(&mut self, index: usize, value: i32) {
        match self {
            BitStorage::Zero { size } => {
                assert!(index < *size, "Index {} out of bounds {}", index, size);
                assert_eq!(value, 0, "Can't set value {} in zero-bit storage", value);
            }
            BitStorage::Simple(s) => s.set(index, value),
        }
    }

    /// `get`.
    pub fn get(&self, index: usize) -> i32 {
        match self {
            BitStorage::Zero { size } => {
                assert!(index < *size, "Index {} out of bounds {}", index, size);
                0
            }
            BitStorage::Simple(s) => s.get(index),
        }
    }

    /// `getRaw` — сырые `long[]` (для ZeroBitStorage — пустой массив,
    /// сериализатор пишет фиксированную длину по `size`/`bits`).
    pub fn get_raw(&self) -> &[i64] {
        match self {
            BitStorage::Zero { .. } => &[],
            BitStorage::Simple(s) => &s.data,
        }
    }

    /// `getRaw` изменяемо (десериализация читает прямо в массив).
    pub fn get_raw_mut(&mut self) -> &mut [i64] {
        match self {
            BitStorage::Zero { .. } => &mut [],
            BitStorage::Simple(s) => &mut s.data,
        }
    }

    /// `getSize`.
    pub fn get_size(&self) -> usize {
        match self {
            BitStorage::Zero { size } => *size,
            BitStorage::Simple(s) => s.size,
        }
    }

    /// `getBits`.
    pub fn get_bits(&self) -> i32 {
        match self {
            BitStorage::Zero { .. } => 0,
            BitStorage::Simple(s) => s.bits,
        }
    }

    /// `getAll` — итерация всех значений по порядку.
    pub fn get_all(&self, mut consumer: impl FnMut(i32)) {
        match self {
            BitStorage::Zero { size } => {
                for _ in 0..*size {
                    consumer(0);
                }
            }
            BitStorage::Simple(s) => s.get_all(consumer),
        }
    }

    /// `unpack` — развернуть в `int[]`.
    pub fn unpack(&self, array: &mut [i32]) {
        match self {
            BitStorage::Zero { size } => {
                assert_eq!(array.len(), *size);
                for v in array.iter_mut() {
                    *v = 0;
                }
            }
            BitStorage::Simple(s) => s.unpack(array),
        }
    }

    /// `copy`.
    pub fn copy(&self) -> BitStorage {
        self.clone()
    }
}

/// `SimpleBitStorage` — значения упакованы LSB-first: значение `i`
/// лежит в `long[i / valuesPerLong]` со сдвигом `(i % valuesPerLong) * bits`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimpleBitStorage {
    /// Сырые `long[]`.
    pub data: Vec<i64>,
    /// `bits` — разрядность значения (1..=32).
    pub bits: i32,
    /// `mask`.
    mask: i64,
    /// `size` — число значений.
    size: usize,
    /// `valuesPerLong = 64 / bits`.
    values_per_long: usize,
}

impl SimpleBitStorage {
    /// `SimpleBitStorage(bits, size)`.
    pub fn new(bits: i32, size: usize) -> Self {
        assert!((1..=32).contains(&bits), "Bits {} out of range 1..32", bits);
        let mask = (1i64 << bits) - 1;
        let values_per_long = (64 / bits) as usize;
        let long_count = (size + values_per_long - 1) / values_per_long;
        if size > 4096 {
            panic!("Size > 4096 not supported");
        }
        Self {
            data: vec![0i64; long_count],
            bits,
            mask,
            size,
            values_per_long,
        }
    }

    /// `SimpleBitStorage(bits, size, data)` — из существующего массива.
    pub fn from_raw(bits: i32, size: usize, data: Vec<i64>) -> Result<Self, String> {
        let storage = Self::new(bits, size);
        let expected = storage.data.len();
        if data.len() != expected {
            return Err(format!(
                "Invalid length given for storage, got: {} but expected: {}",
                data.len(),
                expected
            ));
        }
        Ok(Self { data, ..storage })
    }

    /// `SimpleBitStorage(bits, size, int[])` — из int-массива (NBT-путь).
    pub fn from_ints(bits: i32, size: usize, ints: &[i32]) -> Result<Self, String> {
        let mut storage = Self::new(bits, size);
        let values_per_long = storage.values_per_long;
        let bits_u = bits as u32;
        let mut i = 0usize;
        let mut i1 = 0usize;
        while i1 + values_per_long <= size {
            let mut l: i64 = 0;
            for i2 in (0..values_per_long).rev() {
                l <<= bits_u;
                l |= (ints[i1 + i2] as i64) & storage.mask;
            }
            storage.data[i] = l;
            i += 1;
            i1 += values_per_long;
        }
        let rest = size - i1;
        if rest > 0 {
            let mut l: i64 = 0;
            for i4 in (0..rest).rev() {
                l <<= bits_u;
                l |= (ints[i1 + i4] as i64) & storage.mask;
            }
            storage.data[i] = l;
        }
        Ok(storage)
    }

    fn cell_index(&self, index: usize) -> usize {
        index / self.values_per_long
    }

    fn bit_offset(&self, index: usize) -> u32 {
        ((index % self.values_per_long) * self.bits as usize) as u32
    }

    /// `getAndSet`.
    pub fn get_and_set(&mut self, index: usize, value: i32) -> i32 {
        assert!(index < self.size, "Index {} out of bounds {}", index, self.size);
        let cell = self.cell_index(index);
        let offset = self.bit_offset(index);
        let data = self.data[cell] as u64;
        let mask = self.mask as u64;
        let old = ((data >> offset) & mask) as i32;
        let cleared = data & !(mask << offset);
        self.data[cell] = (cleared | (((value as u64) & mask) << offset)) as i64;
        old
    }

    /// `set`.
    pub fn set(&mut self, index: usize, value: i32) {
        assert!(index < self.size, "Index {} out of bounds {}", index, self.size);
        let cell = self.cell_index(index);
        let offset = self.bit_offset(index);
        let data = self.data[cell] as u64;
        let mask = self.mask as u64;
        let cleared = data & !(mask << offset);
        self.data[cell] = (cleared | (((value as u64) & mask) << offset)) as i64;
    }

    /// `get`.
    pub fn get(&self, index: usize) -> i32 {
        assert!(index < self.size, "Index {} out of bounds {}", index, self.size);
        let cell = self.cell_index(index);
        let offset = self.bit_offset(index);
        ((self.data[cell] as u64 >> offset) & (self.mask as u64)) as i32
    }

    /// `getAll`.
    pub fn get_all(&self, mut consumer: impl FnMut(i32)) {
        let mut i = 0usize;
        for &l in &self.data {
            let mut l = l as u64;
            for _ in 0..self.values_per_long {
                consumer((l & self.mask as u64) as i32);
                l >>= self.bits as u32;
                i += 1;
                if i >= self.size {
                    return;
                }
            }
        }
    }

    /// `unpack`.
    pub fn unpack(&self, array: &mut [i32]) {
        assert_eq!(array.len(), self.size);
        let mut i1 = 0usize;
        for (i2, &l) in self.data.iter().enumerate() {
            if i2 == self.data.len() - 1 {
                break;
            }
            let mut l = l as u64;
            for i3 in 0..self.values_per_long {
                array[i1 + i3] = (l & self.mask as u64) as i32;
                l >>= self.bits as u32;
            }
            i1 += self.values_per_long;
        }
        let rest = self.size - i1;
        if rest > 0 {
            let mut l = self.data[self.data.len() - 1] as u64;
            for i3 in 0..rest {
                array[i1 + i3] = (l & self.mask as u64) as i32;
                l >>= self.bits as u32;
            }
        }
    }
}
