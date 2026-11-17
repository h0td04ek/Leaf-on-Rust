//! Перенос `net.minecraft.network.CompressionEncoder` /
//! `CompressionDecoder` и `Varint21LengthFieldPrepender` —
//! zlib-сжатие пакетов (flate2 вместо java.util.zip).
//!
//! Paper-ветка VelocityCompressor заменена стандартным zlib —
//! формат на проводе идентичен (`Deflater`/`Inflater` default = zlib).

use crate::network::codec::exceptions::{DecoderException, EncoderException};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::var_int;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression as FlateCompression;
use std::io::{Read, Write};

/// `CompressionDecoder.MAXIMUM_COMPRESSED_LENGTH`.
pub const MAXIMUM_COMPRESSED_LENGTH: usize = 2097152;
/// `CompressionDecoder.MAXIMUM_UNCOMPRESSED_LENGTH`.
pub const MAXIMUM_UNCOMPRESSED_LENGTH: usize = 8388608;

/// `CompressionEncoder`/`CompressionDecoder` — состояние сжатия.
pub struct Compression {
    threshold: usize,
    validate_decompressed: bool,
}

impl Compression {
    /// `new CompressionDecoder(threshold, validateDecompressed)` /
    /// `new CompressionEncoder(threshold)` — общий стейт.
    pub fn new(threshold: usize, validate_decompressed: bool) -> Self {
        Self { threshold, validate_decompressed }
    }

    /// `CompressionEncoder.getThreshold`.
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// `CompressionEncoder.setThreshold`.
    pub fn set_threshold(&mut self, threshold: usize) {
        self.threshold = threshold;
    }

    /// `CompressionEncoder.encode` — ByteBuf → ByteBuf.
    /// Формат: VarInt dataLength; 0 → несжатые данные; иначе zlib-поток.
    pub fn encode(&self, input: &[u8]) -> Result<Vec<u8>, EncoderException> {
        let i = input.len();
        if i > MAXIMUM_UNCOMPRESSED_LENGTH {
            return Err(EncoderException::new(format!(
                "Packet too big (is {}, should be less than {})",
                i, MAXIMUM_UNCOMPRESSED_LENGTH
            )));
        }
        let mut out = FriendlyByteBuf::new();
        if i < self.threshold {
            var_int::write(&mut out, 0);
            out.write_bytes(input);
        } else {
            var_int::write(&mut out, i as i32);
            let mut encoder = ZlibEncoder::new(Vec::new(), FlateCompression::default());
            encoder
                .write_all(input)
                .map_err(|e| EncoderException::with_cause(e))?;
            let compressed = encoder
                .finish()
                .map_err(|e| EncoderException::with_cause(e))?;
            out.write_bytes(&compressed);
        }
        Ok(out.readable_bytes_vec())
    }

    /// `CompressionDecoder.decode` — ByteBuf → ByteBuf.
    pub fn decode(&self, input: &mut FriendlyByteBuf) -> Result<Vec<u8>, DecoderException> {
        let i = var_int::read(input);
        if i == 0 {
            let len = input.readable_bytes();
            return Ok(input.read_bytes_vec(len));
        }
        let size = i as usize;
        if self.validate_decompressed {
            if size < self.threshold {
                return Err(DecoderException::new(format!(
                    "Badly compressed packet - size of {} is below server threshold of {}",
                    size, self.threshold
                )));
            }
            if size > MAXIMUM_UNCOMPRESSED_LENGTH {
                return Err(DecoderException::new(format!(
                    "Badly compressed packet - size of {} is larger than protocol maximum of {}",
                    size, MAXIMUM_UNCOMPRESSED_LENGTH
                )));
            }
        }
        // compressed data = remaining readable bytes
        let compressed = input.read_bytes_vec(input.readable_bytes());
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut out = Vec::with_capacity(size);
        decoder
            .take(MAXIMUM_UNCOMPRESSED_LENGTH as u64 + 1)
            .read_to_end(&mut out)
            .map_err(|e| DecoderException::with_cause(e))?;
        if out.len() > MAXIMUM_UNCOMPRESSED_LENGTH {
            return Err(DecoderException::new(format!(
                "Badly compressed packet - inflated size exceeds {}",
                MAXIMUM_UNCOMPRESSED_LENGTH
            )));
        }
        Ok(out)
    }
}

/// `Varint21LengthFieldPrepender.encode` — VarInt длины + данные.
pub fn prepend_varint21_length(data: &[u8]) -> Result<Vec<u8>, EncoderException> {
    let i = data.len();
    let byte_size = var_int::get_byte_size(i as i32);
    if byte_size > 3 {
        return Err(EncoderException::new(format!(
            "Packet too large: size {} is over {}",
            i, MAXIMUM_UNCOMPRESSED_LENGTH
        )));
    }
    let mut out = FriendlyByteBuf::with_capacity(byte_size + i);
    var_int::write(&mut out, i as i32);
    out.write_bytes(data);
    Ok(out.readable_bytes_vec())
}

/// `Varint21LengthFieldPrepender` — декодирование: прочитать VarInt
/// длину и вернуть срез данных (для frame-декодера).
pub fn read_varint21_length(input: &mut FriendlyByteBuf) -> Result<usize, DecoderException> {
    let len = var_int::read(input);
    if len < 0 {
        return Err(DecoderException::new(format!("Negative frame length {}", len)));
    }
    Ok(len as usize)
}
