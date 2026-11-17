//! Перенос `net.minecraft.core.QuartPos`.

pub const BITS: i32 = 2;
pub const SIZE: i32 = 4;
pub const MASK: i32 = 3;
const SECTION_TO_QUARTS_BITS: i32 = 2;

pub fn from_block(value: i32) -> i32 {
    value >> BITS
}

pub fn quart_local(value: i32) -> i32 {
    value & MASK
}

pub fn to_block(value: i32) -> i32 {
    value << BITS
}

pub const fn from_section(value: i32) -> i32 {
    value << SECTION_TO_QUARTS_BITS
}

pub const fn to_section(value: i32) -> i32 {
    value >> SECTION_TO_QUARTS_BITS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_quart_roundtrip() {
        // to_block(from_block(v)) — нижняя граница блока в block-координатах
        for v in [0, 1, 3, 4, 15, 16, -1, -4, 1023, -1024] {
            assert_eq!(to_block(from_block(v)), v & !3, "v={v}");
        }
        // from_block(to_block(q)) == q для quart-координат
        for q in [-256, -1, 0, 1, 255, 256] {
            assert_eq!(from_block(to_block(q)), q, "q={q}");
        }
    }

    #[test]
    fn section_quart() {
        assert_eq!(from_section(1), 4);
        assert_eq!(to_section(7), 1);
        assert_eq!(quart_local(-1), 3);
    }
}
