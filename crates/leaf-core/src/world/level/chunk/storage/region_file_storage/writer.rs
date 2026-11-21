use std::borrow::Cow;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::write::{GzEncoder, ZlibEncoder};
use flate2::Compression;
use fs2::FileExt;
use twox_hash::XxHash32;

use super::{
    invalid_data, location_offset, nbt_io, read_u32_be, ChunkPos, CompoundTag, RegionFileStorage,
    EXTERNAL_STREAM, HEADER_BYTES, SECTOR_BYTES, UNCOMPRESSED_NBT_QUOTA,
};

const ZLIB: u8 = 2;
const MAX_OFFSET: usize = 0xff_ffff;
const MAX_INLINE_SECTORS: u64 = 255;

impl RegionFileStorage {
    /// Записывает чанк, сохраняя остальные записи региона.
    pub fn write(&self, pos: ChunkPos, tag: &CompoundTag) -> io::Result<()> {
        let nbt = nbt_io::write(tag)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if nbt.len() > UNCOMPRESSED_NBT_QUOTA as usize {
            return Err(invalid_data("region NBT exceeds the uncompressed quota"));
        }
        let compressed = compress_zlib(&nbt)?;
        let sectors = sector_count(compressed.len() as u64 + 5);
        let external = sectors > MAX_INLINE_SECTORS;

        fs::create_dir_all(&self.directory)?;
        let mut region = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.region_path(pos))?;
        FileExt::lock_exclusive(&region)?;
        let mut file_len = region.metadata()?.len();
        if file_len == 0 {
            region.set_len(HEADER_BYTES)?;
            file_len = HEADER_BYTES;
        }
        if file_len < HEADER_BYTES {
            return Err(invalid_data("truncated region header"));
        }

        let header_offset = location_offset(pos);
        region.seek(SeekFrom::Start(header_offset))?;
        let location = read_u32_be(&mut region)?;
        let external_path = self.external_path(pos);
        let previous_codec = external_codec(&mut region, location, file_len)?;

        if let Some(codec) = previous_codec.filter(|_| external) {
            // Старый слот уже указывает на внешний поток. Оставляем его кодек,
            // чтобы публикация нового файла не зависела от второй замены заголовка.
            let payload = match codec {
                1 => Cow::Owned(compress_gzip(&nbt)?),
                ZLIB => Cow::Borrowed(compressed.as_slice()),
                3 => Cow::Borrowed(nbt.as_slice()),
                4 => Cow::Owned(encode_lz4(&nbt)),
                _ => return Err(invalid_data("unsupported external chunk codec")),
            };
            write_timestamp(&mut region, header_offset)?;
            region.sync_all()?;
            return replace_external(&external_path, &payload);
        }

        let sectors = if external { 1 } else { sectors as usize };
        let start = allocate_sectors(&mut region, file_len, sectors)?;
        let end = (start + sectors) as u64 * SECTOR_BYTES;
        if end > file_len {
            region.set_len(end)?;
        }
        region.seek(SeekFrom::Start(start as u64 * SECTOR_BYTES))?;
        if external {
            replace_external(&external_path, &compressed)?;
            region.write_all(&[0, 0, 0, 1, ZLIB | EXTERNAL_STREAM])?;
        } else {
            region.write_all(&(1 + compressed.len() as u32).to_be_bytes())?;
            region.write_all(&[ZLIB])?;
            region.write_all(&compressed)?;
        }
        region.sync_all()?;

        write_timestamp(&mut region, header_offset)?;
        region.seek(SeekFrom::Start(header_offset))?;
        region.write_all(&(((start as u32) << 8) | sectors as u32).to_be_bytes())?;
        region.sync_all()?;
        if previous_codec.is_some() {
            // После публикации внутренней записи старый .mcc больше не читается.
            let _ = fs::remove_file(external_path);
        }
        Ok(())
    }
}

fn sector_count(bytes: u64) -> u64 {
    bytes.div_ceil(SECTOR_BYTES)
}

fn external_codec(region: &mut File, location: u32, file_len: u64) -> io::Result<Option<u8>> {
    if location == 0 {
        return Ok(None);
    }
    let start = u64::from(location >> 8) * SECTOR_BYTES;
    if start < HEADER_BYTES || start + 5 > file_len || location & 0xff == 0 {
        return Err(invalid_data("invalid region sector location"));
    }
    region.seek(SeekFrom::Start(start))?;
    let len = u64::from(read_u32_be(region)?);
    let mut codec = [0];
    region.read_exact(&mut codec)?;
    if codec[0] & EXTERNAL_STREAM == 0 {
        return Ok(None);
    }
    if len != 1 {
        return Err(invalid_data(
            "external chunk also declares an inline payload",
        ));
    }
    Ok(Some(codec[0] & !EXTERNAL_STREAM))
}

fn allocate_sectors(region: &mut File, file_len: u64, sectors: usize) -> io::Result<usize> {
    let count = sector_count(file_len);
    if count > MAX_OFFSET as u64 + MAX_INLINE_SECTORS {
        return Err(invalid_data("region exceeds Anvil sector address space"));
    }
    let mut in_use = vec![false; count as usize];
    in_use[..2].fill(true);
    let mut table = [0; SECTOR_BYTES as usize];
    region.seek(SeekFrom::Start(0))?;
    region.read_exact(&mut table)?;
    for slot in table.chunks_exact(4) {
        let location = u32::from_be_bytes(slot.try_into().unwrap());
        if location == 0 {
            continue;
        }
        let start = (location >> 8) as usize;
        let mut len = (location & 0xff) as usize;
        if len == MAX_INLINE_SECTORS as usize {
            // Spigot допускает расширенную запись, помеченную длиной 255.
            region.seek(SeekFrom::Start(start as u64 * SECTOR_BYTES))?;
            let payload_len = u64::from(read_u32_be(region)?);
            len = len.max(sector_count(4 + payload_len) as usize);
        }
        let end = start + len;
        if start < 2
            || len == 0
            || end > in_use.len()
            || in_use[start..end].iter().any(|used| *used)
        {
            return Err(invalid_data("invalid or overlapping region sectors"));
        }
        in_use[start..end].fill(true);
    }

    let mut run = 0;
    for (index, used) in in_use.into_iter().enumerate() {
        run = if used { 0 } else { run + 1 };
        if run == sectors {
            let start = index + 1 - run;
            if start <= MAX_OFFSET {
                return Ok(start);
            }
            return Err(invalid_data("region has no addressable free sector"));
        }
    }
    let start = count as usize - run;
    if start > MAX_OFFSET {
        return Err(invalid_data("region has no addressable free sector"));
    }
    Ok(start)
}

fn write_timestamp(region: &mut File, header_offset: u64) -> io::Result<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    region.seek(SeekFrom::Start(SECTOR_BYTES + header_offset))?;
    region.write_all(&(timestamp.as_secs() as u32).to_be_bytes())
}

fn replace_external(path: &Path, data: &[u8]) -> io::Result<()> {
    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);
    let (temporary, mut file) = loop {
        let suffix = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let temporary = path.with_extension(format!("mcc.{}.{}.tmp", std::process::id(), suffix));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => break (temporary, file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    let write = file.write_all(data).and_then(|_| file.sync_all());
    drop(file);
    let result = write.and_then(|_| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

fn compress_zlib(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

fn compress_gzip(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

// LZ4BlockOutputStream допускает несжатые блоки. Пишем их при обновлении
// существующего внешнего потока LZ4, чтобы можно было оставить его кодек.
fn encode_lz4(data: &[u8]) -> Vec<u8> {
    const BLOCK_SIZE: usize = 64 * 1024;
    let mut output = Vec::new();
    for block in data.chunks(BLOCK_SIZE).chain(std::iter::once(&[][..])) {
        let len = (block.len() as u32).to_le_bytes();
        let checksum = if block.is_empty() {
            0
        } else {
            XxHash32::oneshot(0x9747_b28c, block) & 0x0fff_ffff
        };
        output.extend_from_slice(b"LZ4Block");
        output.push(0x16);
        output.extend_from_slice(&len);
        output.extend_from_slice(&len);
        output.extend_from_slice(&checksum.to_le_bytes());
        output.extend_from_slice(block);
    }
    output
}
