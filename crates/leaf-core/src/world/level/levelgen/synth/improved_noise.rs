//! `net.minecraft.world.level.levelgen.synth.ImprovedNoise` —
//! Perlin-шум (алгоритм Перлина с пермутационной таблицей).

use super::simplex_noise::{SimplexNoise, GRADIENT};
use crate::random::RandomSource;
use crate::util::mth;

/// `FLAT_SIMPLEX_GRAD` — C2ME flat-вариант градиентов (12 градиентов × 4).
const FLAT_SIMPLEX_GRAD: [f64; 64] = [
    1.0, 1.0, 0.0, 0.0,
    -1.0, 1.0, 0.0, 0.0,
    1.0, -1.0, 0.0, 0.0,
    -1.0, -1.0, 0.0, 0.0,
    1.0, 0.0, 1.0, 0.0,
    -1.0, 0.0, 1.0, 0.0,
    1.0, 0.0, -1.0, 0.0,
    -1.0, 0.0, -1.0, 0.0,
    0.0, 1.0, 1.0, 0.0,
    0.0, -1.0, 1.0, 0.0,
    0.0, 1.0, -1.0, 0.0,
    0.0, -1.0, -1.0, 0.0,
    1.0, 1.0, 0.0, 0.0,
    0.0, -1.0, 1.0, 0.0,
    -1.0, 1.0, 0.0, 0.0,
    0.0, -1.0, -1.0, 0.0,
];

const SHIFT_UP_EPSILON: f64 = 1.0e-7;

/// `ImprovedNoise`.
#[derive(Clone)]
pub struct ImprovedNoise {
    /// `p` — байтовая перестановка 256.
    p: [i8; 256],
    /// `xo`.
    pub xo: f64,
    /// `yo`.
    pub yo: f64,
    /// `zo`.
    pub zo: f64,
}

impl ImprovedNoise {
    /// `new ImprovedNoise(random)`.
    pub fn new(random: &mut dyn RandomSource) -> Self {
        let xo = random.next_double() * 256.0;
        let yo = random.next_double() * 256.0;
        let zo = random.next_double() * 256.0;
        let mut p = [0i8; 256];
        for i in 0..256 {
            p[i] = i as i8;
        }
        for i in 0..256 {
            let random_int = random.next_int_bound(256 - i as i32) as usize;
            p.swap(i, i + random_int);
        }
        Self { p, xo, yo, zo }
    }

    /// `noise(x, y, z)`.
    pub fn noise(&self, x: f64, y: f64, z: f64) -> f64 {
        self.noise_with_yscale(x, y, z, 0.0, 0.0)
    }

    /// `noise(x, y, z, yScale, yMax)` — deprecated-вариант с yScale.
    pub fn noise_with_yscale(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64) -> f64 {
        let d = x + self.xo;
        let d1 = y + self.yo;
        let d2 = z + self.zo;
        let floor = d.floor();
        let floor1 = d1.floor();
        let floor2 = d2.floor();
        let d3 = d - floor;
        let d4 = d1 - floor1;
        let d5 = d2 - floor2;
        let d7 = if y_scale != 0.0 {
            let d6 = if y_max >= 0.0 && y_max < d4 { y_max } else { d4 };
            (d6 / y_scale + SHIFT_UP_EPSILON).floor() * y_scale
        } else {
            0.0
        };
        self.sample_and_lerp(
            floor as i32,
            floor1 as i32,
            floor2 as i32,
            d3,
            d4 - d7,
            d5,
            d4,
        )
    }

    /// `noiseWithDerivative` — шум + градиент в `values` ([dx, dy, dz]).
    pub fn noise_with_derivative(&self, x: f64, y: f64, z: f64, values: &mut [f64; 3]) {
        let d = x + self.xo;
        let d1 = y + self.yo;
        let d2 = z + self.zo;
        let floor = d.floor();
        let floor1 = d1.floor();
        let floor2 = d2.floor();
        let d3 = d - floor;
        let d4 = d1 - floor1;
        let d5 = d2 - floor2;
        self.sample_with_derivative(floor as i32, floor1 as i32, floor2 as i32, d3, d4, d5, values);
    }

    #[inline]
    fn p_at(&self, index: usize) -> usize {
        (self.p[index & 0xFF] as u8) as usize
    }

    #[inline]
    fn grad_dot(grad_index: usize, x: f64, y: f64, z: f64) -> f64 {
        SimplexNoise::dot(&GRADIENT[grad_index & 15], x, y, z)
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_and_lerp(
        &self,
        grid_x: i32,
        grid_y: i32,
        grid_z: i32,
        delta_x: f64,
        weird_delta_y: f64,
        delta_z: f64,
        delta_y: f64,
    ) -> f64 {
        let grid_y = (grid_y & 0xFF) as usize;
        let grid_z = (grid_z & 0xFF) as usize;
        let var0 = (grid_x & 0xFF) as usize;
        let var1 = (var0 + 1) & 0xFF;
        let var2 = self.p_at(var0);
        let var3 = self.p_at(var1);
        let var4 = (var2 + grid_y) & 0xFF;
        let var5 = (var3 + grid_y) & 0xFF;
        let var6 = (var2 + grid_y + 1) & 0xFF;
        let var7 = (var3 + grid_y + 1) & 0xFF;
        let var8 = self.p_at(var4);
        let var9 = self.p_at(var5);
        let var10 = self.p_at(var6);
        let var11 = self.p_at(var7);
        let var12 = (var8 + grid_z) & 0xFF;
        let var13 = (var9 + grid_z) & 0xFF;
        let var14 = (var10 + grid_z) & 0xFF;
        let var15 = (var11 + grid_z) & 0xFF;
        let var16 = (var8 + grid_z + 1) & 0xFF;
        let var17 = (var9 + grid_z + 1) & 0xFF;
        let var18 = (var10 + grid_z + 1) & 0xFF;
        let var19 = (var11 + grid_z + 1) & 0xFF;
        let var20 = (self.p[var12] & 15) << 2;
        let var21 = (self.p[var13] & 15) << 2;
        let var22 = (self.p[var14] & 15) << 2;
        let var23 = (self.p[var15] & 15) << 2;
        let var24 = (self.p[var16] & 15) << 2;
        let var25 = (self.p[var17] & 15) << 2;
        let var26 = (self.p[var18] & 15) << 2;
        let var27 = (self.p[var19] & 15) << 2;

        let x1 = delta_x - 1.0;
        let y1 = weird_delta_y - 1.0;
        let z1 = delta_z - 1.0;

        let g = |idx: usize, x: f64, y: f64, z: f64| -> f64 {
            FLAT_SIMPLEX_GRAD[(idx) as usize] * x + FLAT_SIMPLEX_GRAD[(idx + 1) as usize] * y + FLAT_SIMPLEX_GRAD[(idx + 2) as usize] * z
        };

        let var87 = g(var20 as usize, delta_x, weird_delta_y, delta_z);
        let var88 = g(var21 as usize, x1, weird_delta_y, delta_z);
        let var89 = g(var22 as usize, delta_x, y1, delta_z);
        let var90 = g(var23 as usize, x1, y1, delta_z);
        let var91 = g(var24 as usize, delta_x, weird_delta_y, z1);
        let var92 = g(var25 as usize, x1, weird_delta_y, z1);
        let var93 = g(var26 as usize, delta_x, y1, z1);
        let var94 = g(var27 as usize, x1, y1, z1);

        let var95 = delta_x * 6.0 - 15.0;
        let var96 = delta_y * 6.0 - 15.0;
        let var97 = delta_z * 6.0 - 15.0;
        let var98 = delta_x * var95 + 10.0;
        let var99 = delta_y * var96 + 10.0;
        let var100 = delta_z * var97 + 10.0;
        let var101 = delta_x * delta_x * delta_x * var98;
        let var102 = delta_y * delta_y * delta_y * var99;
        let var103 = delta_z * delta_z * delta_z * var100;

        let var113 = var87 + var101 * (var88 - var87);
        let var114 = var93 + var101 * (var94 - var93);
        let var115 = var91 + var101 * (var92 - var91);
        let var116 = var89 + var101 * (var90 - var89);
        let var117 = var114 - var115;
        let var118 = var102 * (var116 - var113);
        let var119 = var102 * var117;
        let var120 = var113 + var118;
        let var121 = var115 + var119;
        var120 + var103 * (var121 - var120)
    }

    fn sample_with_derivative(
        &self,
        grid_x: i32,
        grid_y: i32,
        grid_z: i32,
        delta_x: f64,
        delta_y: f64,
        delta_z: f64,
        noise_values: &mut [f64; 3],
    ) -> f64 {
        let grid_x = (grid_x & 0xFF) as usize;
        let grid_y = (grid_y & 0xFF) as usize;
        let grid_z = (grid_z & 0xFF) as usize;
        let i = self.p_at(grid_x);
        let i1 = self.p_at(grid_x + 1);
        let i2 = self.p_at(i + grid_y);
        let i3 = self.p_at(i + grid_y + 1);
        let i4 = self.p_at(i1 + grid_y);
        let i5 = self.p_at(i1 + grid_y + 1);
        let i6 = self.p_at(i2 + grid_z);
        let i7 = self.p_at(i4 + grid_z);
        let i8 = self.p_at(i3 + grid_z);
        let i9 = self.p_at(i5 + grid_z);
        let i10 = self.p_at(i2 + grid_z + 1);
        let i11 = self.p_at(i4 + grid_z + 1);
        let i12 = self.p_at(i3 + grid_z + 1);
        let i13 = self.p_at(i5 + grid_z + 1);

        let ints = &GRADIENT[i6 & 15];
        let ints1 = &GRADIENT[i7 & 15];
        let ints2 = &GRADIENT[i8 & 15];
        let ints3 = &GRADIENT[i9 & 15];
        let ints4 = &GRADIENT[i10 & 15];
        let ints5 = &GRADIENT[i11 & 15];
        let ints6 = &GRADIENT[i12 & 15];
        let ints7 = &GRADIENT[i13 & 15];

        let d = SimplexNoise::dot(ints, delta_x, delta_y, delta_z);
        let d1 = SimplexNoise::dot(ints1, delta_x - 1.0, delta_y, delta_z);
        let d2 = SimplexNoise::dot(ints2, delta_x, delta_y - 1.0, delta_z);
        let d3 = SimplexNoise::dot(ints3, delta_x - 1.0, delta_y - 1.0, delta_z);
        let d4 = SimplexNoise::dot(ints4, delta_x, delta_y, delta_z - 1.0);
        let d5 = SimplexNoise::dot(ints5, delta_x - 1.0, delta_y, delta_z - 1.0);
        let d6 = SimplexNoise::dot(ints6, delta_x, delta_y - 1.0, delta_z - 1.0);
        let d7 = SimplexNoise::dot(ints7, delta_x - 1.0, delta_y - 1.0, delta_z - 1.0);
        let d8 = mth::smoothstep(delta_x);
        let d9 = mth::smoothstep(delta_y);
        let d10 = mth::smoothstep(delta_z);
        let d11 = mth::lerp3_f64(d8, d9, d10, ints[0] as f64, ints1[0] as f64, ints2[0] as f64, ints3[0] as f64, ints4[0] as f64, ints5[0] as f64, ints6[0] as f64, ints7[0] as f64);
        let d12 = mth::lerp3_f64(d8, d9, d10, ints[1] as f64, ints1[1] as f64, ints2[1] as f64, ints3[1] as f64, ints4[1] as f64, ints5[1] as f64, ints6[1] as f64, ints7[1] as f64);
        let d13 = mth::lerp3_f64(d8, d9, d10, ints[2] as f64, ints1[2] as f64, ints2[2] as f64, ints3[2] as f64, ints4[2] as f64, ints5[2] as f64, ints6[2] as f64, ints7[2] as f64);
        let d14 = mth::lerp2_f64(d9, d10, d1 - d, d3 - d2, d5 - d4, d7 - d6);
        let d15 = mth::lerp2_f64(d10, d8, d2 - d, d6 - d4, d3 - d1, d7 - d5);
        let d16 = mth::lerp2_f64(d8, d9, d4 - d, d5 - d1, d6 - d2, d7 - d3);
        let d17 = mth::smoothstep_derivative(delta_x);
        let d18 = mth::smoothstep_derivative(delta_y);
        let d19 = mth::smoothstep_derivative(delta_z);
        let d20 = d11 + d17 * d14;
        let d21 = d12 + d18 * d15;
        let d22 = d13 + d19 * d16;
        noise_values[0] += d20;
        noise_values[1] += d21;
        noise_values[2] += d22;
        mth::lerp3_f64(d8, d9, d10, d, d1, d2, d3, d4, d5, d6, d7)
    }

    /// `parityConfigString`.
    pub fn parity_config_string(&self, builder: &mut String) {
        let s = super::noise_utils::parity_noise_octave_config_string_bytes(
            self.xo, self.yo, self.zo, &self.p,
        );
        builder.push_str(&s);
    }
}
