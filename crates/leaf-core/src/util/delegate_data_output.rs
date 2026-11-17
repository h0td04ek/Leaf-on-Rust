//! Перенос `net.minecraft.util.DelegateDataOutput`.
//! `java.io.DataOutput` → `std::io::Write` (big-endian, как DataOutputStream).

use std::io::{self, Write};

/// `DelegateDataOutput` — DataOutput, делегирующий родителю.
pub struct DelegateDataOutput<'a> {
    parent: &'a mut dyn Write,
}

impl<'a> DelegateDataOutput<'a> {
    pub fn new(parent: &'a mut dyn Write) -> Self {
        Self { parent }
    }

    fn write_i32_be(&mut self, value: i32) -> io::Result<()> {
        self.parent.write_all(&value.to_be_bytes())
    }
    fn write_i16_be(&mut self, value: i16) -> io::Result<()> {
        self.parent.write_all(&value.to_be_bytes())
    }
    fn write_i64_be(&mut self, value: i64) -> io::Result<()> {
        self.parent.write_all(&value.to_be_bytes())
    }
    fn write_f32_be(&mut self, value: f32) -> io::Result<()> {
        self.parent.write_all(&value.to_bits().to_be_bytes())
    }
    fn write_f64_be(&mut self, value: f64) -> io::Result<()> {
        self.parent.write_all(&value.to_bits().to_be_bytes())
    }
}

impl Write for DelegateDataOutput<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.parent.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.parent.flush()
    }
}

impl DelegateDataOutput<'_> {
    /// `writeBoolean`.
    pub fn write_boolean(&mut self, value: bool) -> io::Result<()> {
        self.parent.write_all(&[value as u8])
    }
    /// `writeByte`.
    pub fn write_byte(&mut self, value: i8) -> io::Result<()> {
        self.parent.write_all(&[value as u8])
    }
    /// `writeShort`.
    pub fn write_short(&mut self, value: i16) -> io::Result<()> {
        self.write_i16_be(value)
    }
    /// `writeChar`.
    pub fn write_char(&mut self, value: u16) -> io::Result<()> {
        self.parent.write_all(&value.to_be_bytes())
    }
    /// `writeInt`.
    pub fn write_int(&mut self, value: i32) -> io::Result<()> {
        self.write_i32_be(value)
    }
    /// `writeLong`.
    pub fn write_long(&mut self, value: i64) -> io::Result<()> {
        self.write_i64_be(value)
    }
    /// `writeFloat`.
    pub fn write_float(&mut self, value: f32) -> io::Result<()> {
        self.write_f32_be(value)
    }
    /// `writeDouble`.
    pub fn write_double(&mut self, value: f64) -> io::Result<()> {
        self.write_f64_be(value)
    }
    /// `writeBytes(String)` — модифицированный UTF-8 без длины.
    pub fn write_bytes(&mut self, value: &str) -> io::Result<()> {
        for b in value.bytes() {
            self.parent.write_all(&[b])?;
        }
        Ok(())
    }
    /// `writeChars(String)` — каждый char как 2 байта BE.
    pub fn write_chars(&mut self, value: &str) -> io::Result<()> {
        for c in value.encode_utf16() {
            self.parent.write_all(&c.to_be_bytes())?;
        }
        Ok(())
    }
    /// `writeUTF(String)` — modified UTF-8 с 2-байтовой длиной.
    pub fn write_utf(&mut self, value: &str) -> io::Result<()> {
        let bytes = crate::nbt::mutf8::encode(value)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Unpaired surrogate"))?;
        if bytes.len() > u16::MAX as usize {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "UTF string too long"));
        }
        self.parent.write_all(&(bytes.len() as u16).to_be_bytes())?;
        self.parent.write_all(&bytes)
    }
}
