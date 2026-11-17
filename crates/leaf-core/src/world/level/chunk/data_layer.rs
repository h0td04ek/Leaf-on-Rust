//! Перенос `net.minecraft.world.level.chunk.DataLayer` —
//! nibble-хранилище 16×16×16 (2048 байт) для light-данных.

/// `DataLayer` — `byte[2048]`, 4 бита на ячейку, индекс
/// `(y << 8) | (z << 4) | x` (младший/старший ниббл по чётности).
pub struct DataLayer {
    data: Box<[u8; 2048]>,
}

impl DataLayer {
    /// `new` — все нули.
    pub fn new() -> Self {
        Self { data: Box::new([0u8; 2048]) }
    }

    /// `new(byte[])` — из сырых данных.
    pub fn from_raw(data: [u8; 2048]) -> Self {
        Self { data: Box::new(data) }
    }

    /// Сырые данные.
    pub fn get_data(&self) -> &[u8; 2048] {
        &self.data
    }

    /// Изменяемые сырые данные.
    pub fn get_data_mut(&mut self) -> &mut [u8; 2048] {
        &mut self.data
    }

    /// `isEmpty` — все значения 0.
    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|&b| b == 0)
    }

    fn index(x: i32, y: i32, z: i32) -> usize {
        assert!((0..16).contains(&x) && (0..16).contains(&y) && (0..16).contains(&z));
        ((y << 8) | (z << 4) | x) as usize
    }

    /// `get(x, y, z)` — значение 0..=15.
    pub fn get(&self, x: i32, y: i32, z: i32) -> u8 {
        let i = Self::index(x, y, z);
        if i & 1 == 0 {
            self.data[i >> 1] & 15
        } else {
            self.data[i >> 1] >> 4
        }
    }

    /// `set(x, y, z, value)`.
    pub fn set(&mut self, x: i32, y: i32, z: i32, value: u8) {
        assert!(value < 16);
        let i = Self::index(x, y, z);
        let idx = i >> 1;
        if i & 1 == 0 {
            self.data[idx] = (self.data[idx] & 0xF0) | (value & 15);
        } else {
            self.data[idx] = (self.data[idx] & 0x0F) | ((value & 15) << 4);
        }
    }

    /// `setAll` — заполнить все ячейки значением.
    pub fn set_all(&mut self, value: u8) {
        assert!(value < 16);
        let b = (value & 15) | ((value & 15) << 4);
        self.data.fill(b);
    }
}

impl Default for DataLayer {
    fn default() -> Self {
        Self::new()
    }
}
