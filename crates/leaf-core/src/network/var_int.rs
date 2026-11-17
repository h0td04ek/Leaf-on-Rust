//! Перенос `net.minecraft.network.VarInt`.
//! Оптимизированные ветки Velocity/Paper дают побайтово тот же вывод,
//! что и классический LEB128-цикл — используется он.

use crate::network::friendly_byte_buf::FriendlyByteBuf;

pub const MAX_VARINT_SIZE: usize = 5;
const DATA_BITS_MASK: i32 = 127;
const CONTINUATION_BIT_MASK: i32 = 128;

fn varint_exact_byte_lengths() -> [usize; 33] {
    // Java: ceil((31 - (i-1)) / 7), i in 0..=32; [32]=1
    let mut t = [0usize; 33];
    for i in 0..=32usize {
        t[i] = ((31 - (i as i64 - 1)) as f64 / 7.0).ceil() as usize;
    }
    t[32] = 1;
    t
}

fn byte_size_table() -> &'static [usize; 33] {
    static mut TABLE: [usize; 33] = [0; 33];
    static ONCE: std::sync::Once = std::sync::Once::new();
    unsafe {
        ONCE.call_once(|| {
            TABLE = varint_exact_byte_lengths();
        });
        &*(&raw const TABLE)
    }
}

/// `VarInt.getByteSize(int)` — Paper-таблица по numberOfLeadingZeros.
pub fn get_byte_size(data: i32) -> usize {
    let lz = data.leading_zeros() as usize;
    byte_size_table()[lz]
}

/// `VarInt.getByteSizeOld` — цикл (для верификации).
pub fn get_byte_size_old(data: i32) -> usize {
    for i in 1..MAX_VARINT_SIZE {
        if (data & (-1i32 << (i * 7))) == 0 {
            return i;
        }
    }
    MAX_VARINT_SIZE
}

pub fn has_continuation_bit(data: u8) -> bool {
    (data as i32 & CONTINUATION_BIT_MASK) == CONTINUATION_BIT_MASK
}

/// `VarInt.read(ByteBuf)`
pub fn read(buffer: &mut FriendlyByteBuf) -> i32 {
    let mut i = 0i32;
    let mut i1 = 0usize;
    loop {
        let byte = buffer.read_byte() as u8;
        i |= ((byte as i32) & DATA_BITS_MASK) << (i1 * 7);
        i1 += 1;
        if i1 > 5 {
            panic!("VarInt too big");
        }
        if !has_continuation_bit(byte) {
            return i;
        }
    }
}

/// `VarInt.write(ByteBuf, int)` — LEB128 (идентичен оптимизированной версии).
pub fn write(buffer: &mut FriendlyByteBuf, value: i32) {
    let mut v = value;
    while (v & !DATA_BITS_MASK) != 0 {
        buffer.write_byte(((v & DATA_BITS_MASK) | CONTINUATION_BIT_MASK) as u8 as i32);
        v = (v as u32 >> 7) as i32; // >>>= 7
    }
    buffer.write_byte(v as u8 as i32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::friendly_byte_buf::FriendlyByteBuf;

    #[test]
    fn byte_size_matches_old() {
        for v in [0, 1, 127, 128, 255, 256, 16383, 16384, i32::MAX, -1, -2, i32::MIN] {
            assert_eq!(get_byte_size(v), get_byte_size_old(v), "v={v}");
        }
        assert_eq!(get_byte_size(0), 1);
    }

    #[test]
    fn roundtrip() {
        for v in [0i32, 1, 127, 128, 300, 16384, -1, -128, i32::MAX, i32::MIN, 2097151, -2097152] {
            let mut buf = FriendlyByteBuf::new();
            write(&mut buf, v);
            let mut rbuf = FriendlyByteBuf::new();
            rbuf.write_bytes(&buf.copy_bytes());
            assert_eq!(read(&mut rbuf), v, "v={v}");
        }
    }

    #[test]
    fn known_encoding() {
        // 300 -> AC 02
        let mut buf = FriendlyByteBuf::new();
        write(&mut buf, 300);
        assert_eq!(buf.copy_bytes(), vec![0xAC, 0x02]);
    }
}
