//! Modified UTF-8 — формат `DataOutput.writeUTF` / `DataInput.readUTF`.
//!
//! Отличия от обычного UTF-8:
//! - U+0000 кодируется как `C0 80` (не `00`);
//! - supplementary-символы кодируются парой суррогатов, каждый как 3 байта;
//! - байты `F5..=FF` запрещены.

/// Кодирует строку в modified UTF-8. Возвращает `None`, если строка не влезает
/// в лимит `writeUTF` (u16 байт).
pub fn encode(value: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(value.len() + 8);
    for c in value.encode_utf16() {
        match c {
            0x0001..=0x007F => out.push(c as u8),
            0x0000 => out.extend_from_slice(&[0xC0, 0x80]),
            0x0080..=0x07FF => {
                out.push(0xC0 | (c >> 6) as u8);
                out.push(0x80 | (c & 0x3F) as u8);
            }
            _ => {
                out.push(0xE0 | (c >> 12) as u8);
                out.push(0x80 | ((c >> 6) & 0x3F) as u8);
                out.push(0x80 | (c & 0x3F) as u8);
            }
        }
    }
    if out.len() > u16::MAX as usize {
        return None;
    }
    Some(out)
}

/// Декодирует modified UTF-8. Ошибка — при некорректной последовательности
/// или если результат не является валидной строкой.
pub fn decode(bytes: &[u8]) -> Result<String, Utf8Error> {
    let mut units: Vec<u16> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b & 0x80 == 0 {
            if b == 0 {
                return Err(Utf8Error::Malformed);
            }
            units.push(b as u16);
            i += 1;
        } else if b & 0xE0 == 0xC0 {
            if i + 1 >= bytes.len() {
                return Err(Utf8Error::Malformed);
            }
            let b2 = bytes[i + 1];
            if b2 & 0xC0 != 0x80 {
                return Err(Utf8Error::Malformed);
            }
            let cp = (((b & 0x1F) as u32) << 6) | (b2 & 0x3F) as u32;
            if cp < 0x80 {
                // overlong или закодированный U+0000
                if cp == 0 && b == 0xC0 && b2 == 0x80 {
                    units.push(0);
                } else {
                    return Err(Utf8Error::Malformed);
                }
            } else {
                units.push(cp as u16);
            }
            i += 2;
        } else if b & 0xF0 == 0xE0 {
            if i + 2 >= bytes.len() {
                return Err(Utf8Error::Malformed);
            }
            let (b2, b3) = (bytes[i + 1], bytes[i + 2]);
            if b2 & 0xC0 != 0x80 || b3 & 0xC0 != 0x80 {
                return Err(Utf8Error::Malformed);
            }
            let cp = (((b & 0x0F) as u32) << 12)
                | (((b2 & 0x3F) as u32) << 6)
                | (b3 & 0x3F) as u32;
            if !(0x800..=0xFFFF).contains(&cp) {
                return Err(Utf8Error::Malformed);
            }
            units.push(cp as u16);
            i += 3;
        } else {
            return Err(Utf8Error::Malformed);
        }
    }
    String::from_utf16(&units).map_err(|_| Utf8Error::Malformed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Utf8Error {
    Malformed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        for s in ["", "hello", "привет", "\u{0}", "\u{1F600}x", "Ā😀"] {
            let enc = encode(s).unwrap();
            assert_eq!(decode(&enc).unwrap(), s);
        }
    }

    #[test]
    fn null_encoding() {
        assert_eq!(encode("\0").unwrap(), vec![0xC0, 0x80]);
        assert_eq!(decode(&[0xC0, 0x80]).unwrap(), "\0");
        assert!(decode(&[0x00]).is_err());
    }
}
