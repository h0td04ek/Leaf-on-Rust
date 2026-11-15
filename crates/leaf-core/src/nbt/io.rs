//! Бинарное чтение/запись NBT — перенос `NbtIo` + `TagType.load/skip`.
//!
//! Все целые — big-endian, строки — modified UTF-8 (`mutf8`).
//! Gzip-обёртка (`read_compressed`/`write_compressed`) — через flate2.

use crate::nbt::accounter::{NbtAccounter, NbtAccounterException};
use crate::nbt::compound::CompoundTag;
use crate::nbt::mutf8;
use crate::nbt::tag::*;
use std::fmt;
use std::io::{self, Read, Write};

#[derive(Debug)]
pub enum NbtError {
    Io(io::Error),
    Utf8,
    Format(String),
    Accounter(NbtAccounterException),
}

impl fmt::Display for NbtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NbtError::Io(e) => write!(f, "{e}"),
            NbtError::Utf8 => f.write_str("Malformed modified UTF-8"),
            NbtError::Format(m) => write!(f, "NBT format error: {m}"),
            NbtError::Accounter(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for NbtError {}

impl From<io::Error> for NbtError {
    fn from(e: io::Error) -> Self {
        NbtError::Io(e)
    }
}

impl From<NbtAccounterException> for NbtError {
    fn from(e: NbtAccounterException) -> Self {
        NbtError::Accounter(e)
    }
}

fn unexpected_eof() -> NbtError {
    NbtError::Io(io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of NBT data"))
}

// ---------- Reader ----------

pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], NbtError> {
        if self.remaining() < n {
            return Err(unexpected_eof());
        }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    pub fn read_u8(&mut self) -> Result<u8, NbtError> {
        Ok(self.take(1)?[0])
    }

    pub fn read_i8(&mut self) -> Result<i8, NbtError> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_i16(&mut self) -> Result<i16, NbtError> {
        let b = self.take(2)?;
        Ok(i16::from_be_bytes([b[0], b[1]]))
    }

    pub fn read_i32(&mut self) -> Result<i32, NbtError> {
        let b = self.take(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_i64(&mut self) -> Result<i64, NbtError> {
        let b = self.take(8)?;
        Ok(i64::from_be_bytes(b.try_into().unwrap()))
    }

    pub fn read_f32(&mut self) -> Result<f32, NbtError> {
        Ok(f32::from_bits(self.read_i32()? as u32))
    }

    pub fn read_f64(&mut self) -> Result<f64, NbtError> {
        Ok(f64::from_bits(self.read_i64()? as u64))
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], NbtError> {
        self.take(n)
    }

    /// `readUTF` — u16 длина + modified UTF-8.
    pub fn read_utf(&mut self) -> Result<String, NbtError> {
        let len = self.read_i16()? as usize;
        let bytes = self.take(len)?;
        mutf8::decode(bytes).map_err(|_| NbtError::Utf8)
    }

    /// `skipBytes`.
    pub fn skip(&mut self, n: usize) -> Result<(), NbtError> {
        self.take(n).map(|_| ())
    }
}

// ---------- Writer ----------

#[derive(Default)]
pub struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    fn put(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn write_i8(&mut self, v: i8) {
        self.buf.push(v as u8);
    }

    pub fn write_i16(&mut self, v: i16) {
        self.put(&v.to_be_bytes());
    }

    pub fn write_i32(&mut self, v: i32) {
        self.put(&v.to_be_bytes());
    }

    pub fn write_i64(&mut self, v: i64) {
        self.put(&v.to_be_bytes());
    }

    pub fn write_f32(&mut self, v: f32) {
        // DataOutput.writeFloat использует floatToIntBits — NaN канонизируется.
        let bits = if v.is_nan() { 0x7FC0_0000u32 } else { v.to_bits() };
        self.write_i32(bits as i32);
    }

    pub fn write_f64(&mut self, v: f64) {
        let bits = if v.is_nan() { 0x7FF8_0000_0000_0000u64 } else { v.to_bits() };
        self.write_i64(bits as i64);
    }

    /// `writeUTF` — при переполнении u16 кидает ошибку (как UTFDataFormatException).
    pub fn write_utf(&mut self, value: &str) -> Result<(), NbtError> {
        let bytes = mutf8::encode(value)
            .ok_or_else(|| NbtError::Format("NBT string too long (>65535 bytes)".into()))?;
        self.write_i16(bytes.len() as i16);
        self.put(&bytes);
        Ok(())
    }

    /// `writeUTF` с fallback на пустую строку (`StringFallbackDataOutput`).
    pub fn write_utf_fallback(&mut self, value: &str) {
        match mutf8::encode(value) {
            Some(bytes) => {
                self.write_i16(bytes.len() as i16);
                self.put(&bytes);
            }
            None => self.write_i16(0),
        }
    }
}

// ---------- Tag loading ----------

fn read_string(reader: &mut Reader, acc: &mut NbtAccounter) -> Result<String, NbtError> {
    let s = reader.read_utf()?;
    acc.account_bytes(28)?;
    acc.account_bytes(2 * s.len() as i64)?;
    Ok(s)
}

fn read_list_count(reader: &mut Reader) -> Result<i32, NbtError> {
    reader.read_i32()
}

fn load_tag(reader: &mut Reader, acc: &mut NbtAccounter, id: u8) -> Result<Tag, NbtError> {
    Ok(match id {
        TAG_END => Tag::End,
        TAG_BYTE => {
            acc.account_bytes(9)?;
            Tag::Byte(reader.read_i8()?)
        }
        TAG_SHORT => {
            acc.account_bytes(10)?;
            Tag::Short(reader.read_i16()?)
        }
        TAG_INT => {
            acc.account_bytes(12)?;
            Tag::Int(reader.read_i32()?)
        }
        TAG_LONG => {
            acc.account_bytes(16)?;
            Tag::Long(reader.read_i64()?)
        }
        TAG_FLOAT => {
            acc.account_bytes(12)?;
            Tag::Float(reader.read_f32()?)
        }
        TAG_DOUBLE => {
            acc.account_bytes(16)?;
            Tag::Double(reader.read_f64()?)
        }
        TAG_BYTE_ARRAY => {
            acc.account_bytes(24)?;
            let len = reader.read_i32()?;
            if len < 0 {
                return Err(NbtError::Format("Negative size in byte array".into()));
            }
            acc.account_bytes_per_item(1, len as i64)?;
            let bytes = reader.read_bytes(len as usize)?;
            Tag::ByteArray(bytes.iter().map(|b| *b as i8).collect())
        }
        TAG_STRING => Tag::String(read_string(reader, acc)?),
        TAG_LIST => load_list(reader, acc)?,
        TAG_COMPOUND => load_compound(reader, acc)?,
        TAG_INT_ARRAY => {
            acc.account_bytes(24)?;
            let len = reader.read_i32()?;
            if len < 0 {
                return Err(NbtError::Format("Negative size in int array".into()));
            }
            acc.account_bytes_per_item(4, len as i64)?;
            let mut out = Vec::with_capacity(len as usize);
            for _ in 0..len {
                out.push(reader.read_i32()?);
            }
            Tag::IntArray(out)
        }
        TAG_LONG_ARRAY => {
            acc.account_bytes(24)?;
            let len = reader.read_i32()?;
            if len < 0 {
                return Err(NbtError::Format("Negative size in long array".into()));
            }
            acc.account_bytes_per_item(8, len as i64)?;
            let mut out = Vec::with_capacity(len as usize);
            for _ in 0..len {
                out.push(reader.read_i64()?);
            }
            Tag::LongArray(out)
        }
        other => return Err(NbtError::Format(format!("Invalid tag id: {other}"))),
    })
}

fn load_list(reader: &mut Reader, acc: &mut NbtAccounter) -> Result<Tag, NbtError> {
    acc.push_depth_one()?;
    let result = (|| {
        acc.account_bytes(36)?;
        let elem_id = reader.read_u8()?;
        let count = read_list_count(reader)?;
        if elem_id == TAG_END && count > 0 {
            return Err(NbtError::Format("Missing type on ListTag".into()));
        }
        if count < 0 {
            return Err(NbtError::Format("Negative list size".into()));
        }
        acc.account_bytes_per_item(4, count as i64)?;
        let mut list = Vec::with_capacity(count as usize);
        for _ in 0..count {
            list.push(load_tag(reader, acc, elem_id)?);
        }
        Ok(Tag::List(list))
    })();
    acc.pop_depth()?;
    result
}

fn load_compound(reader: &mut Reader, acc: &mut NbtAccounter) -> Result<Tag, NbtError> {
    acc.push_depth_one()?;
    let result = (|| {
        acc.account_bytes(48)?;
        let mut map = CompoundTag::with_capacity(8);
        loop {
            let id = reader.read_u8()?;
            if id == TAG_END {
                break;
            }
            let name = read_string(reader, acc)?;
            let tag = load_tag(reader, acc, id)?;
            if map.put(name.clone(), tag).is_none() {
                acc.account_bytes(36)?;
            }
        }
        Ok(Tag::Compound(map))
    })();
    acc.pop_depth()?;
    result
}

/// `readUnnamedTag` — читает тег без имени (u8 type + payload).
pub fn read_unnamed_tag(reader: &mut Reader, acc: &mut NbtAccounter) -> Result<Tag, NbtError> {
    let id = reader.read_u8()?;
    if id == TAG_END {
        return Ok(Tag::End);
    }
    // имя пропускаем (всегда пустое при записи NbtIo)
    skip_string(reader)?;
    load_tag(reader, acc, id)
}

/// `readAnyTag` — u8 type, если не 0 то payload (без имени).
pub fn read_any_tag(reader: &mut Reader, acc: &mut NbtAccounter) -> Result<Tag, NbtError> {
    let id = reader.read_u8()?;
    if id == TAG_END {
        Ok(Tag::End)
    } else {
        load_tag(reader, acc, id)
    }
}

/// `NbtIo.read` — корневой тег обязан быть именованным compound.
pub fn read(reader: &mut Reader, acc: &mut NbtAccounter) -> Result<CompoundTag, NbtError> {
    match read_unnamed_tag(reader, acc)? {
        Tag::Compound(c) => Ok(c),
        _ => Err(NbtError::Format("Root tag must be a named compound tag".into())),
    }
}

pub fn read_bytes(data: &[u8], acc: &mut NbtAccounter) -> Result<CompoundTag, NbtError> {
    let mut reader = Reader::new(data);
    read(&mut reader, acc)
}

pub fn read_bytes_unlimited(data: &[u8]) -> Result<CompoundTag, NbtError> {
    read_bytes(data, &mut NbtAccounter::unlimited_heap())
}

// ---------- Tag skipping ----------

fn skip_string(reader: &mut Reader) -> Result<(), NbtError> {
    let len = reader.read_i16()? as usize;
    reader.skip(len)
}

/// `TagType.skip` — пропуск payload тега без построения значения.
pub fn skip_tag(reader: &mut Reader, acc: &mut NbtAccounter, id: u8) -> Result<(), NbtError> {
    skip_payload(reader, acc, id)
}

fn skip_payload(reader: &mut Reader, acc: &mut NbtAccounter, id: u8) -> Result<(), NbtError> {
    match id {
        TAG_END => Ok(()),
        TAG_BYTE => reader.skip(1).map_err(Into::into),
        TAG_SHORT => reader.skip(2).map_err(Into::into),
        TAG_INT | TAG_FLOAT => reader.skip(4).map_err(Into::into),
        TAG_LONG | TAG_DOUBLE => reader.skip(8).map_err(Into::into),
        TAG_BYTE_ARRAY | TAG_INT_ARRAY | TAG_LONG_ARRAY => {
            let len = reader.read_i32()?;
            if len < 0 {
                return Err(NbtError::Format("Negative array size".into()));
            }
            let elem = match id {
                TAG_BYTE_ARRAY => 1i64,
                TAG_INT_ARRAY => 4,
                _ => 8,
            };
            acc.account_bytes_per_item(elem, len as i64)?;
            reader.skip(len as usize * elem as usize).map_err(Into::into)
        }
        TAG_STRING => {
            acc.account_bytes(36)?;
            skip_string(reader)?;
            Ok(())
        }
        TAG_LIST => {
            acc.push_depth_one()?;
            let result = (|| {
                acc.account_bytes(36)?;
                let elem_id = reader.read_u8()?;
                let count = reader.read_i32()?;
                if count < 0 {
                    return Err(NbtError::Format("Negative list size".into()));
                }
                acc.account_bytes_per_item(4, count as i64)?;
                for _ in 0..count {
                    skip_payload(reader, acc, elem_id)?;
                }
                Ok(())
            })();
            acc.pop_depth()?;
            result
        }
        TAG_COMPOUND => {
            acc.push_depth_one()?;
            let result = (|| {
                acc.account_bytes(48)?;
                loop {
                    let id = reader.read_u8()?;
                    if id == TAG_END {
                        break;
                    }
                    skip_string(reader)?;
                    skip_payload(reader, acc, id)?;
                }
                Ok(())
            })();
            acc.pop_depth()?;
            result
        }
        other => Err(NbtError::Format(format!("Invalid tag id: {other}"))),
    }
}

// ---------- Tag writing ----------

fn write_payload(writer: &mut Writer, tag: &Tag) -> Result<(), NbtError> {
    match tag {
        Tag::End => Ok(()),
        Tag::Byte(v) => {
            writer.write_i8(*v);
            Ok(())
        }
        Tag::Short(v) => {
            writer.write_i16(*v);
            Ok(())
        }
        Tag::Int(v) => {
            writer.write_i32(*v);
            Ok(())
        }
        Tag::Long(v) => {
            writer.write_i64(*v);
            Ok(())
        }
        Tag::Float(v) => {
            writer.write_f32(*v);
            Ok(())
        }
        Tag::Double(v) => {
            writer.write_f64(*v);
            Ok(())
        }
        Tag::ByteArray(a) => {
            writer.write_i32(a.len() as i32);
            for b in a {
                writer.write_i8(*b);
            }
            Ok(())
        }
        Tag::String(s) => {
            writer.write_utf_fallback(s);
            Ok(())
        }
        Tag::List(list) => {
            let elem_id = list.first().map(Tag::id).unwrap_or(TAG_END);
            if list.iter().any(|t| t.id() != elem_id) {
                return Err(NbtError::Format(
                    "ListTag elements must all have the same type".into(),
                ));
            }
            writer.write_u8(elem_id);
            writer.write_i32(list.len() as i32);
            for t in list {
                write_payload(writer, t)?;
            }
            Ok(())
        }
        Tag::Compound(c) => {
            for (name, t) in c.iter() {
                writer.write_u8(t.id());
                writer.write_utf_fallback(name);
                write_payload(writer, t)?;
            }
            writer.write_u8(TAG_END);
            Ok(())
        }
        Tag::IntArray(a) => {
            writer.write_i32(a.len() as i32);
            for v in a {
                writer.write_i32(*v);
            }
            Ok(())
        }
        Tag::LongArray(a) => {
            writer.write_i32(a.len() as i32);
            for v in a {
                writer.write_i64(*v);
            }
            Ok(())
        }
    }
}

/// `writeUnnamedTag` — u8 type + пустое имя + payload.
pub fn write_unnamed_tag(tag: &Tag) -> Result<Vec<u8>, NbtError> {
    let mut w = Writer::new();
    w.write_u8(tag.id());
    if tag.id() != TAG_END {
        w.write_i16(0); // пустое имя
        write_payload(&mut w, tag)?;
    }
    Ok(w.into_vec())
}

/// `writeAnyTag` — u8 type + payload (без имени).
pub fn write_any_tag(tag: &Tag) -> Result<Vec<u8>, NbtError> {
    let mut w = Writer::new();
    w.write_u8(tag.id());
    if tag.id() != TAG_END {
        write_payload(&mut w, tag)?;
    }
    Ok(w.into_vec())
}

/// `NbtIo.write` — корневой compound с пустым именем.
pub fn write(tag: &CompoundTag) -> Result<Vec<u8>, NbtError> {
    write_unnamed_tag(&Tag::Compound(tag.clone()))
}

// ---------- Gzip ----------

pub fn read_compressed<R: Read>(reader: R, acc: &mut NbtAccounter) -> io::Result<CompoundTag> {
    let decoder = flate2::read::GzDecoder::new(reader);
    let mut buf = Vec::new();
    let mut limited = decoder;
    limited.read_to_end(&mut buf)?;
    read_bytes(&buf, acc).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

pub fn write_compressed<W: Write>(tag: &CompoundTag, writer: W) -> io::Result<()> {
    let data = write(tag).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let mut encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::default());
    encoder.write_all(&data)?;
    encoder.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_compound() {
        let mut c = CompoundTag::new();
        c.put("name", Tag::String("тест \u{0} 😀".into()));
        c.put("count", Tag::Int(42));
        c.put("flag", Tag::Byte(1));
        c.put("ratio", Tag::Double(0.5));
        c.put("list", Tag::List(vec![Tag::Int(1), Tag::Int(2)]));
        c.put("nested", Tag::Compound({
            let mut n = CompoundTag::new();
            n.put("x", Tag::Long(-7));
            n
        }));

        let bytes = write(&c).unwrap();
        let back = read_bytes_unlimited(&bytes).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn root_must_be_compound() {
        let bytes = write_any_tag(&Tag::Int(5)).unwrap();
        assert!(read_bytes_unlimited(&bytes).is_err());
    }

    #[test]
    fn gzip_roundtrip() {
        let mut c = CompoundTag::new();
        c.put("a", Tag::String("hello".into()));
        let mut enc = Vec::new();
        write_compressed(&c, &mut enc).unwrap();
        let back = read_compressed(&enc[..], &mut NbtAccounter::unlimited_heap()).unwrap();
        assert_eq!(back.get_string("a"), Some("hello"));
    }
}
