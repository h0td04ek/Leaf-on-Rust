//! Перенос `net.minecraft.util.BitStorage`, `SimpleBitStorage`,
//! `ZeroBitStorage`.
//! Paper-оптимизации через magic-числа заменены на прямое деление
//! `index / values_per_long` — семантика идентична.

/// `BitStorage` — трейт битового хранилища.
pub trait BitStorage {
    fn get_and_set(&mut self, index: usize, value: i32) -> i32;
    fn set(&mut self, index: usize, value: i32);
    fn get(&self, index: usize) -> i32;
    fn get_raw(&self) -> &[i64];
    fn get_size(&self) -> usize;
    fn get_bits(&self) -> u32;
    fn get_all(&self, consumer: &mut dyn FnMut(i32));
    fn unpack(&self, array: &mut [i32]);
    fn copy(&self) -> Box<dyn BitStorage>;

    /// `moonrise$countEntries` (Paper block counting) —
    /// индексы по каждому значению палитры.
    fn count_entries(&self) -> std::collections::HashMap<i32, Vec<i32>> {
        let size = self.get_size();
        let mut ret: std::collections::HashMap<i32, Vec<i32>> = std::collections::HashMap::new();
        for index in 0..size {
            let palette_idx = self.get(index);
            ret.entry(palette_idx).or_default().push(index as i32);
        }
        ret
    }
}

/// `SimpleBitStorage` — packed long[] хранилище.
pub struct SimpleBitStorage {
    data: Vec<i64>,
    bits: u32,
    mask: i64,
    size: usize,
    values_per_long: usize,
}

impl SimpleBitStorage {
    /// `new SimpleBitStorage(bits, size)`.
    pub fn new(bits: u32, size: usize) -> Self {
        Self::with_raw(bits, size, None)
    }

    /// `new SimpleBitStorage(bits, size, long[] data)`.
    /// При Some(data) длина должна совпадать с расчётной.
    pub fn with_raw(bits: u32, size: usize, data: Option<Vec<i64>>) -> Self {
        assert!((1..=32).contains(&bits), "bits must be in [1, 32]");
        let mask = (1i64 << bits) - 1;
        let values_per_long = 64 / bits as usize;
        let expected = (size + values_per_long - 1) / values_per_long;
        let data = match data {
            Some(d) => {
                assert_eq!(
                    d.len(),
                    expected,
                    "Invalid length given for storage, got: {} but expected: {}",
                    d.len(),
                    expected
                );
                d
            }
            None => vec![0i64; expected],
        };
        if size > 4096 {
            // Paper-оптимизация не поддерживает >4096; оставляем как в Java
            // (SimpleBitStorage без BETTER_MAGIC работал для любых size).
        }
        Self { data, bits, mask, size, values_per_long }
    }

    /// `new SimpleBitStorage(bits, size, int[] data)` — упаковка int[].
    pub fn from_ints(bits: u32, size: usize, data: &[i32]) -> Self {
        let mut storage = Self::new(bits, size);
        let mut i = 0usize;
        let mut i1 = 0usize;
        while i1 + storage.values_per_long <= size {
            let mut l = 0i64;
            for i2 in (0..storage.values_per_long).rev() {
                l <<= bits;
                l |= (data[i1 + i2] as i64) & storage.mask;
            }
            storage.data[i] = l;
            i += 1;
            i1 += storage.values_per_long;
        }
        let i3 = size - i1;
        if i3 > 0 {
            let mut l1 = 0i64;
            for i4 in (0..i3).rev() {
                l1 <<= bits;
                l1 |= (data[i1 + i4] as i64) & storage.mask;
            }
            storage.data[i] = l1;
        }
        storage
    }

    /// Индекс ячейки long и битовый сдвиг — вместо magic-деления.
    fn locate(&self, index: usize) -> (usize, u32) {
        let cell = index / self.values_per_long;
        let bit = ((index % self.values_per_long) * self.bits as usize) as u32;
        (cell, bit)
    }
}

impl BitStorage for SimpleBitStorage {
    fn get_and_set(&mut self, index: usize, value: i32) -> i32 {
        let (cell, bit) = self.locate(index);
        let data = self.data[cell];
        let mask = self.mask;
        let write = (data & !(mask << bit)) | (((value as i64) & mask) << bit);
        self.data[cell] = write;
        (data >> bit & mask) as i32
    }

    fn set(&mut self, index: usize, value: i32) {
        let (cell, bit) = self.locate(index);
        let data = self.data[cell];
        let mask = self.mask;
        self.data[cell] = (data & !(mask << bit)) | (((value as i64) & mask) << bit);
    }

    fn get(&self, index: usize) -> i32 {
        let (cell, bit) = self.locate(index);
        (self.data[cell] >> bit & self.mask) as i32
    }

    fn get_raw(&self) -> &[i64] {
        &self.data
    }

    fn get_size(&self) -> usize {
        self.size
    }

    fn get_bits(&self) -> u32 {
        self.bits
    }

    fn get_all(&self, consumer: &mut dyn FnMut(i32)) {
        let mut i = 0usize;
        for &l0 in &self.data {
            let mut l = l0;
            for _ in 0..self.values_per_long {
                consumer((l & self.mask) as i32);
                l >>= self.bits;
                i += 1;
                if i >= self.size {
                    return;
                }
            }
        }
    }

    fn unpack(&self, array: &mut [i32]) {
        let mut i1 = 0usize;
        for (i2, &l0) in self.data.iter().enumerate() {
            let mut l = l0;
            if i2 == self.data.len() - 1 {
                let rem = self.size - i1;
                for i3 in 0..rem {
                    array[i1 + i3] = (l & self.mask) as i32;
                    l >>= self.bits;
                }
            } else {
                for i3 in 0..self.values_per_long {
                    array[i1 + i3] = (l & self.mask) as i32;
                    l >>= self.bits;
                }
                i1 += self.values_per_long;
            }
        }
    }

    fn copy(&self) -> Box<dyn BitStorage> {
        Box::new(Self {
            data: self.data.clone(),
            bits: self.bits,
            mask: self.mask,
            size: self.size,
            values_per_long: self.values_per_long,
        })
    }
}

/// `ZeroBitStorage` — все значения 0.
pub struct ZeroBitStorage {
    size: usize,
}

impl ZeroBitStorage {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl BitStorage for ZeroBitStorage {
    fn get_and_set(&mut self, _index: usize, _value: i32) -> i32 {
        0
    }

    fn set(&mut self, _index: usize, _value: i32) {}

    fn get(&self, _index: usize) -> i32 {
        0
    }

    fn get_raw(&self) -> &[i64] {
        &[]
    }

    fn get_size(&self) -> usize {
        self.size
    }

    fn get_bits(&self) -> u32 {
        0
    }

    fn get_all(&self, consumer: &mut dyn FnMut(i32)) {
        for _ in 0..self.size {
            consumer(0);
        }
    }

    fn unpack(&self, array: &mut [i32]) {
        for v in array.iter_mut().take(self.size) {
            *v = 0;
        }
    }

    fn copy(&self) -> Box<dyn BitStorage> {
        Box::new(ZeroBitStorage { size: self.size })
    }
}

/// Фабрика как в Java (`SimpleBitStorage` vs `ZeroBitStorage` по bits).
pub fn create(bits: u32, size: usize) -> Box<dyn BitStorage> {
    if bits == 0 {
        Box::new(ZeroBitStorage::new(size))
    } else {
        Box::new(SimpleBitStorage::new(bits, size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_roundtrip() {
        for bits in [1u32, 3, 4, 5, 8, 12] {
            let size = 4096usize;
            let mut s = SimpleBitStorage::new(bits, size);
            let max = (1usize << bits) - 1;
            for i in 0..size {
                let v = (i * 7 % (max + 1)) as i32;
                s.set(i, v);
            }
            for i in 0..size {
                let expected = (i * 7 % (max + 1)) as i32;
                assert_eq!(s.get(i), expected, "bits={bits} i={i}");
            }
        }
    }

    #[test]
    fn get_and_set_returns_old() {
        let mut s = SimpleBitStorage::new(4, 100);
        assert_eq!(s.get_and_set(5, 9), 0);
        assert_eq!(s.get(5), 9);
        assert_eq!(s.get_and_set(5, 3), 9);
        assert_eq!(s.get(5), 3);
    }

    #[test]
    fn from_ints_matches_sets() {
        let bits = 5u32;
        let size = 100usize;
        let data: Vec<i32> = (0..size).map(|i| (i * 13 % 32) as i32).collect();
        let s = SimpleBitStorage::from_ints(bits, size, &data);
        for i in 0..size {
            assert_eq!(s.get(i), data[i], "i={i}");
        }
    }

    #[test]
    fn unpack_and_get_all() {
        let mut s = SimpleBitStorage::new(3, 50);
        for i in 0..50 {
            s.set(i, (i % 8) as i32);
        }
        let mut arr = vec![0i32; 50];
        s.unpack(&mut arr);
        for i in 0..50 {
            assert_eq!(arr[i], (i % 8) as i32);
        }
        let mut count = 0;
        s.get_all(&mut |_| count += 1);
        assert_eq!(count, 50);
    }

    #[test]
    fn zero_storage() {
        let mut z = ZeroBitStorage::new(10);
        assert_eq!(z.get_and_set(3, 5), 0);
        assert_eq!(z.get(3), 0);
        assert_eq!(z.get_bits(), 0);
        let mut arr = vec![7i32; 10];
        z.unpack(&mut arr);
        assert!(arr.iter().all(|&v| v == 0));
    }

    #[test]
    fn copy_works() {
        let mut s = SimpleBitStorage::new(4, 16);
        s.set(3, 11);
        let c = s.copy();
        assert_eq!(c.get(3), 11);
    }
}
