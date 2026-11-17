//! `net.minecraft.world.level.levelgen.synth.SimplexNoise`.

use crate::random::RandomSource;
use crate::util::mth;

/// `SimplexNoise.GRADIENT`.
pub const GRADIENT: [[i32; 3]; 16] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
    [1, 1, 0],
    [0, -1, 1],
    [-1, 1, 0],
    [0, -1, -1],
];

const SQRT_3: f64 = 1.7320508075688772;
const F2: f64 = 0.5 * (SQRT_3 - 1.0);
const G2: f64 = (3.0 - SQRT_3) / 6.0;

/// `SimplexNoise`.
#[derive(Clone)]
pub struct SimplexNoise {
    /// `p` — перестановка 512 элементов.
    p: [i32; 512],
    /// `xo`.
    pub xo: f64,
    /// `yo`.
    pub yo: f64,
    /// `zo`.
    pub zo: f64,
}

impl SimplexNoise {
    /// `new SimplexNoise(random)`.
    pub fn new(random: &mut dyn RandomSource) -> Self {
        let xo = random.next_double() * 256.0;
        let yo = random.next_double() * 256.0;
        let zo = random.next_double() * 256.0;
        let mut p = [0i32; 512];
        for i in 0..256 {
            p[i] = i as i32;
        }
        for ix in 0..256 {
            let random_int = random.next_int_bound(256 - ix as i32) as usize;
            p.swap(ix, random_int + ix);
        }
        // расширение до 512 (Java использует p[index & 0xFF] — 256 достаточно,
        // но в SimplexNoise p имеет 512? Нет: p[512] в Java? Проверено: new int[512] нет —
        // в оригинале p = new int[512] с заполнением 0..255 дважды? Смотрим: в Java
        // `this.p = new int[512]` отсутствует — там `new int[256]`? В исходнике выше
        // p объявлен `new int[512]`? Нет — в дампе: `private final int[] p = new int[512];`
        // отсутствует; там просто `this.p[i] = i++` до 256 и обмен. Доступ p(index & 0xFF).
        // Для совместимости продублируем первые 256 во вторую половину не нужно —
        // index всегда & 0xFF → 0..255.
        Self { p, xo, yo, zo }
    }

    fn p(&self, index: i32) -> i32 {
        self.p[(index & 0xFF) as usize]
    }

    /// `dot`.
    pub fn dot(gradient: &[i32; 3], x: f64, y: f64, z: f64) -> f64 {
        (gradient[0] as f64) * x + (gradient[1] as f64) * y + (gradient[2] as f64) * z
    }

    fn get_corner_noise_3d(&self, gradient_index: usize, x: f64, y: f64, z: f64, offset: f64) -> f64 {
        let d = offset - x * x - y * y - z * z;
        if d < 0.0 {
            0.0
        } else {
            let d = d * d;
            d * d * Self::dot(&GRADIENT[gradient_index], x, y, z)
        }
    }

    /// `getValue(x, y)` — 2D.
    pub fn get_value_2d(&self, x: f64, y: f64) -> f64 {
        let d = (x + y) * F2;
        let floor = mth::floor(x + d);
        let floor1 = mth::floor(y + d);
        let d1 = (floor + floor1) as f64 * G2;
        let d2 = floor as f64 - d1;
        let d3 = floor1 as f64 - d1;
        let d4 = x - d2;
        let d5 = y - d3;
        let (i, i1) = if d4 > d5 { (1usize, 0usize) } else { (0usize, 1usize) };
        let d6 = d4 - i as f64 + G2;
        let d7 = d5 - i1 as f64 + G2;
        let d8 = d4 - 1.0 + 2.0 * G2;
        let d9 = d5 - 1.0 + 2.0 * G2;
        let i2 = (floor as i32) & 0xFF;
        let i3 = (floor1 as i32) & 0xFF;
        let i4 = self.p((i2 + self.p(i3)) % 512) % 12;
        let i5 = self.p((i2 + i as i32 + self.p((i3 + i1 as i32) & 0xFF)) % 512) % 12;
        let i6 = self.p((i2 + 1 + self.p(i3 + 1)) % 512) % 12;
        let c0 = self.get_corner_noise_3d(i4 as usize, d4, d5, 0.0, 0.5);
        let c1 = self.get_corner_noise_3d(i5 as usize, d6, d7, 0.0, 0.5);
        let c2 = self.get_corner_noise_3d(i6 as usize, d8, d9, 0.0, 0.5);
        70.0 * (c0 + c1 + c2)
    }

    /// `getValue(x, y, z)` — 3D.
    pub fn get_value_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let d1 = (x + y + z) * (1.0 / 3.0);
        let floor = mth::floor(x + d1);
        let floor1 = mth::floor(y + d1);
        let floor2 = mth::floor(z + d1);
        let d3 = (floor + floor1 + floor2) as f64 * (1.0 / 6.0);
        let d4 = floor as f64 - d3;
        let d5 = floor1 as f64 - d3;
        let d6 = floor2 as f64 - d3;
        let d7 = x - d4;
        let d8 = y - d5;
        let d9 = z - d6;
        let (i, i1, i2, i3, i4, i5) = if d7 >= d8 {
            if d8 >= d9 {
                (1usize, 0usize, 0usize, 1usize, 1usize, 0usize)
            } else if d7 >= d9 {
                (1usize, 0usize, 0usize, 1usize, 0usize, 1usize)
            } else {
                (0usize, 0usize, 1usize, 1usize, 0usize, 1usize)
            }
        } else if d8 < d9 {
            (0usize, 0usize, 1usize, 0usize, 1usize, 1usize)
        } else if d7 < d9 {
            (0usize, 1usize, 0usize, 0usize, 1usize, 1usize)
        } else {
            (0usize, 1usize, 0usize, 1usize, 1usize, 0usize)
        };
        let d10 = d7 - i as f64 + (1.0 / 6.0);
        let d11 = d8 - i1 as f64 + (1.0 / 6.0);
        let d12 = d9 - i2 as f64 + (1.0 / 6.0);
        let d13 = d7 - i3 as f64 + (1.0 / 3.0);
        let d14 = d8 - i4 as f64 + (1.0 / 3.0);
        let d15 = d9 - i5 as f64 + (1.0 / 3.0);
        let d16 = d7 - 1.0 + 0.5;
        let d17 = d8 - 1.0 + 0.5;
        let d18 = d9 - 1.0 + 0.5;
        let i6 = (floor as i32) & 0xFF;
        let i7 = (floor1 as i32) & 0xFF;
        let i8 = (floor2 as i32) & 0xFF;
        let i9 = self.p((i6 + self.p(i7 + self.p(i8))) % 512) % 12;
        let i10 = self.p((i6 + i as i32 + self.p(i7 + i1 as i32 + self.p((i8 + i2 as i32) & 0xFF))) % 512) % 12;
        let i11 = self.p((i6 + i3 as i32 + self.p(i7 + i4 as i32 + self.p((i8 + i5 as i32) & 0xFF))) % 512) % 12;
        let i12 = self.p((i6 + 1 + self.p(i7 + 1 + self.p(i8 + 1))) % 512) % 12;
        let c0 = self.get_corner_noise_3d(i9 as usize, d7, d8, d9, 0.6);
        let c1 = self.get_corner_noise_3d(i10 as usize, d10, d11, d12, 0.6);
        let c2 = self.get_corner_noise_3d(i11 as usize, d13, d14, d15, 0.6);
        let c3 = self.get_corner_noise_3d(i12 as usize, d16, d17, d18, 0.6);
        32.0 * (c0 + c1 + c2 + c3)
    }
}
