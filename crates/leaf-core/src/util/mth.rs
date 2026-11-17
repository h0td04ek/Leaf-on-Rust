//! Полный порт `net.minecraft.util.Mth` (Leaf 1.21.11).
//!
//! Отступления от Java-оригинала:
//! - `RandomSource` — `&mut dyn crate::random::RandomSource`;
//! - `Vec3`/`Aabb` — `crate::world::phys`;
//! - `UUID` — кортеж `(msb, lsb)`;
//! - `IntStream outFromOrigin` — `Vec<i32>` (поток в Java ленивый);
//! - `Fraction` (commons-math) — `mul_and_truncate(num, den, factor)`;
//! - `RANDOM` (`createThreadSafe`) — thread_local LegacyRandomSource:
//!   в Java это synchronized singleton, здесь per-thread экземпляр.

use crate::core::vec3i::Vec3i;
use crate::joml::{Quaternionf, Vector3f};
use crate::random::{self, RandomSource};
use crate::util::argb;
use crate::world::phys::{Aabb, Vec3};
use std::cell::RefCell;
use std::sync::OnceLock;

pub const PI: f32 = std::f32::consts::PI;
pub const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;
pub const TWO_PI: f32 = std::f32::consts::TAU;
pub const DEG_TO_RAD: f32 = std::f32::consts::PI / 180.0;
pub const RAD_TO_DEG: f32 = 180.0 / std::f32::consts::PI;
pub const EPSILON: f32 = 1.0e-5;
pub const SQRT_OF_TWO: f32 = std::f32::consts::SQRT_2;
pub const Y_AXIS_F: Vector3f = Vector3f::new(0.0, 1.0, 0.0);
pub const X_AXIS_F: Vector3f = Vector3f::new(1.0, 0.0, 0.0);
pub const Z_AXIS_F: Vector3f = Vector3f::new(0.0, 0.0, 1.0);

const SIN_MASK: i64 = 65535;
const COS_OFFSET: f64 = 16384.0;
const SIN_SCALE: f64 = 10430.378350470453;

fn sin_table() -> &'static [f32; 65536] {
    // SAFETY-free: храним Vec и приводим через slice — длина фиксирована.
    static CELL: OnceLock<Box<[f32; 65536]>> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut arr = Box::new([0.0f32; 65536]);
        for (i, v) in arr.iter_mut().enumerate() {
            *v = (i as f64 / SIN_SCALE).sin() as f32;
        }
        arr
    })
}

static ASIN_TAB: OnceLock<Box<[f64; 257]>> = OnceLock::new();
static COS_TAB: OnceLock<Box<[f64; 257]>> = OnceLock::new();

fn asin_cos_tabs() -> (&'static [f64; 257], &'static [f64; 257]) {
    let asin = ASIN_TAB.get_or_init(|| {
        let mut arr = Box::new([0.0f64; 257]);
        for (i, v) in arr.iter_mut().enumerate() {
            let d = i as f64 / 256.0;
            *v = d.asin();
        }
        arr
    });
    let cos = COS_TAB.get_or_init(|| {
        let mut arr = Box::new([0.0f64; 257]);
        for (i, v) in arr.iter_mut().enumerate() {
            let d = i as f64 / 256.0;
            *v = d.asin().cos();
        }
        arr
    });
    (asin, cos)
}

thread_local! {
    static RANDOM: RefCell<Box<dyn RandomSource>> = RefCell::new(random::create_unique());
}

const MULTIPLY_DE_BRUIJN_BIT_POSITION: [i32; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7,
    26, 12, 18, 6, 11, 5, 10, 9,
];
const ONE_SIXTH: f64 = 0.16666666666666666;
const FRAC_BIAS: f64 = f64::from_bits(4805340802404319232);

// ---------------------------------------------------------------- sin/cos

/// `Mth.sin(double)` — табличный синус (квантование 2^16).
pub fn sin(value: f64) -> f32 {
    let idx = ((value * SIN_SCALE) as i64) & SIN_MASK;
    sin_table()[idx as usize]
}

/// `Mth.cos(double)`
pub fn cos(value: f64) -> f32 {
    let idx = ((value * SIN_SCALE + COS_OFFSET) as i64) & SIN_MASK;
    sin_table()[idx as usize]
}

// ---------------------------------------------------------------- sqrt/abs

pub fn sqrt_f32(value: f32) -> f32 {
    value.sqrt()
}

pub fn sqrt_f64(value: f64) -> f64 {
    value.sqrt()
}

pub fn abs_f32(value: f32) -> f32 {
    value.abs()
}

pub fn abs_i32(value: i32) -> i32 {
    value.abs()
}

pub fn abs_f64(value: f64) -> f64 {
    value.abs()
}

// ---------------------------------------------------------------- floor/ceil

/// Java `(int) Math.floor(v)` — для f32/f64 входа.
pub fn floor_f32(value: f32) -> i32 {
    value.floor() as i32
}

/// `Mth.floor(double)` — `(int) Math.floor`.
pub fn floor(value: f64) -> i32 {
    value.floor() as i32
}

/// `Mth.lfloor(double)` — `(long) Math.floor`.
pub fn lfloor(value: f64) -> i64 {
    value.floor() as i64
}

/// `Mth.ceil(float)`
pub fn ceil_f32(value: f32) -> i32 {
    value.ceil() as i32
}

/// `Mth.ceil(double)`
pub fn ceil(value: f64) -> i32 {
    value.ceil() as i32
}

/// `Mth.ceilLong(double)`
pub fn ceil_long(value: f64) -> i64 {
    let l = value as i64;
    if value > l as f64 {
        l + 1
    } else {
        l
    }
}

// ---------------------------------------------------------------- clamp

pub fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.min(max).max(min)
}

pub fn clamp_i64(value: i64, min: i64, max: i64) -> i64 {
    value.min(max).max(min)
}

pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else {
        value.min(max)
    }
}

pub fn clamp_f64(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else {
        value.min(max)
    }
}

// ---------------------------------------------------------------- lerp и родственные

pub fn clamped_lerp_f64(start: f64, end: f64, delta: f64) -> f64 {
    if start < 0.0 {
        end
    } else if start > 1.0 {
        delta
    } else {
        lerp_f64(start, end, delta)
    }
}

pub fn clamped_lerp_f32(start: f32, end: f32, delta: f32) -> f32 {
    if start < 0.0 {
        end
    } else if start > 1.0 {
        delta
    } else {
        lerp_f32(start, end, delta)
    }
}

pub fn abs_max_i32(i: i32, i1: i32) -> i32 {
    i.abs().max(i1.abs())
}

pub fn abs_max_f32(f: f32, f1: f32) -> f32 {
    f.abs().max(f1.abs())
}

pub fn abs_max_f64(x: f64, y: f64) -> f64 {
    x.abs().max(y.abs())
}

pub fn chessboard_distance(i: i32, i1: i32, i2: i32, i3: i32) -> i32 {
    abs_max_i32(i2 - i, i3 - i1)
}

/// `Math.floorDiv` — округление к -inf.
pub fn floor_div(a: i32, b: i32) -> i32 {
    let q = a / b;
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

pub fn next_int(random: &mut dyn RandomSource, minimum: i32, maximum: i32) -> i32 {
    if minimum >= maximum {
        minimum
    } else {
        random.next_int_bound(maximum - minimum + 1) + minimum
    }
}

pub fn next_float(random: &mut dyn RandomSource, minimum: f32, maximum: f32) -> f32 {
    if minimum >= maximum {
        minimum
    } else {
        random.next_float() * (maximum - minimum) + minimum
    }
}

pub fn next_double(random: &mut dyn RandomSource, minimum: f64, maximum: f64) -> f64 {
    if minimum >= maximum {
        minimum
    } else {
        random.next_double() * (maximum - minimum) + minimum
    }
}

pub fn equal_f32(x: f32, y: f32) -> bool {
    (y - x).abs() < 1.0e-5
}

pub fn equal_f64(x: f64, y: f64) -> bool {
    (y - x).abs() < 1.0e-5
}

/// `Math.floorMod`
pub fn positive_modulo(x: i32, y: i32) -> i32 {
    x.rem_euclid(y)
}

pub fn positive_modulo_f32(numerator: f32, denominator: f32) -> f32 {
    (numerator % denominator + denominator) % denominator
}

pub fn positive_modulo_f64(numerator: f64, denominator: f64) -> f64 {
    (numerator % denominator + denominator) % denominator
}

pub fn positive_modulo_pos_den_f32(numerator: f32, denominator: f32) -> f32 {
    let modulo = numerator % denominator;
    if modulo < 0.0 { modulo + denominator } else { modulo }
}

pub fn positive_modulo_pos_den_f64(numerator: f64, denominator: f64) -> f64 {
    let modulo = numerator % denominator;
    if modulo < 0.0 { modulo + denominator } else { modulo }
}

pub fn is_multiple_of(number: i32, multiple: i32) -> bool {
    number % multiple == 0
}

pub fn pack_degrees(degrees: f32) -> i8 {
    floor_f32(degrees * 256.0 / 360.0) as i8
}

pub fn unpack_degrees(degrees: i8) -> f32 {
    ((degrees as i32) * 360) as f32 / 256.0
}

pub fn wrap_degrees_int(angle: i32) -> i32 {
    let mut i = angle % 360;
    if i >= 180 {
        i -= 360;
    }
    if i < -180 {
        i += 360;
    }
    i
}

pub fn wrap_degrees_long(angle: i64) -> f32 {
    let mut f = (angle % 360) as f32;
    if f >= 180.0 {
        f -= 360.0;
    }
    if f < -180.0 {
        f += 360.0;
    }
    f
}

pub fn wrap_degrees_f32(value: f32) -> f32 {
    let mut f = value % 360.0;
    if f >= 180.0 {
        f -= 360.0;
    }
    if f < -180.0 {
        f += 360.0;
    }
    f
}

pub fn wrap_degrees_f64(value: f64) -> f64 {
    let mut d = value % 360.0;
    if d >= 180.0 {
        d -= 360.0;
    }
    if d < -180.0 {
        d += 360.0;
    }
    d
}

pub fn degrees_difference(start: f32, end: f32) -> f32 {
    wrap_degrees_f32(end - start)
}

pub fn degrees_difference_abs(start: f32, end: f32) -> f32 {
    abs_f32(degrees_difference(start, end))
}

pub fn rotate_if_necessary(rotation_to_adjust: f32, actual_rotation: f32, max_difference: f32) -> f32 {
    let f = degrees_difference(rotation_to_adjust, actual_rotation);
    let f1 = clamp_f32(f, -max_difference, max_difference);
    actual_rotation - f1
}

pub fn approach(value: f32, limit: f32, step_size: f32) -> f32 {
    let step_size = abs_f32(step_size);
    if value < limit {
        clamp_f32(value + step_size, value, limit)
    } else {
        clamp_f32(value - step_size, limit, value)
    }
}

pub fn approach_degrees(angle: f32, limit: f32, step_size: f32) -> f32 {
    let f = degrees_difference(angle, limit);
    approach(angle, angle + f, step_size)
}

/// `NumberUtils.toInt(value, defaultValue)` — упрощённый порт
/// (decimal/hex/octal, `_`-разделители), TODO: полная семантика commons-lang3.
pub fn get_int(value: &str, default_value: i32) -> i32 {
    let s = value.trim().replace('_', "");
    if s.is_empty() {
        return default_value;
    }
    let parsed = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i32::from_str_radix(hex, 16)
    } else if s.len() > 1 && s.starts_with('0') && s[1..].chars().all(|c| c.is_ascii_digit()) {
        i32::from_str_radix(&s[1..], 8)
    } else {
        s.parse::<i32>()
    };
    parsed.unwrap_or(default_value)
}

pub fn smallest_encompassing_power_of_two(value: i32) -> i32 {
    let mut i = value.wrapping_sub(1);
    i |= i >> 1;
    i |= i >> 2;
    i |= i >> 4;
    i |= i >> 8;
    i |= i >> 16;
    i.wrapping_add(1)
}

pub fn smallest_square_side(item_count: i32) -> i32 {
    assert!(item_count >= 0, "itemCount must be greater than or equal to zero");
    ceil_f32(sqrt_f32(item_count as f32))
}

pub fn is_power_of_two(value: i32) -> bool {
    value != 0 && (value & (value - 1)) == 0
}

pub fn ceillog2(value: i32) -> i32 {
    let value = if is_power_of_two(value) {
        value
    } else {
        smallest_encompassing_power_of_two(value)
    };
    let idx = (((value as i64).wrapping_mul(125613361) >> 27) & 31) as usize;
    MULTIPLY_DE_BRUIJN_BIT_POSITION[idx]
}

pub fn log2(value: i32) -> i32 {
    ceillog2(value) - if is_power_of_two(value) { 0 } else { 1 }
}

pub fn frac_f32(number: f32) -> f32 {
    number - floor_f32(number) as f32
}

pub fn frac_f64(number: f64) -> f64 {
    number - lfloor(number) as f64
}

// ---------------------------------------------------------------- seed/uuid

/// `Mth.getSeed(int x, int y, int z)` — deprecated-вариант Java.
/// В Java `x * 3129871` — 32-битное умножение с переполнением,
/// затем расширение до long для XOR.
pub fn get_seed(x: i32, y: i32, z: i32) -> i64 {
    let mut l = (x.wrapping_mul(3129871) as i64)
        ^ ((z as i64).wrapping_mul(116129781))
        ^ (y as i64);
    l = l
        .wrapping_mul(l)
        .wrapping_mul(42317861)
        .wrapping_add(l.wrapping_mul(11));
    l >> 16
}

/// `Mth.getSeed(Vec3i)`
pub fn get_seed_vec3i(pos: &Vec3i) -> i64 {
    get_seed(pos.x, pos.y, pos.z)
}

/// `Mth.createInsecureUUID(RandomSource)` — возвращает (mostSigBits, leastSigBits).
pub fn create_insecure_uuid(random: &mut dyn RandomSource) -> (i64, i64) {
    let l = (random.next_long() & -61441) | 16384;
    let l1 = (random.next_long() & 4611686018427387903) | i64::MIN;
    (l, l1)
}

/// `Mth.createInsecureUUID()` — thread-safe RNG.
pub fn create_insecure_uuid_default() -> (i64, i64) {
    RANDOM.with(|r| create_insecure_uuid(&mut **r.borrow_mut()))
}

// ---------------------------------------------------------------- интерполяция

pub fn inverse_lerp_f64(delta: f64, start: f64, end: f64) -> f64 {
    (delta - start) / (end - start)
}

pub fn inverse_lerp_f32(delta: f32, start: f32, end: f32) -> f32 {
    (delta - start) / (end - start)
}

/// `Mth.rayIntersectsAABB`
pub fn ray_intersects_aabb(start: &Vec3, end: &Vec3, bounding_box: &Aabb) -> bool {
    let d = (bounding_box.min_x + bounding_box.max_x) * 0.5;
    let d1 = (bounding_box.max_x - bounding_box.min_x) * 0.5;
    let d2 = start.x - d;
    if d2.abs() > d1 && d2 * end.x >= 0.0 {
        return false;
    }
    let d3 = (bounding_box.min_y + bounding_box.max_y) * 0.5;
    let d4 = (bounding_box.max_y - bounding_box.min_y) * 0.5;
    let d5 = start.y - d3;
    if d5.abs() > d4 && d5 * end.y >= 0.0 {
        return false;
    }
    let d6 = (bounding_box.min_z + bounding_box.max_z) * 0.5;
    let d7 = (bounding_box.max_z - bounding_box.min_z) * 0.5;
    let d8 = start.z - d6;
    if d8.abs() > d7 && d8 * end.z >= 0.0 {
        return false;
    }
    let abs = end.x.abs();
    let abs1 = end.y.abs();
    let abs2 = end.z.abs();
    let mut d9 = end.y * d8 - end.z * d5;
    if d9.abs() > d4 * abs2 + d7 * abs1 {
        return false;
    }
    d9 = end.z * d2 - end.x * d8;
    if d9.abs() > d1 * abs2 + d7 * abs {
        return false;
    }
    d9 = end.x * d5 - end.y * d2;
    d9.abs() < d1 * abs1 + d4 * abs
}

// ---------------------------------------------------------------- atan2

/// `Mth.atan2` — быстрый atan2 через таблицы asin/cos.
pub fn atan2(y: f64, x: f64) -> f64 {
    let d = x * x + y * y;
    if d.is_nan() {
        return f64::NAN;
    }
    let mut y = y;
    let mut x = x;
    let flag = y < 0.0;
    if flag {
        y = -y;
    }
    let flag1 = x < 0.0;
    if flag1 {
        x = -x;
    }
    let flag2 = y > x;
    if flag2 {
        std::mem::swap(&mut x, &mut y);
    }
    let d1 = fast_inv_sqrt(d);
    x *= d1;
    y *= d1;
    let d2 = FRAC_BIAS + y;
    let i = d2.to_bits() as i32; // (int) doubleToRawLongBits — младшие 32 бита
    let (asin_tab, cos_tab) = asin_cos_tabs();
    let d3 = asin_tab[i as usize];
    let d4 = cos_tab[i as usize];
    let d5 = d2 - FRAC_BIAS;
    let d6 = y * d4 - x * d5;
    let d7 = (6.0 + d6 * d6) * d6 * ONE_SIXTH;
    let mut d8 = d3 + d7;
    if flag2 {
        d8 = (std::f64::consts::FRAC_PI_2) - d8;
    }
    if flag1 {
        d8 = std::f64::consts::PI - d8;
    }
    if flag {
        d8 = -d8;
    }
    d8
}

/// `org.joml.Math.invsqrt`
pub fn inv_sqrt_f32(number: f32) -> f32 {
    1.0 / number.sqrt()
}

pub fn inv_sqrt_f64(number: f64) -> f64 {
    1.0 / number.sqrt()
}

/// `Mth.fastInvSqrt` (deprecated)
pub fn fast_inv_sqrt(number: f64) -> f64 {
    let d = 0.5 * number;
    let l = number.to_bits() as i64;
    let l = 6910469410427058090i64.wrapping_sub(l >> 1);
    let number = f64::from_bits(l as u64);
    number * (1.5 - d * number * number)
}

pub fn fast_inv_cube_root(number: f32) -> f32 {
    let mut i = number.to_bits() as i32;
    i = 1419967116 - i / 3;
    let f = f32::from_bits(i as u32);
    let f = 0.6666667 * f + 1.0 / (3.0 * f * f * number);
    0.6666667 * f + 1.0 / (3.0 * f * f * number)
}

// ---------------------------------------------------------------- цвет

/// `Mth.hsvToRgb` — без альфы (alpha=0).
pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> i32 {
    hsv_to_argb(hue, saturation, value, 0)
}

/// `Mth.hsvToArgb`
pub fn hsv_to_argb(hue: f32, saturation: f32, value: f32, alpha: i32) -> i32 {
    let i = ((hue * 6.0) as i32) % 6;
    let f = hue * 6.0 - i as f32;
    let f1 = value * (1.0 - saturation);
    let f2 = value * (1.0 - f * saturation);
    let f3 = value * (1.0 - (1.0 - f) * saturation);
    let (f4, f5, f6) = match i {
        0 => (value, f3, f1),
        1 => (f2, value, f1),
        2 => (f1, value, f3),
        3 => (f1, f2, value),
        4 => (f3, f1, value),
        5 => (value, f1, f2),
        _ => panic!(
            "Something went wrong when converting from HSV to RGB. Input was {}, {}, {}",
            hue, saturation, value
        ),
    };
    argb::color4(
        alpha,
        clamp_i32((f4 * 255.0) as i32, 0, 255),
        clamp_i32((f5 * 255.0) as i32, 0, 255),
        clamp_i32((f6 * 255.0) as i32, 0, 255),
    )
}

// ---------------------------------------------------------------- поиск/дискретное

/// `Mth.murmurHash3Mixer`
pub fn murmur_hash3_mixer(input: i32) -> i32 {
    let mut input = input;
    input ^= input >> 16;
    input = input.wrapping_mul(-2048144789);
    input ^= input >> 13;
    input = input.wrapping_mul(-1028477387);
    input ^ (input >> 16)
}

/// `Mth.binarySearch` — бинпоиск по IntPredicate.
pub fn binary_search(min: i32, max: i32, is_target_before_or_at: &mut dyn FnMut(i32) -> bool) -> i32 {
    let mut min = min;
    let mut i = max - min;
    while i > 0 {
        let i1 = i / 2;
        let i2 = min + i1;
        if is_target_before_or_at(i2) {
            i = i1;
        } else {
            min = i2 + 1;
            i -= i1 + 1;
        }
    }
    min
}

pub fn lerp_int(delta: f32, start: i32, end: i32) -> i32 {
    start + floor_f32(delta * (end - start) as f32)
}

pub fn lerp_discrete(delta: f32, start: i32, end: i32) -> i32 {
    let i = end - start;
    start + floor_f32(delta * (i - 1) as f32) + if delta > 0.0 { 1 } else { 0 }
}

pub fn lerp_f32(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

pub fn lerp_f64(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

/// `Mth.lerp(double, Vec3, Vec3)`
pub fn lerp_vec(delta: f64, start: &Vec3, end: &Vec3) -> Vec3 {
    Vec3::new(
        lerp_f64(delta, start.x, end.x),
        lerp_f64(delta, start.y, end.y),
        lerp_f64(delta, start.z, end.z),
    )
}

pub fn lerp2_f64(d1: f64, d2: f64, s1: f64, e1: f64, s2: f64, e2: f64) -> f64 {
    lerp_f64(d2, lerp_f64(d1, s1, e1), lerp_f64(d1, s2, e2))
}

pub fn lerp3_f64(
    d1: f64,
    d2: f64,
    d3: f64,
    s1: f64,
    e1: f64,
    s2: f64,
    e2: f64,
    s3: f64,
    e3: f64,
    s4: f64,
    e4: f64,
) -> f64 {
    lerp_f64(d3, lerp2_f64(d1, d2, s1, e1, s2, e2), lerp2_f64(d1, d2, s3, e3, s4, e4))
}

pub fn catmullrom(delta: f32, cp1: f32, cp2: f32, cp3: f32, cp4: f32) -> f32 {
    0.5 * (
        2.0 * cp2
            + (cp3 - cp1) * delta
            + (2.0 * cp1 - 5.0 * cp2 + 4.0 * cp3 - cp4) * delta * delta
            + (3.0 * cp2 - cp1 - 3.0 * cp3 + cp4) * delta * delta * delta
    )
}

pub fn smoothstep(input: f64) -> f64 {
    input * input * input * (input * (input * 6.0 - 15.0) + 10.0)
}

pub fn smoothstep_derivative(input: f64) -> f64 {
    30.0 * input * input * (input - 1.0) * (input - 1.0)
}

pub fn sign(x: f64) -> i32 {
    if x == 0.0 {
        0
    } else if x > 0.0 {
        1
    } else {
        -1
    }
}

pub fn rot_lerp_f32(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * wrap_degrees_f32(end - start)
}

pub fn rot_lerp_f64(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * wrap_degrees_f64(end - start)
}

pub fn rot_lerp_rad(delta: f32, start: f32, end: f32) -> f32 {
    let mut f = end - start;
    let pi = std::f32::consts::PI;
    while f < -pi {
        f += pi * 2.0;
    }
    while f >= pi {
        f -= pi * 2.0;
    }
    start + delta * f
}

pub fn triangle_wave(input: f32, period: f32) -> f32 {
    ((input % period - period * 0.5).abs() - period * 0.25) / (period * 0.25)
}

pub fn square_f32(value: f32) -> f32 {
    value * value
}

pub fn square_f64(value: f64) -> f64 {
    value * value
}

pub fn cube_f32(value: f32) -> f32 {
    value * value * value
}

pub fn square_i32(value: i32) -> i32 {
    value * value
}

pub fn square_i64(value: i64) -> i64 {
    value * value
}

pub fn clamped_map_f64(input: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    clamped_lerp_f64(inverse_lerp_f64(input, in_min, in_max), out_min, out_max)
}

pub fn clamped_map_f32(input: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    clamped_lerp_f32(inverse_lerp_f32(input, in_min, in_max), out_min, out_max)
}

pub fn map_f64(input: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    lerp_f64(inverse_lerp_f64(input, in_min, in_max), out_min, out_max)
}

pub fn map_f32(input: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    lerp_f32(inverse_lerp_f32(input, in_min, in_max), out_min, out_max)
}

/// `Mth.wobble` — `RandomSource.create` = LegacyRandomSource (FastRNG off).
pub fn wobble(input: f64) -> f64 {
    let seed = floor(input * 3000.0) as i64;
    let mut rng = random::create(seed);
    input + (2.0 * rng.next_double() - 1.0) * 1.0e-7 / 2.0
}

pub fn round_toward(value: i32, factor: i32) -> i32 {
    positive_ceil_div(value, factor) * factor
}

pub fn positive_ceil_div(x: i32, y: i32) -> i32 {
    -floor_div(-x, y)
}

pub fn random_between_inclusive(random: &mut dyn RandomSource, min_inclusive: i32, max_inclusive: i32) -> i32 {
    random.next_int_bound(max_inclusive - min_inclusive + 1) + min_inclusive
}

pub fn random_between_f32(random: &mut dyn RandomSource, min_inclusive: f32, max_exclusive: f32) -> f32 {
    random.next_float() * (max_exclusive - min_inclusive) + min_inclusive
}

pub fn normal_f32(random: &mut dyn RandomSource, mean: f32, deviation: f32) -> f32 {
    mean + random.next_gaussian() as f32 * deviation
}

pub fn length_squared_2d(x_distance: f64, y_distance: f64) -> f64 {
    x_distance * x_distance + y_distance * y_distance
}

pub fn length_2d(x_distance: f64, y_distance: f64) -> f64 {
    sqrt_f64(length_squared_2d(x_distance, y_distance))
}

pub fn length_squared_2d_f32(x_distance: f32, y_distance: f32) -> f32 {
    x_distance * x_distance + y_distance * y_distance
}

pub fn length_2d_f32(x_distance: f32, y_distance: f32) -> f32 {
    sqrt_f32(length_squared_2d_f32(x_distance, y_distance))
}

pub fn length_squared_3d(x_distance: f64, y_distance: f64, z_distance: f64) -> f64 {
    x_distance * x_distance + y_distance * y_distance + z_distance * z_distance
}

pub fn length_3d(x_distance: f64, y_distance: f64, z_distance: f64) -> f64 {
    sqrt_f64(length_squared_3d(x_distance, y_distance, z_distance))
}

pub fn length_squared_3d_f32(x_distance: f32, y_distance: f32, z_distance: f32) -> f32 {
    x_distance * x_distance + y_distance * y_distance + z_distance * z_distance
}

pub fn length_3d_f32(x_distance: f32, y_distance: f32, z_distance: f32) -> f32 {
    sqrt_f32(length_squared_3d_f32(x_distance, y_distance, z_distance))
}

pub fn quantize(value: f64, factor: i32) -> i32 {
    floor(value / factor as f64) * factor
}

/// `Mth.outFromOrigin` — Java возвращает ленивый IntStream;
/// здесь собираем в Vec (диапазоны ограничены границами).
pub fn out_from_origin(input: i32, lower_bound: i32, upper_bound: i32, steps: i32) -> Vec<i32> {
    assert!(
        lower_bound <= upper_bound,
        "upperBound {} expected to be > lowerBound {}",
        upper_bound,
        lower_bound
    );
    assert!(steps >= 1, "step size expected to be >= 1, was {}", steps);
    let i = clamp_i32(input, lower_bound, upper_bound);
    let has_next = |t: i32| -> bool {
        let abs = (i - t).abs();
        i - abs >= lower_bound || i + abs <= upper_bound
    };
    let next = |t: i32| -> i32 {
        let flag = t <= i;
        let abs = (i - t).abs();
        let flag1 = i.wrapping_add(abs).wrapping_add(steps) <= upper_bound;
        if !flag || !flag1 {
            let i2 = i.wrapping_sub(abs).wrapping_sub(if flag { steps } else { 0 });
            if i2 >= lower_bound {
                return i2;
            }
        }
        i.wrapping_add(abs).wrapping_add(steps)
    };
    let mut out = Vec::new();
    let mut cur = i;
    out.push(cur);
    while has_next(cur) {
        cur = next(cur);
        out.push(cur);
    }
    out
}

pub fn out_from_origin_default_steps(input: i32, lower_bound: i32, upper_bound: i32) -> Vec<i32> {
    out_from_origin(input, lower_bound, upper_bound, 1)
}

/// `Mth.rotationAroundAxis`
pub fn rotation_around_axis<'a>(
    axis: &Vector3f,
    camera_orientation: &Quaternionf,
    output: &'a mut Quaternionf,
) -> &'a mut Quaternionf {
    let f = axis.dot(camera_orientation.x, camera_orientation.y, camera_orientation.z);
    output.set(axis.x * f, axis.y * f, axis.z * f, camera_orientation.w);
    output.normalize();
    output
}

/// `Mth.mulAndTruncate(Fraction, int)` — Fraction как (num, den).
/// В Java `num * factor` — int с переполнением.
pub fn mul_and_truncate(numerator: i32, denominator: i32, factor: i32) -> i32 {
    numerator.wrapping_mul(factor) / denominator
}
