//! Перенос `net.minecraft.util.PngInfo`.

use std::io::{self, Read};

const PNG_HEADER: u64 = 0x89504E470D0A1A0A;
const IHDR_TYPE: u32 = 0x49484452; // "IHDR"
const IHDR_SIZE: u32 = 13;

/// `PngInfo(width, height)` — размеры из PNG-заголовка.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PngInfo {
    pub width: i32,
    pub height: i32,
}

impl PngInfo {
    /// `fromStream(InputStream)` — читает сигнатуру + IHDR.
    pub fn from_stream(mut stream: impl Read) -> io::Result<Self> {
        let mut header = [0u8; 8];
        stream.read_exact(&mut header)?;
        let sig = u64::from_be_bytes(header);
        if sig != PNG_HEADER {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Bad PNG Signature: {:016X}", sig),
            ));
        }
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf);
        if len != IHDR_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Bad length for IHDR chunk: {}", len),
            ));
        }
        let mut type_buf = [0u8; 4];
        stream.read_exact(&mut type_buf)?;
        let ty = u32::from_be_bytes(type_buf);
        if ty != IHDR_TYPE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Bad type for IHDR chunk: {:08X}", ty),
            ));
        }
        let mut dim = [0u8; 8];
        stream.read_exact(&mut dim)?;
        Ok(Self {
            width: i32::from_be_bytes([dim[0], dim[1], dim[2], dim[3]]),
            height: i32::from_be_bytes([dim[4], dim[5], dim[6], dim[7]]),
        })
    }

    /// `fromBytes(byte[])`.
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Self::from_stream(bytes)
    }

    /// `validateHeader(ByteBuffer)` — проверка первых 16 байт PNG.
    pub fn validate_header(buffer: &[u8]) -> io::Result<()> {
        if buffer.len() < 16 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "PNG header missing"));
        }
        let sig = u64::from_be_bytes(buffer[0..8].try_into().unwrap());
        if sig != PNG_HEADER {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Bad PNG Signature"));
        }
        let len = u32::from_be_bytes(buffer[8..12].try_into().unwrap());
        if len != IHDR_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Bad length for IHDR chunk!"));
        }
        let ty = u32::from_be_bytes(buffer[12..16].try_into().unwrap());
        if ty != IHDR_TYPE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Bad type for IHDR chunk!"));
        }
        Ok(())
    }
}
