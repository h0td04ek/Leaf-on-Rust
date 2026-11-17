//! Перенос `net.minecraft.network.LpVec3` — компактная квантизация Vec3.

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::var_int;
use crate::util::mth;
use crate::world::phys::Vec3;

const DATA_BITS: i64 = 15;
const DATA_BITS_MASK: i64 = 32767;
const MAX_QUANTIZED_VALUE: f64 = 32766.0;
const SCALE_BITS: i64 = 2;
const SCALE_BITS_MASK: i64 = 3;
const CONTINUATION_FLAG: i64 = 4;
const X_OFFSET: i64 = 3;
const Y_OFFSET: i64 = 18;
const Z_OFFSET: i64 = 33;
pub const ABS_MAX_VALUE: f64 = 1.7179869183e10;
pub const ABS_MIN_VALUE: f64 = 3.051944088384301e-5;

pub fn has_continuation_bit(bytes: i32) -> bool {
    (bytes & CONTINUATION_FLAG as i32) == CONTINUATION_FLAG as i32
}

/// `LpVec3.read`
pub fn read(buffer: &mut FriendlyByteBuf) -> Vec3 {
    let unsigned_byte = buffer.read_unsigned_byte();
    if unsigned_byte == 0 {
        Vec3::ZERO
    } else {
        let unsigned_byte1 = buffer.read_unsigned_byte() as i64;
        let unsigned_int = buffer.read_unsigned_int() as i64; // u32
        let mut l = (unsigned_int << 16) | (unsigned_byte1 << 8) | (unsigned_byte as i64);
        let mut l1 = (unsigned_byte as i64) & SCALE_BITS_MASK;
        if has_continuation_bit(unsigned_byte) {
            l1 |= ((var_int::read(buffer) as i64) & 0xFFFFFFFF) << 2;
        }
        Vec3::new(
            unpack(l >> X_OFFSET) * l1 as f64,
            unpack(l >> Y_OFFSET) * l1 as f64,
            unpack(l >> Z_OFFSET) * l1 as f64,
        )
    }
}

/// `LpVec3.write`
pub fn write(buffer: &mut FriendlyByteBuf, vector: &Vec3) {
    let d = sanitize(vector.x);
    let d1 = sanitize(vector.y);
    let d2 = sanitize(vector.z);
    let max = mth::abs_max_f64(d, mth::abs_max_f64(d1, d2));
    if max < ABS_MIN_VALUE {
        buffer.write_byte(0);
    } else {
        let l = mth::ceil_long(max);
        let flag = (l & 3) != l;
        let l1 = if flag { (l & 3) | CONTINUATION_FLAG } else { l };
        let l2 = pack(d / l as f64) << X_OFFSET;
        let l3 = pack(d1 / l as f64) << Y_OFFSET;
        let l4 = pack(d2 / l as f64) << Z_OFFSET;
        let l5 = l1 | l2 | l3 | l4;
        buffer.write_byte(l5 as i32);
        buffer.write_byte((l5 >> 8) as i32);
        buffer.write_int((l5 >> 16) as i32);
        if flag {
            var_int::write(buffer, (l >> 2) as i32);
        }
    }
}

fn sanitize(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        mth::clamp_f64(value, -ABS_MAX_VALUE, ABS_MAX_VALUE)
    }
}

fn pack(unpacked: f64) -> i64 {
    ((unpacked * 0.5 + 0.5) * MAX_QUANTIZED_VALUE).round() as i64
}

fn unpack(packed: i64) -> f64 {
    ((packed & DATA_BITS_MASK).min(MAX_QUANTIZED_VALUE as i64)) as f64 * 2.0 / MAX_QUANTIZED_VALUE - 1.0
}
