//! Чтение и запись регионов Anvil и вынесенных файлов чанков.

use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;

use flate2::read::{MultiGzDecoder, ZlibDecoder};
use fs2::FileExt;
use twox_hash::XxHash32;

use crate::nbt::accounter::{NbtAccounter, UNCOMPRESSED_NBT_QUOTA};
use crate::nbt::{io as nbt_io, CompoundTag};
use crate::world::level::chunk_pos::ChunkPos;

mod writer;

const SECTOR_BYTES: u64 = 4096;
const HEADER_BYTES: u64 = 2 * SECTOR_BYTES;
const EXTERNAL_STREAM: u8 = 0x80;

/// Источник NBT для каталогов Anvil (`region`, `entities`, `poi`).
pub struct RegionFileStorage {
    directory: PathBuf,
}

impl RegionFileStorage {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    /// Читает запись чанка, включая отдельный файл `c.<x>.<z>.mcc`.
    /// Отсутствующий регион или пустая запись возвращают `Ok(None)`.
    pub fn read(&self, pos: ChunkPos) -> io::Result<Option<CompoundTag>> {
        let path = self.region_path(pos);
        let region = match File::open(path) {
            Ok(region) => region,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        FileExt::lock_shared(&region)?;
        let file_len = region.metadata()?.len();
        if file_len == 0 {
            return Ok(None);
        }
        if file_len < HEADER_BYTES {
            return Err(invalid_data("truncated region header"));
        }

        let mut region = BufReader::new(region);
        region.seek(SeekFrom::Start(location_offset(pos)))?;
        let location = read_u32_be(&mut region)?;
        if location == 0 {
            return Ok(None);
        }

        let sector_offset = u64::from(location >> 8) * SECTOR_BYTES;
        let sectors = u64::from(location & 0xff);
        if sectors == 0 || sector_offset < HEADER_BYTES || sector_offset + 5 > file_len {
            return Err(invalid_data("invalid region sector location"));
        }

        region.seek(SeekFrom::Start(sector_offset))?;
        let length = u64::from(read_u32_be(&mut region)?);
        if length == 0 || length > i32::MAX as u64 || sector_offset + 4 + length > file_len {
            return Err(invalid_data("invalid region chunk length"));
        }
        // Spigot used 255 as a marker for older chunks spanning more than 255 sectors.
        if sectors != 255 && 4 + length > sectors * SECTOR_BYTES {
            return Err(invalid_data("chunk exceeds its allocated region sectors"));
        }

        let mut version = [0];
        region.read_exact(&mut version)?;
        let version = version[0];
        let compression = version & !EXTERNAL_STREAM;
        let tag = if version & EXTERNAL_STREAM != 0 {
            if length != 1 {
                return Err(invalid_data(
                    "external chunk also declares an inline payload",
                ));
            }
            let path = self.external_path(pos);
            decode_chunk(BufReader::new(File::open(path)?), compression)?
        } else {
            decode_chunk(region.take(length - 1), compression)?
        };
        Ok(Some(tag))
    }

    fn region_path(&self, pos: ChunkPos) -> PathBuf {
        self.directory
            .join(format!("r.{}.{}.mca", pos.x >> 5, pos.z >> 5))
    }

    fn external_path(&self, pos: ChunkPos) -> PathBuf {
        self.directory.join(format!("c.{}.{}.mcc", pos.x, pos.z))
    }
}

fn location_offset(pos: ChunkPos) -> u64 {
    u64::from(((pos.x & 31) + (pos.z & 31) * 32) as u32) * 4
}

fn read_u32_be(reader: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_be_bytes(bytes))
}

fn decode_chunk(source: impl Read, compression: u8) -> io::Result<CompoundTag> {
    let bytes = match compression {
        1 => read_limited(MultiGzDecoder::new(source))?,
        2 => read_limited(ZlibDecoder::new(source))?,
        3 => read_limited(source)?,
        4 => read_lz4(source)?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("region compression {compression}"),
            ))
        }
    };
    nbt_io::read_bytes(&bytes, &mut NbtAccounter::uncompressed_quota())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn read_limited(source: impl Read) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    source
        .take(UNCOMPRESSED_NBT_QUOTA as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > UNCOMPRESSED_NBT_QUOTA as usize {
        return Err(invalid_data("region NBT exceeds the uncompressed quota"));
    }
    Ok(bytes)
}

// Minecraft uses lz4-java's LZ4BlockInputStream format.
fn read_lz4(mut source: impl Read) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut compressed = Vec::new();
    loop {
        let mut header = [0; 21];
        source.read_exact(&mut header)?;
        if &header[..8] != b"LZ4Block" {
            return Err(invalid_data("invalid LZ4 block signature"));
        }
        let method = header[8] & 0xf0;
        let max_block_len = 1 << (10 + (header[8] & 0x0f));
        let compressed_len = u32::from_le_bytes(header[9..13].try_into().unwrap()) as usize;
        let decoded_len = u32::from_le_bytes(header[13..17].try_into().unwrap()) as usize;
        let checksum = u32::from_le_bytes(header[17..21].try_into().unwrap());
        if !matches!(method, 0x10 | 0x20)
            || decoded_len > max_block_len
            || compressed_len > decoded_len
            || (method == 0x10 && compressed_len != decoded_len)
            || (decoded_len == 0) != (compressed_len == 0)
        {
            return Err(invalid_data("invalid LZ4 block lengths or method"));
        }
        if decoded_len == 0 {
            return if checksum == 0 {
                Ok(output)
            } else {
                Err(invalid_data("invalid LZ4 terminator"))
            };
        }

        let start = output.len();
        if start + decoded_len > UNCOMPRESSED_NBT_QUOTA as usize {
            return Err(invalid_data("region NBT exceeds the uncompressed quota"));
        }
        output.resize(start + decoded_len, 0);
        let block = &mut output[start..];
        if method == 0x10 {
            source.read_exact(block)?;
        } else {
            compressed.resize(compressed_len, 0);
            source.read_exact(&mut compressed)?;
            let written = lz4_flex::block::decompress_into(&compressed, block)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            if written != decoded_len {
                return Err(invalid_data("LZ4 block decoded to the wrong size"));
            }
        }

        // StreamingXXHash32.asChecksum() in lz4-java 1.8.1 keeps the low 28 bits.
        if XxHash32::oneshot(0x9747_b28c, block) & 0x0fff_ffff != checksum {
            return Err(invalid_data("LZ4 block checksum mismatch"));
        }
    }
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    use base64::Engine;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;

    #[test]
    fn reads_java_lz4_block_stream() {
        // Generated by Leaf's lz4-java 1.8.1 with 64-byte blocks.
        let fixture = concat!(
            "TFo0QmxvY2sgKAAAAEAAAADn3xsM/w4KAAADAAR4UG9z/////wcAB3BheWxvYWQAAACAAAEAC1AAAA",
            "AAAExaNEJsb2NrIAsAAABAAAAABJn6Ax8AAQAnUAAAAAAATFo0QmxvY2sgCwAAAB0AAACbyogBHwABA",
            "ARQAAAAAABMWjRCbG9jaxAAAAAAAAAAAAAAAAA="
        );
        let mut bytes = base64::engine::general_purpose::STANDARD
            .decode(fixture)
            .unwrap();

        let tag = decode_chunk(bytes.as_slice(), 4).unwrap();
        assert_eq!(tag.get_int("xPos"), Some(-1));
        assert_eq!(tag.get_byte_array("payload"), Some(&[0; 128][..]));

        bytes[17] ^= 1;
        assert_eq!(
            decode_chunk(bytes.as_slice(), 4).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn reads_negative_region_coordinates_and_external_payloads() {
        let directory = test_directory();
        let storage = RegionFileStorage::new(&directory.0);
        let pos = ChunkPos::new(-1, -33);
        assert!(storage.read(pos).unwrap().is_none());

        let mut tag = CompoundTag::new();
        tag.put_int("xPos", pos.x);
        tag.put_int("zPos", pos.z);
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&nbt_io::write(&tag).unwrap()).unwrap();
        let deflate = encoder.finish().unwrap();
        let region_path = directory.0.join("r.-1.-2.mca");
        fs::write(&region_path, region_bytes(2, &deflate)).unwrap();
        assert_eq!(storage.read(pos).unwrap(), Some(tag.clone()));
        assert!(storage.read(ChunkPos::new(-2, -33)).unwrap().is_none());

        fs::write(&region_path, region_bytes(1 | EXTERNAL_STREAM, &[])).unwrap();
        let mut gzip = Vec::new();
        nbt_io::write_compressed(&tag, &mut gzip).unwrap();
        fs::write(directory.0.join("c.-1.-33.mcc"), gzip).unwrap();
        assert_eq!(storage.read(pos).unwrap(), Some(tag));

        let mut truncated = region_bytes(2, &deflate);
        truncated.pop();
        fs::write(region_path, truncated).unwrap();
        assert_eq!(
            storage.read(pos).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn writes_inline_and_external_chunks_without_losing_neighbours() {
        use rand::{rngs::StdRng, RngCore, SeedableRng};

        let directory = test_directory();
        let storage = RegionFileStorage::new(&directory.0);
        let pos = ChunkPos::new(-1, -33);
        let neighbour = ChunkPos::new(-2, -33);
        let mut tag = CompoundTag::new();
        tag.put_int("xPos", pos.x);
        tag.put_int("zPos", pos.z);
        let mut other = CompoundTag::new();
        other.put_int("xPos", neighbour.x);
        other.put_int("zPos", neighbour.z);
        std::thread::scope(|scope| {
            let storage2 = RegionFileStorage::new(&directory.0);
            let other = &other;
            let neighbour_write = scope.spawn(move || storage2.write(neighbour, other));
            storage.write(pos, &tag).unwrap();
            neighbour_write.join().unwrap().unwrap();
        });
        assert_eq!(storage.read(pos).unwrap(), Some(tag.clone()));
        assert_eq!(storage.read(neighbour).unwrap(), Some(other.clone()));

        let mut random = vec![0; 1100 * 1024];
        StdRng::seed_from_u64(42).fill_bytes(&mut random);
        tag.put(
            "payload",
            crate::nbt::Tag::ByteArray(random.into_iter().map(|byte| byte as i8).collect()),
        );
        storage.write(pos, &tag).unwrap();
        assert!(storage.external_path(pos).is_file());
        assert_eq!(storage.read(pos).unwrap(), Some(tag.clone()));
        assert_eq!(storage.read(neighbour).unwrap(), Some(other.clone()));

        tag.put_bool("edited", true);
        storage.write(pos, &tag).unwrap();
        assert_eq!(storage.read(pos).unwrap(), Some(tag.clone()));
        tag.remove("payload");
        storage.write(pos, &tag).unwrap();
        assert!(!storage.external_path(pos).exists());
        assert_eq!(storage.read(pos).unwrap(), Some(tag));
        assert_eq!(storage.read(neighbour).unwrap(), Some(other));
    }

    fn test_directory() -> TestDirectory {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("leaf-anvil-{}-{unique_id}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        TestDirectory(directory)
    }

    fn region_bytes(compression: u8, data: &[u8]) -> Vec<u8> {
        let mut region = vec![0; HEADER_BYTES as usize];
        // Last location slot, pointing to sector 2 for one sector.
        region[4092..4096].copy_from_slice(&0x0201u32.to_be_bytes());
        region.extend_from_slice(&(1 + data.len() as u32).to_be_bytes());
        region.push(compression);
        region.extend_from_slice(data);
        region
    }

    struct TestDirectory(PathBuf);

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            if let Ok(files) = fs::read_dir(&self.0) {
                for file in files.flatten() {
                    let _ = fs::remove_file(file.path());
                }
            }
            let _ = fs::remove_dir(&self.0);
        }
    }
}
