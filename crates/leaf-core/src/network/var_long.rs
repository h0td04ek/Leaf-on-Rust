//! Перенос `net.minecraft.network.VarLong`.

use crate::network::friendly_byte_buf::FriendlyByteBuf;

pub const MAX_VARLONG_SIZE: usize = 10;
const DATA_BITS_MASK: i64 = 127;
const CONTINUATION_BIT_MASK: i64 = 128;

fn varlong_exact_byte_lengths() -> [usize; 65] {
    let mut t = [0usize; 65];
    for i in 0..64usize {
        t[i] = ((63 - (i as i64 - 1)) as f64 / 7.0).ceil() as usize;
    }
    t[64] = 1;
    t
}

fn byte_size_table() -> &'static [usize; 65] {
    static mut TABLE: [usize; 65] = [0; 65];
    static ONCE: std::sync::Once = std::sync::Once::new();
    unsafe {
        ONCE.call_once(|| {
            TABLE = varlong_exact_byte_lengths();
        });
        &*(&raw const TABLE)
    }
}

/// `VarLong.getByteSize(long)`.
pub fn get_byte_size(data: i64) -> usize {
    let lz = data.leading_zeros() as usize;
    byte_size_table()[lz]
}

/// `VarLong.getByteSizeOld`.
pub fn get_byte_size_old(data: i64) -> usize {
    for i in 1..MAX_VARLONG_SIZE {
        if (data & (-1i64 << (i * 7))) == 0 {
            return i;
        }
    }
    MAX_VARLONG_SIZE
}

pub fn has_continuation_bit(data: u8) -> bool {
    (data as i64 & CONTINUATION_BIT_MASK) == CONTINUATION_BIT_MASK
}

/// `VarLong.read(ByteBuf)`
pub fn read(buffer: &mut FriendlyByteBuf) -> i64 {
    let mut l = 0i64;
    let mut i = 0usize;
    loop {
        let byte = buffer.read_byte() as u8;
        l |= (((byte as i64) & DATA_BITS_MASK) << (i * 7)) as i64;
        i += 1;
        if i > 10 {
            panic!("VarLong too big");
        }
        if !has_continuation_bit(byte) {
            return l;
        }
    }
}

/// `VarLong.write(ByteBuf, long)` — LEB128.
pub fn write(buffer: &mut FriendlyByteBuf, value: i64) {
    let mut v = value;
    while (v & !DATA_BITS_MASK) != 0 {
        buffer.write_byte((((v & DATA_BITS_MASK) | CONTINUATION_BIT_MASK) as u8) as i32);
        v = (v as u64 >> 7) as i64; // >>>= 7
    }
    buffer.write_byte((v as u8) as i32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::friendly_byte_buf::FriendlyByteBuf;

    #[test]
    fn byte_size_matches_old() {
        for v in [0i64, 1, 127, 128, 300, i32::MAX as i64, i64::MAX, -1, -128, i64::MIN, 1i64 << 40] {
            assert_eq!(get_byte_size(v), get_byte_size_old(v), "v={v}");
        }
        assert_eq!(get_byte_size(0), 1);
    }

    #[test]
    fn roundtrip() {
        for v in [0i64, 1, 127, 128, 300, -1, -128, i64::MAX, i64::MIN, 1i64 << 40, -(1i64 << 40)] {
            let mut buf = FriendlyByteBuf::new();
            write(&mut buf, v);
            let mut rbuf = FriendlyByteBuf::new();
            rbuf.write_bytes(&buf.copy_bytes());
            assert_eq!(read(&mut rbuf), v, "v={v}");
        }
    }
}
