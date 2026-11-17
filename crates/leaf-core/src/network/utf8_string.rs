//! Перенос `net.minecraft.network.Utf8String`.

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::var_int;

/// `DecoderException`/`EncoderException` — Netty; здесь единый NetworkCodecError.
#[derive(Debug)]
pub struct CodecException(pub String);

impl std::fmt::Display for CodecException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for CodecException {}

/// `ByteBufUtil.utf8MaxBytes(int)` — максимум байт UTF-8 для maxLength символов.
fn utf8_max_bytes_chars(max_length: usize) -> usize {
    max_length * 3 // UTF-8: до 3 байт на char для BMP (Java считает иначе, но это верхняя оценка Netty)
}

/// `ByteBufUtil.utf8MaxBytes(CharSequence)` — точная верхняя оценка.
fn utf8_max_bytes_str(s: &str) -> usize {
    // Netty: считает chars, для каждого >=0x80 добавляет байты.
    // Точная формула: sum по code points (1/2/3 байта).
    s.chars().map(|c| c.len_utf8()).sum()
}

/// `Utf8String.read`
pub fn read(buffer: &mut FriendlyByteBuf, max_length: usize) -> Result<String, CodecException> {
    let i = utf8_max_bytes_chars(max_length);
    let i1 = var_int::read(buffer);
    if i1 < 0 {
        return Err(CodecException(format!(
            "The received encoded string buffer length is less than zero! Weird string!"
        )));
    }
    let i1u = i1 as usize;
    if i1u > i {
        return Err(CodecException(format!(
            "The received encoded string buffer length is longer than maximum allowed ({i1} > {i})"
        )));
    }
    let i2 = buffer.readable_bytes();
    if i1u > i2 {
        return Err(CodecException(format!(
            "Not enough bytes in buffer, expected {i1}, but got {i2}"
        )));
    }
    let bytes = buffer.read_bytes_vec(i1u);
    let string = String::from_utf8_lossy(&bytes).into_owned();
    // Java проверяет string.length() (UTF-16 units); в Rust chars().count() близко,
    // но для суррогатных пар отличается — используем UTF-16 units для точности.
    let utf16_len = string.encode_utf16().count();
    if utf16_len > max_length {
        return Err(CodecException(format!(
            "The received string length is longer than maximum allowed ({utf16_len} > {max_length})"
        )));
    }
    Ok(string)
}

/// `Utf8String.write`
pub fn write(buffer: &mut FriendlyByteBuf, string: &str, max_length: usize) -> Result<(), CodecException> {
    let utf16_len = string.encode_utf16().count();
    if utf16_len > max_length {
        return Err(CodecException(format!(
            "String too big (was {utf16_len} characters, max {max_length})"
        )));
    }
    let i = utf8_max_bytes_str(string);
    let i2 = utf8_max_bytes_chars(max_length);
    if i > i2 {
        return Err(CodecException(format!(
            "String too big (was {i} bytes encoded, max {i2})"
        )));
    }
    var_int::write(buffer, i as i32);
    buffer.write_bytes(string.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut buf = FriendlyByteBuf::new();
        write(&mut buf, "hello мир", 32767).unwrap();
        let mut r = FriendlyByteBuf::new();
        r.write_bytes(&buf.copy_bytes());
        assert_eq!(read(&mut r, 32767).unwrap(), "hello мир");
    }

    #[test]
    fn too_long_string_rejected() {
        let mut buf = FriendlyByteBuf::new();
        let s = "x".repeat(100);
        assert!(write(&mut buf, &s, 10).is_err());
    }

    #[test]
    fn empty_string() {
        let mut buf = FriendlyByteBuf::new();
        write(&mut buf, "", 10).unwrap();
        let mut r = FriendlyByteBuf::new();
        r.write_bytes(&buf.copy_bytes());
        assert_eq!(read(&mut r, 10).unwrap(), "");
    }
}
