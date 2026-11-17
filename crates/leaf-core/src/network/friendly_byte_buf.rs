//! Перенос `io.netty.buffer.ByteBuf` (ядро) +
//! `net.minecraft.network.FriendlyByteBuf`.
//!
//! Netty-специфика (reference counting, direct buffers, channels) не
//! портируется — буфер это Vec<u8> + reader/writer индексы,
//! big-endian по умолчанию (как ByteBuf.DEFAULT_ORDER).

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::global_pos::GlobalPos;
use crate::core::registry::Registry;
use crate::core::section_pos::SectionPos;
use crate::core::vec3i::Vec3i;
use crate::nbt::accounter::NbtAccounter;
use crate::nbt::compound::CompoundTag;
use crate::nbt::io as nbt_io;
use crate::nbt::tag::Tag;
use crate::network::utf8_string::{self, CodecException};
use crate::network::var_int;
use crate::network::var_long;
use crate::resources::{Identifier, ResourceKey};
use crate::util::mth;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::Level;
use crate::world::phys::vec3;
use crate::world::phys::{BlockHitResult, HitResultType, Vec3};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

pub const MAX_STRING_LENGTH: usize = 32767;
pub const MAX_COMPONENT_STRING_LENGTH: usize = 262144;

/// Ошибки кодирования/декодирования пакетов (аналог Netty Decoder/EncoderException).
#[derive(Debug)]
pub struct CodecError(pub String);

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for CodecError {}

impl From<CodecException> for CodecError {
    fn from(e: CodecException) -> Self {
        CodecError(e.0)
    }
}

// ===================================================================== ByteBuf

/// Ядро буфера — аналог `io.netty.buffer.ByteBuf` (heap, big-endian).
#[derive(Clone, Debug, Default)]
pub struct ByteBuf {
    pub data: Vec<u8>,
    pub reader_index: usize,
    pub writer_index: usize,
}

impl ByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { data: Vec::with_capacity(capacity), reader_index: 0, writer_index: 0 }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self { data: bytes.to_vec(), reader_index: 0, writer_index: bytes.len() }
    }

    // ---- индексы ----------------------------------------------------

    pub fn reader_index(&self) -> usize {
        self.reader_index
    }

    pub fn writer_index(&self) -> usize {
        self.writer_index
    }

    pub fn set_index(&mut self, reader_index: usize, writer_index: usize) {
        assert!(reader_index <= writer_index && writer_index <= self.data.len());
        self.reader_index = reader_index;
        self.writer_index = writer_index;
    }

    pub fn readable_bytes(&self) -> usize {
        self.writer_index - self.reader_index
    }

    pub fn writable_bytes(&self) -> usize {
        self.data.len() - self.writer_index
    }

    pub fn max_writable_bytes(&self) -> usize {
        usize::MAX - self.writer_index
    }

    pub fn is_readable(&self) -> bool {
        self.writer_index > self.reader_index
    }

    pub fn is_readable_n(&self, size: usize) -> bool {
        self.writer_index - self.reader_index >= size
    }

    pub fn is_writable(&self) -> bool {
        self.writable_bytes() > 0
    }

    pub fn is_writable_n(&self, size: usize) -> bool {
        self.writable_bytes() >= size
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn max_capacity(&self) -> usize {
        usize::MAX
    }

    pub fn clear(&mut self) {
        self.reader_index = 0;
        self.writer_index = 0;
    }

    pub fn mark_reader_index(&mut self) -> usize {
        self.reader_index
    }

    pub fn reset_reader_index(&mut self, mark: usize) {
        self.reader_index = mark;
    }

    pub fn mark_writer_index(&mut self) -> usize {
        self.writer_index
    }

    pub fn reset_writer_index(&mut self, mark: usize) {
        self.writer_index = mark;
    }

    pub fn discard_read_bytes(&mut self) {
        self.data.drain(..self.reader_index);
        self.writer_index -= self.reader_index;
        self.reader_index = 0;
    }

    pub fn discard_some_read_bytes(&mut self) {
        if self.reader_index > 0 {
            self.discard_read_bytes();
        }
    }

    /// Чтение без продвижения readerIndex — getX(index).
    fn check_read(&self, index: usize, length: usize) {
        assert!(index + length <= self.writer_index, "index out of bounds");
    }

    fn check_write(&mut self, index: usize, length: usize) {
        if index + length > self.data.len() {
            self.data.resize(index + length, 0);
        }
        if index + length > self.writer_index {
            self.writer_index = index + length;
        }
    }

    // ---- примитивы (absolute get/set, big-endian) --------------------

    pub fn get_u8(&self, index: usize) -> u8 {
        self.data[index]
    }

    pub fn get_bool(&self, index: usize) -> bool {
        self.get_u8(index) != 0
    }

    pub fn get_i16(&self, index: usize) -> i16 {
        i16::from_be_bytes([self.data[index], self.data[index + 1]])
    }

    pub fn get_i16_le(&self, index: usize) -> i16 {
        i16::from_le_bytes([self.data[index], self.data[index + 1]])
    }

    pub fn get_u16(&self, index: usize) -> u16 {
        u16::from_be_bytes([self.data[index], self.data[index + 1]])
    }

    pub fn get_i32(&self, index: usize) -> i32 {
        i32::from_be_bytes(self.data[index..index + 4].try_into().unwrap())
    }

    pub fn get_i32_le(&self, index: usize) -> i32 {
        i32::from_le_bytes(self.data[index..index + 4].try_into().unwrap())
    }

    pub fn get_i64(&self, index: usize) -> i64 {
        i64::from_be_bytes(self.data[index..index + 8].try_into().unwrap())
    }

    pub fn get_i64_le(&self, index: usize) -> i64 {
        i64::from_le_bytes(self.data[index..index + 8].try_into().unwrap())
    }

    pub fn get_f32(&self, index: usize) -> f32 {
        f32::from_bits(self.get_i32(index) as u32)
    }

    pub fn get_f64(&self, index: usize) -> f64 {
        f64::from_bits(self.get_i64(index) as u64)
    }

    /// 3-байтовое medium (big-endian).
    pub fn get_medium(&self, index: usize) -> i32 {
        ((self.data[index] as i32) << 16) | ((self.data[index + 1] as i32) << 8) | self.data[index + 2] as i32
    }

    pub fn get_medium_le(&self, index: usize) -> i32 {
        self.data[index] as i32 | ((self.data[index + 1] as i32) << 8) | ((self.data[index + 2] as i32) << 16)
    }

    pub fn set_u8(&mut self, index: usize, value: u8) {
        self.check_write(index, 1);
        self.data[index] = value;
    }

    pub fn set_bool(&mut self, index: usize, value: bool) {
        self.set_u8(index, value as u8);
    }

    pub fn set_i16(&mut self, index: usize, value: i16) {
        self.check_write(index, 2);
        self.data[index..index + 2].copy_from_slice(&value.to_be_bytes());
    }

    pub fn set_i16_le(&mut self, index: usize, value: i16) {
        self.check_write(index, 2);
        self.data[index..index + 2].copy_from_slice(&value.to_le_bytes());
    }

    pub fn set_i32(&mut self, index: usize, value: i32) {
        self.check_write(index, 4);
        self.data[index..index + 4].copy_from_slice(&value.to_be_bytes());
    }

    pub fn set_i32_le(&mut self, index: usize, value: i32) {
        self.check_write(index, 4);
        self.data[index..index + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn set_i64(&mut self, index: usize, value: i64) {
        self.check_write(index, 8);
        self.data[index..index + 8].copy_from_slice(&value.to_be_bytes());
    }

    pub fn set_i64_le(&mut self, index: usize, value: i64) {
        self.check_write(index, 8);
        self.data[index..index + 8].copy_from_slice(&value.to_le_bytes());
    }

    pub fn set_medium(&mut self, index: usize, value: i32) {
        self.check_write(index, 3);
        self.data[index] = (value >> 16) as u8;
        self.data[index + 1] = (value >> 8) as u8;
        self.data[index + 2] = value as u8;
    }

    pub fn set_medium_le(&mut self, index: usize, value: i32) {
        self.check_write(index, 3);
        self.data[index] = value as u8;
        self.data[index + 1] = (value >> 8) as u8;
        self.data[index + 2] = (value >> 16) as u8;
    }

    pub fn set_zero(&mut self, index: usize, length: usize) {
        self.check_write(index, length);
        self.data[index..index + length].fill(0);
    }

    pub fn set_bytes_from(&mut self, index: usize, source: &[u8]) {
        self.check_write(index, source.len());
        self.data[index..index + source.len()].copy_from_slice(source);
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn copy_range(&self, index: usize, length: usize) -> Self {
        Self::from_bytes(&self.data[index..index + length])
    }

    // ---- relative read ( продвигают readerIndex ) ---------------------

    pub fn read_u8(&mut self) -> u8 {
        let v = self.data[self.reader_index];
        self.reader_index += 1;
        v
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_u8() != 0
    }

    pub fn read_i16(&mut self) -> i16 {
        let v = self.get_i16(self.reader_index);
        self.reader_index += 2;
        v
    }

    pub fn read_u16(&mut self) -> u16 {
        self.read_i16() as u16
    }

    pub fn read_i16_le(&mut self) -> i16 {
        let v = self.get_i16_le(self.reader_index);
        self.reader_index += 2;
        v
    }

    pub fn read_i32(&mut self) -> i32 {
        let v = self.get_i32(self.reader_index);
        self.reader_index += 4;
        v
    }

    pub fn read_i32_le(&mut self) -> i32 {
        let v = self.get_i32_le(self.reader_index);
        self.reader_index += 4;
        v
    }

    pub fn read_u32(&mut self) -> u32 {
        self.read_i32() as u32
    }

    pub fn read_u32_le(&mut self) -> u32 {
        self.read_i32_le() as u32
    }

    pub fn read_medium(&mut self) -> i32 {
        let v = self.get_medium(self.reader_index);
        self.reader_index += 3;
        v
    }

    pub fn read_medium_le(&mut self) -> i32 {
        let v = self.get_medium_le(self.reader_index);
        self.reader_index += 3;
        v
    }

    pub fn read_i64(&mut self) -> i64 {
        let v = self.get_i64(self.reader_index);
        self.reader_index += 8;
        v
    }

    pub fn read_i64_le(&mut self) -> i64 {
        let v = self.get_i64_le(self.reader_index);
        self.reader_index += 8;
        v
    }

    pub fn read_f32(&mut self) -> f32 {
        f32::from_bits(self.read_u32())
    }

    pub fn read_f32_bits(&mut self) -> f32 {
        f32::from_bits(self.read_u32())
    }

    pub fn read_f64(&mut self) -> f64 {
        f64::from_bits(self.read_i64() as u64)
    }

    pub fn read_f64_bits(&mut self) -> f64 {
        f64::from_bits(self.read_i64() as u64)
    }

    pub fn read_u8_array(&mut self, length: usize) -> Vec<u8> {
        let s = self.reader_index;
        self.reader_index += length;
        self.data[s..s + length].to_vec()
    }

    pub fn skip_bytes(&mut self, length: usize) {
        self.reader_index += length;
    }

    // ---- relative write ( продвигают writerIndex ) --------------------

    pub fn ensure_writable(&mut self, size: usize) {
        let needed = self.writer_index + size;
        if needed > self.data.len() {
            self.data.resize(needed, 0);
        }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.ensure_writable(1);
        self.data[self.writer_index] = value;
        self.writer_index += 1;
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(value as u8);
    }

    pub fn write_i16(&mut self, value: i16) {
        self.ensure_writable(2);
        self.data[self.writer_index..self.writer_index + 2].copy_from_slice(&value.to_be_bytes());
        self.writer_index += 2;
    }

    pub fn write_i16_le(&mut self, value: i16) {
        self.ensure_writable(2);
        self.data[self.writer_index..self.writer_index + 2].copy_from_slice(&value.to_le_bytes());
        self.writer_index += 2;
    }

    pub fn write_i32(&mut self, value: i32) {
        self.ensure_writable(4);
        self.data[self.writer_index..self.writer_index + 4].copy_from_slice(&value.to_be_bytes());
        self.writer_index += 4;
    }

    pub fn write_i32_le(&mut self, value: i32) {
        self.ensure_writable(4);
        self.data[self.writer_index..self.writer_index + 4].copy_from_slice(&value.to_le_bytes());
        self.writer_index += 4;
    }

    pub fn write_medium(&mut self, value: i32) {
        self.ensure_writable(3);
        self.data[self.writer_index] = (value >> 16) as u8;
        self.data[self.writer_index + 1] = (value >> 8) as u8;
        self.data[self.writer_index + 2] = value as u8;
        self.writer_index += 3;
    }

    pub fn write_medium_le(&mut self, value: i32) {
        self.ensure_writable(3);
        self.data[self.writer_index] = value as u8;
        self.data[self.writer_index + 1] = (value >> 8) as u8;
        self.data[self.writer_index + 2] = (value >> 16) as u8;
        self.writer_index += 3;
    }

    pub fn write_i64(&mut self, value: i64) {
        self.ensure_writable(8);
        self.data[self.writer_index..self.writer_index + 8].copy_from_slice(&value.to_be_bytes());
        self.writer_index += 8;
    }

    pub fn write_i64_le(&mut self, value: i64) {
        self.ensure_writable(8);
        self.data[self.writer_index..self.writer_index + 8].copy_from_slice(&value.to_le_bytes());
        self.writer_index += 8;
    }

    pub fn write_f32(&mut self, value: f32) {
        self.write_i32(value.to_bits() as i32);
    }

    pub fn write_f64(&mut self, value: f64) {
        self.write_i64(value.to_bits() as i64);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.ensure_writable(bytes.len());
        let end = self.writer_index + bytes.len();
        self.data[self.writer_index..end].copy_from_slice(bytes);
        self.writer_index = end;
    }

    pub fn write_zero(&mut self, length: usize) {
        self.ensure_writable(length);
        let end = self.writer_index + length;
        self.data[self.writer_index..end].fill(0);
        self.writer_index = end;
    }

    // ---- доступ к содержимому ----------------------------------------

    /// Байты [readerIndex, writerIndex) — аналог toString(UTF_8) без charset.
    pub fn readable_bytes_vec(&self) -> Vec<u8> {
        self.data[self.reader_index..self.writer_index].to_vec()
    }

    pub fn copy_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }
}

// ================================================================= FriendlyByteBuf

/// `FriendlyByteBuf` — байтовый буфер с MC-методами.
#[derive(Clone, Debug)]
pub struct FriendlyByteBuf {
    pub source: ByteBuf,
    /// Netty markReaderIndex/markWriterIndex.
    reader_mark: usize,
    writer_mark: usize,
}

impl Default for FriendlyByteBuf {
    fn default() -> Self {
        Self { source: ByteBuf::default(), reader_mark: 0, writer_mark: 0 }
    }
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { source: ByteBuf::with_capacity(capacity), reader_mark: 0, writer_mark: 0 }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self { source: ByteBuf::from_bytes(bytes), reader_mark: 0, writer_mark: bytes.len() }
    }

    pub fn unwrap(&self) -> &ByteBuf {
        &self.source
    }

    // ---- индексы ----------------------------------------------------

    pub fn reader_index(&self) -> usize {
        self.source.reader_index()
    }

    pub fn reader_index_set(&mut self, idx: usize) -> &mut Self {
        self.source.reader_index = idx;
        self
    }

    pub fn writer_index(&self) -> usize {
        self.source.writer_index()
    }

    pub fn writer_index_set(&mut self, idx: usize) -> &mut Self {
        self.source.writer_index = idx;
        self
    }

    pub fn set_index(&mut self, reader_index: usize, writer_index: usize) -> &mut Self {
        self.source.set_index(reader_index, writer_index);
        self
    }

    pub fn readable_bytes(&self) -> usize {
        self.source.readable_bytes()
    }

    pub fn writable_bytes(&self) -> usize {
        self.source.writable_bytes()
    }

    pub fn max_writable_bytes(&self) -> usize {
        self.source.max_writable_bytes()
    }

    pub fn is_readable(&self) -> bool {
        self.source.is_readable()
    }

    pub fn is_readable_n(&self, size: usize) -> bool {
        self.source.is_readable_n(size)
    }

    pub fn is_writable(&self) -> bool {
        self.source.is_writable()
    }

    pub fn is_writable_n(&self, size: usize) -> bool {
        self.source.is_writable_n(size)
    }

    pub fn capacity(&self) -> usize {
        self.source.capacity()
    }

    pub fn clear(&mut self) -> &mut Self {
        self.source.clear();
        self
    }

    pub fn mark_reader_index(&mut self) -> &mut Self {
        self.reader_mark = self.source.reader_index();
        self
    }

    pub fn reset_reader_index(&mut self) -> &mut Self {
        self.source.reset_reader_index(self.reader_mark);
        self
    }

    pub fn mark_writer_index(&mut self) -> &mut Self {
        self.writer_mark = self.source.writer_index();
        self
    }

    pub fn reset_writer_index(&mut self) -> &mut Self {
        self.source.reset_writer_index(self.writer_mark);
        self
    }

    pub fn skip_bytes(&mut self, length: usize) -> &mut Self {
        self.source.skip_bytes(length);
        self
    }

    // ---- примитивы --------------------------------------------------

    pub fn read_bool(&mut self) -> bool {
        self.source.read_bool()
    }

    pub fn write_bool(&mut self, value: bool) -> &mut Self {
        self.source.write_bool(value);
        self
    }

    pub fn read_byte(&mut self) -> i8 {
        self.source.read_u8() as i8
    }

    pub fn read_unsigned_byte(&mut self) -> i32 {
        self.source.read_u8() as i32
    }

    pub fn write_byte(&mut self, value: i32) -> &mut Self {
        self.source.write_u8(value as u8);
        self
    }

    pub fn read_short(&mut self) -> i16 {
        self.source.read_i16()
    }

    pub fn read_short_le(&mut self) -> i16 {
        self.source.read_i16_le()
    }

    pub fn read_unsigned_short(&mut self) -> i32 {
        self.source.read_i16() as u16 as i32
    }

    pub fn read_unsigned_short_le(&mut self) -> i32 {
        self.source.read_i16_le() as u16 as i32
    }

    pub fn write_short(&mut self, value: i32) -> &mut Self {
        self.source.write_i16(value as i16);
        self
    }

    pub fn write_short_le(&mut self, value: i32) -> &mut Self {
        self.source.write_i16_le(value as i16);
        self
    }

    pub fn read_medium(&mut self) -> i32 {
        self.source.read_medium()
    }

    pub fn read_medium_le(&mut self) -> i32 {
        self.source.read_medium_le()
    }

    pub fn read_unsigned_medium(&mut self) -> i32 {
        self.source.read_medium() & 0xFFFFFF
    }

    pub fn read_unsigned_medium_le(&mut self) -> i32 {
        self.source.read_medium_le() & 0xFFFFFF
    }

    pub fn write_medium(&mut self, value: i32) -> &mut Self {
        self.source.write_medium(value);
        self
    }

    pub fn write_medium_le(&mut self, value: i32) -> &mut Self {
        self.source.write_medium_le(value);
        self
    }

    pub fn read_int(&mut self) -> i32 {
        self.source.read_i32()
    }

    pub fn read_int_le(&mut self) -> i32 {
        self.source.read_i32_le()
    }

    pub fn read_unsigned_int(&mut self) -> i64 {
        self.source.read_i32() as u32 as i64
    }

    pub fn read_unsigned_int_le(&mut self) -> i64 {
        self.source.read_i32_le() as u32 as i64
    }

    pub fn write_int(&mut self, value: i32) -> &mut Self {
        self.source.write_i32(value);
        self
    }

    pub fn write_int_le(&mut self, value: i32) -> &mut Self {
        self.source.write_i32_le(value);
        self
    }

    pub fn read_long(&mut self) -> i64 {
        self.source.read_i64()
    }

    pub fn read_long_le(&mut self) -> i64 {
        self.source.read_i64_le()
    }

    pub fn write_long(&mut self, value: i64) -> &mut Self {
        self.source.write_i64(value);
        self
    }

    pub fn write_long_le(&mut self, value: i64) -> &mut Self {
        self.source.write_i64_le(value);
        self
    }

    pub fn read_char(&mut self) -> i32 {
        self.source.read_i16() as u16 as i32
    }

    pub fn write_char(&mut self, value: i32) -> &mut Self {
        self.source.write_i16(value as i16);
        self
    }

    pub fn read_float(&mut self) -> f32 {
        f32::from_bits(self.source.read_u32())
    }

    pub fn write_float(&mut self, value: f32) -> &mut Self {
        self.source.write_i32(value.to_bits() as i32);
        self
    }

    pub fn read_double(&mut self) -> f64 {
        f64::from_bits(self.source.read_i64() as u64)
    }

    pub fn write_double(&mut self, value: f64) -> &mut Self {
        self.source.write_i64(value.to_bits() as i64);
        self
    }

    // ---- bytes ------------------------------------------------------

    /// `readBytes(length)` — возвращает новый буфер с length байтами.
    pub fn read_bytes(&mut self, length: usize) -> Self {
        let s = self.source.reader_index;
        self.source.reader_index += length;
        Self { source: ByteBuf::from_bytes(&self.source.data[s..s + length]), reader_mark: 0, writer_mark: length }
    }

    /// `readBytes(byte[])` — прочитать в существующий Vec.
    pub fn read_bytes_into(&mut self, destination: &mut [u8]) -> &mut Self {
        let len = destination.len();
        let s = self.source.reader_index;
        self.source.reader_index += len;
        destination.copy_from_slice(&self.source.data[s..s + len]);
        self
    }

    /// `readBytes(byte[], off, len)`
    pub fn read_bytes_into_at(&mut self, destination: &mut [u8], destination_index: usize, length: usize) -> &mut Self {
        let s = self.source.reader_index;
        self.source.reader_index += length;
        destination[destination_index..destination_index + length]
            .copy_from_slice(&self.source.data[s..s + length]);
        self
    }

    /// Удобный: прочитать N байт в Vec.
    pub fn read_bytes_vec(&mut self, length: usize) -> Vec<u8> {
        let s = self.source.reader_index;
        self.source.reader_index += length;
        self.source.data[s..s + length].to_vec()
    }

    pub fn read_byte_array(&mut self) -> Result<Vec<u8>, CodecError> {
        Self::read_byte_array_n(self, MAX_STRING_LENGTH * 4) // разумный лимит
    }

    /// `readByteArray(maxLength)` — VarInt length + bytes.
    pub fn read_byte_array_n(&mut self, max_length: usize) -> Result<Vec<u8>, CodecError> {
        let i = var_int::read(self);
        if i > max_length as i32 {
            return Err(CodecError(format!("ByteArray with size {} is bigger than allowed {}", i, max_length)));
        }
        if i < 0 {
            return Err(CodecError("ByteArray with negative size".into()));
        }
        Ok(self.read_bytes_vec(i as usize))
    }

    pub fn write_byte_array(&mut self, array: &[u8]) -> &mut Self {
        var_int::write(self, array.len() as i32);
        self.write_bytes(array);
        self
    }

    /// `readVarIntArray`
    pub fn read_var_int_array(&mut self) -> Vec<i32> {
        self.read_var_int_array_n(MAX_STRING_LENGTH)
    }

    pub fn read_var_int_array_n(&mut self, max_length: usize) -> Vec<i32> {
        let i = var_int::read(self);
        assert!(i <= max_length as i32, "VarIntArray with size {} is bigger than allowed {}", i, max_length);
        (0..i).map(|_| var_int::read(self)).collect()
    }

    pub fn write_var_int_array(&mut self, array: &[i32]) -> &mut Self {
        var_int::write(self, array.len() as i32);
        for &v in array {
            var_int::write(self, v);
        }
        self
    }

    /// `writeFixedSizeLongArray` — без длины.
    pub fn write_fixed_size_long_array(&mut self, array: &[i64]) -> &mut Self {
        for &v in array {
            self.source.write_i64(v);
        }
        self
    }

    pub fn write_long_array(&mut self, array: &[i64]) -> &mut Self {
        var_int::write(self, array.len() as i32);
        self.write_fixed_size_long_array(array);
        self
    }

    /// `readFixedSizeLongArray(long[])` — читает в существующий срез.
    pub fn read_fixed_size_long_array_into(&mut self, output: &mut [i64]) -> &mut Self {
        for v in output.iter_mut() {
            *v = self.source.read_i64();
        }
        self
    }

    pub fn read_fixed_size_long_array(&mut self, size: usize) -> Vec<i64> {
        let mut out = vec![0i64; size];
        self.read_fixed_size_long_array_into(&mut out);
        out
    }

    /// `readLongArray` — VarInt count + longs.
    pub fn read_long_array(&mut self) -> Vec<i64> {
        let i = var_int::read(self);
        (0..i).map(|_| self.source.read_i64()).collect()
    }

    /// `writeBytes(ByteBuf)` / `writeBytes(byte[])`
    pub fn write_bytes(&mut self, source: &[u8]) -> &mut Self {
        self.source.write_bytes(source);
        self
    }

    pub fn write_bytes_from_buf(&mut self, source: &ByteBuf) -> &mut Self {
        let bytes = source.readable_bytes_vec();
        self.source.write_bytes(&bytes);
        self
    }

    pub fn write_zero(&mut self, length: usize) -> &mut Self {
        self.source.write_zero(length);
        self
    }

    // ---- VarInt/VarLong ----------------------------------------------

    pub fn read_var_int(&mut self) -> i32 {
        var_int::read(self)
    }

    pub fn write_var_int(&mut self, input: i32) -> &mut Self {
        var_int::write(self, input);
        self
    }

    pub fn read_var_long(&mut self) -> i64 {
        var_long::read(self)
    }

    pub fn write_var_long(&mut self, value: i64) -> &mut Self {
        var_long::write(self, value);
        self
    }

    /// `readContainerId`/`writeContainerId` — VarInt.
    pub fn read_container_id(&mut self) -> i32 {
        var_int::read(self)
    }

    pub fn write_container_id(&mut self, container_id: i32) -> &mut Self {
        var_int::write(self, container_id);
        self
    }

    // ---- позиции/векторы --------------------------------------------

    /// `readBlockPos` — long packed.
    pub fn read_block_pos(&mut self) -> BlockPos {
        BlockPos::of(self.source.read_i64())
    }

    pub fn write_block_pos(&mut self, pos: &BlockPos) -> &mut Self {
        self.source.write_i64(pos.as_long());
        self
    }

    /// `readSectionPos`-эквивалент: packed section long.
    pub fn read_section_pos(&mut self) -> SectionPos {
        SectionPos::of_packed(self.source.read_i64())
    }

    pub fn write_section_pos(&mut self, pos: &SectionPos) -> &mut Self {
        self.source.write_i64(pos.as_long());
        self
    }

    pub fn read_chunk_pos(&mut self) -> ChunkPos {
        ChunkPos::of(self.source.read_i64())
    }

    pub fn write_chunk_pos(&mut self, chunk_pos: &ChunkPos) -> &mut Self {
        self.source.write_i64(chunk_pos.as_long());
        self
    }

    pub fn read_global_pos(&mut self) -> GlobalPos {
        let resource_key = self.read_resource_key_for::<Level>(
            crate::core::registry::registries::dimension(),
        );
        let pos = self.read_block_pos();
        GlobalPos::of(resource_key, pos)
    }

    pub fn write_global_pos(&mut self, pos: &GlobalPos) -> &mut Self {
        self.write_resource_key(&pos.dimension);
        self.write_block_pos(&pos.pos);
        self
    }

    pub fn read_vec3(&mut self) -> Vec3 {
        Vec3::new(self.source.read_f64_bits(), self.source.read_f64_bits(), self.source.read_f64_bits())
    }

    pub fn write_vec3(&mut self, vec3: &Vec3) -> &mut Self {
        self.source.write_f64(vec3.x);
        self.source.write_f64(vec3.y);
        self.source.write_f64(vec3.z);
        self
    }

    pub fn read_lp_vec3(&mut self) -> Vec3 {
        crate::network::lp_vec3::read(self)
    }

    pub fn write_lp_vec3(&mut self, vec3: &Vec3) -> &mut Self {
        crate::network::lp_vec3::write(self, vec3);
        self
    }

    pub fn read_vector3f(&mut self) -> crate::joml::Vector3f {
        crate::joml::Vector3f::new(self.source.read_f32_bits(), self.source.read_f32_bits(), self.source.read_f32_bits())
    }

    pub fn write_vector3f(&mut self, v: &crate::joml::Vector3f) -> &mut Self {
        self.source.write_f32(v.x);
        self.source.write_f32(v.y);
        self.source.write_f32(v.z);
        self
    }

    pub fn read_quaternion(&mut self) -> crate::joml::Quaternionf {
        crate::joml::Quaternionf::new(
            self.source.read_f32_bits(),
            self.source.read_f32_bits(),
            self.source.read_f32_bits(),
            self.source.read_f32_bits(),
        )
    }

    pub fn write_quaternion(&mut self, q: &crate::joml::Quaternionf) -> &mut Self {
        self.source.write_f32(q.x);
        self.source.write_f32(q.y);
        self.source.write_f32(q.z);
        self.source.write_f32(q.w);
        self
    }

    // ---- enum/byId ---------------------------------------------------

    /// `readEnum` — VarInt ordinal.
    pub fn read_enum<T, F>(&mut self, values: &[T], from_i32: F) -> T
    where
        F: Fn(i32) -> Option<T>,
    {
        let i = self.read_var_int();
        let _ = values;
        from_i32(i).expect("Invalid enum id")
    }

    /// `writeEnum` — VarInt ordinal.
    pub fn write_enum<T>(&mut self, _value: &T, ordinal: i32) -> &mut Self {
        self.write_var_int(ordinal);
        self
    }

    /// `readById` — VarInt + lookup.
    pub fn read_by_id<T>(&mut self, id_lookup: &dyn Fn(i32) -> Option<T>) -> T {
        let id = self.read_var_int();
        id_lookup(id).expect("Unknown id")
    }

    /// `writeById`
    pub fn write_by_id<T>(&mut self, value: &T, id_getter: impl Fn(&T) -> i32) -> &mut Self {
        self.write_var_int(id_getter(value));
        self
    }

    /// `readIntIdList` — VarInt count + VarInt ids.
    pub fn read_int_id_list(&mut self) -> Vec<i32> {
        let var_int = self.read_var_int();
        (0..var_int).map(|_| self.read_var_int()).collect()
    }

    pub fn write_int_id_list(&mut self, list: &[i32]) -> &mut Self {
        self.write_var_int(list.len() as i32);
        for &v in list {
            self.write_var_int(v);
        }
        self
    }

    // ---- UUID --------------------------------------------------------

    pub fn read_uuid(&mut self) -> crate::util::uuid::Uuid {
        crate::util::uuid::Uuid::new(self.source.read_i64(), self.source.read_i64())
    }

    pub fn write_uuid(&mut self, uuid: &crate::util::uuid::Uuid) -> &mut Self {
        self.source.write_i64(uuid.most_significant_bits);
        self.source.write_i64(uuid.least_significant_bits);
        self
    }

    // ---- NBT ---------------------------------------------------------

    /// `writeNbt` — null → EndTag.
    pub fn write_nbt(&mut self, tag: Option<&Tag>) -> Result<&mut Self, CodecError> {
        let tag = tag.cloned().unwrap_or(Tag::End);
        let bytes = nbt_io::write_any_tag(&tag).map_err(|e| CodecError(format!("NBT write failed: {e}")))?;
        self.source.write_bytes(&bytes);
        Ok(self)
    }

    /// `readNbt` → Option<Tag> (None при End).
    pub fn read_nbt(&mut self) -> Result<Option<Tag>, CodecError> {
        self.read_nbt_with_acc(&mut NbtAccounter::default_quota())
    }

    pub fn read_nbt_with_acc(&mut self, acc: &mut NbtAccounter) -> Result<Option<Tag>, CodecError> {
        let remaining = self.source.data[self.source.reader_index..self.source.writer_index].to_vec();
        let mut reader = nbt_io::Reader::new(&remaining);
        let tag = nbt_io::read_any_tag(&mut reader, acc).map_err(|e| CodecError(format!("NBT read failed: {e}")))?;
        let consumed = reader.position();
        self.source.reader_index += consumed;
        if matches!(tag, Tag::End) {
            Ok(None)
        } else {
            Ok(Some(tag))
        }
    }

    /// `readNbt` → CompoundTag (ошибка если не compound).
    pub fn read_nbt_compound(&mut self) -> Result<Option<CompoundTag>, CodecError> {
        match self.read_nbt()? {
            None => Ok(None),
            Some(Tag::Compound(c)) => Ok(Some(c)),
            Some(t) => Err(CodecError(format!("Not a compound tag: {}", t.type_name()))),
        }
    }

    /// `writeNbt` для CompoundTag.
    pub fn write_nbt_compound(&mut self, tag: Option<&CompoundTag>) -> Result<&mut Self, CodecError> {
        match tag {
            None => self.write_nbt(Some(&Tag::End)),
            Some(c) => self.write_nbt(Some(&Tag::Compound(c.clone()))),
        }
    }

    // ---- строки ------------------------------------------------------

    /// `readUtf()` — MAX_STRING_LENGTH.
    pub fn read_utf(&mut self) -> Result<String, CodecError> {
        self.read_utf_n(MAX_STRING_LENGTH)
    }

    pub fn read_utf_n(&mut self, max_length: usize) -> Result<String, CodecError> {
        utf8_string::read(self, max_length).map_err(Into::into)
    }

    pub fn write_utf(&mut self, string: &str) -> Result<&mut Self, CodecError> {
        self.write_utf_n(string, MAX_STRING_LENGTH)
    }

    pub fn write_utf_n(&mut self, string: &str, max_length: usize) -> Result<&mut Self, CodecError> {
        utf8_string::write(self, string, max_length)?;
        Ok(self)
    }

    /// `readUtfWithAllWhitespaceRemoved`? — не портировано (используется редко).

    /// `readIdentifier`
    pub fn read_identifier(&mut self) -> Result<Identifier, CodecError> {
        let s = self.read_utf()?;
        Identifier::parse(&s).map_err(|e| CodecError(e.to_string()))
    }

    pub fn write_identifier(&mut self, id: &Identifier) -> Result<&mut Self, CodecError> {
        self.write_utf(&id.to_string())
    }

    /// `readResourceKey` с явным именем реестра.
    pub fn read_resource_key(&mut self, registry: Identifier) -> ResourceKey<Level> {
        let location = self.read_identifier().expect("invalid identifier");
        ResourceKey::create_with_registry_name(registry, location)
    }

    /// Общая версия для любого T.
    pub fn read_resource_key_for<T>(&mut self, registry: Identifier) -> ResourceKey<T> {
        let location = self.read_identifier().expect("invalid identifier");
        ResourceKey::create_with_registry_name(registry, location)
    }

    /// `writeResourceKey` — пишет identifier.
    pub fn write_resource_key<T>(&mut self, key: &ResourceKey<T>) -> Result<&mut Self, CodecError> {
        self.write_identifier(&key.identifier)
    }

    /// `readRegistryKey` → ResourceKey<Registry<T>>.
    pub fn read_registry_key<T>(&mut self) -> ResourceKey<Registry<T>> {
        let id = self.read_identifier().expect("invalid identifier");
        ResourceKey::create_registry_key(id)
    }

    /// `readInstant` — epoch milli long.
    pub fn read_instant(&mut self) -> i64 {
        self.source.read_i64()
    }

    pub fn write_instant(&mut self, epoch_milli: i64) -> &mut Self {
        self.source.write_i64(epoch_milli);
        self
    }

    /// `readBlockHitResult` / `writeBlockHitResult`.
    pub fn read_block_hit_result(&mut self) -> BlockHitResult {
        let block_pos = self.read_block_pos();
        let direction = self.read_direction();
        let fx = self.source.read_f32_bits() as f64;
        let fy = self.source.read_f32_bits() as f64;
        let fz = self.source.read_f32_bits() as f64;
        let inside = self.source.read_bool();
        let world_border = self.source.read_bool();
        BlockHitResult::new_border(
            Vec3::new(block_pos.x as f64 + fx, block_pos.y as f64 + fy, block_pos.z as f64 + fz),
            direction,
            block_pos,
            inside,
            world_border,
        )
    }

    pub fn write_block_hit_result(&mut self, result: &BlockHitResult) -> &mut Self {
        let block_pos = result.block_pos;
        self.write_block_pos(&block_pos);
        self.write_direction(&result.direction);
        let loc = &result.location;
        self.source.write_f32((loc.x - block_pos.x as f64) as f32);
        self.source.write_f32((loc.y - block_pos.y as f64) as f32);
        self.source.write_f32((loc.z - block_pos.z as f64) as f32);
        self.source.write_bool(result.inside);
        self.source.write_bool(result.world_border_hit);
        self
    }

    /// `readDirection` — helper (направление через enum id).
    pub fn read_direction(&mut self) -> Direction {
        Direction::from_3d_data_value(self.read_var_int())
    }

    pub fn write_direction(&mut self, direction: &Direction) -> &mut Self {
        self.write_var_int(direction.get_3d_data_value());
        self
    }

    // ---- PublicKey (Crypt.byteToPublicKey / X509 SPKI DER) ----------

    /// `readPublicKey` — byte[] (max 512) → X509 RSA public key.
    pub fn read_public_key(&mut self) -> Result<rsa::RsaPublicKey, CodecError> {
        let bytes = self.read_byte_array_n(512)?;
        crate::network::encryption::crypt::byte_to_public_key(&bytes)
            .map_err(|e| CodecError(e.to_string()))
    }

    /// `writePublicKey` — `publicKey.getEncoded()` (X509 SPKI DER).
    pub fn write_public_key(&mut self, key: &rsa::RsaPublicKey) -> Result<&mut Self, CodecError> {
        use rsa::pkcs8::EncodePublicKey;
        let der = key
            .to_public_key_der()
            .map_err(|e| CodecError(e.to_string()))?;
        self.write_byte_array(der.as_bytes());
        Ok(self)
    }

    // ---- BitSet ------------------------------------------------------

    /// `readBitSet` — long array → bitset (Vec<u64>).
    pub fn read_bit_set(&mut self) -> Vec<i64> {
        self.read_long_array()
    }

    pub fn write_bit_set(&mut self, bit_set: &[i64]) -> &mut Self {
        self.write_long_array(bit_set);
        self
    }

    /// `readFixedBitSet(size)`
    pub fn read_fixed_bit_set(&mut self, size: usize) -> Vec<i64> {
        let nbytes = mth::positive_ceil_div(size as i32, 8) as usize;
        let bytes = self.read_bytes_vec(nbytes);
        // bytes → little-endian u64s (Java BitSet.valueOf(byte[]) — little-endian)
        let mut words = vec![0i64; (size + 63) / 64];
        for (i, chunk) in bytes.chunks(8).enumerate() {
            if i < words.len() {
                let mut buf = [0u8; 8];
                buf[..chunk.len()].copy_from_slice(chunk);
                words[i] = i64::from_le_bytes(buf);
            }
        }
        words
    }

    /// `writeFixedBitSet`
    pub fn write_fixed_bit_set(&mut self, bit_set: &[u64], size: usize) -> Result<&mut Self, CodecError> {
        // bitset.length() > size → ошибка; здесь считаем used bits
        let used = bit_set
            .iter()
            .rposition(|&w| w != 0)
            .map(|i| i * 64 + (64 - (bit_set[i] as u64).leading_zeros() as usize))
            .unwrap_or(0);
        if used > size {
            return Err(CodecError(format!("BitSet is larger than expected size ({used}>{size})")));
        }
        let nbytes = mth::positive_ceil_div(size as i32, 8) as usize;
        let mut bytes = vec![0u8; nbytes];
        for (i, &w) in bit_set.iter().enumerate() {
            let start = i * 8;
            if start >= nbytes { break; }
            let le = w.to_le_bytes();
            let take = (nbytes - start).min(8);
            bytes[start..start + take].copy_from_slice(&le[..take]);
        }
        self.write_bytes(&bytes);
        Ok(self)
    }

    // ---- коллекции ---------------------------------------------------

    /// `readCollection` — VarInt count + элементы через decoder.
    pub fn read_collection<T>(
        &mut self,
        factory: impl Fn(usize) -> Vec<T>,
        reader: impl Fn(&mut Self) -> T,
    ) -> Vec<T> {
        let i = self.read_var_int();
        let mut out = factory(i.max(0) as usize);
        for _ in 0..i {
            out.push(reader(self));
        }
        out
    }

    /// `writeCollection`
    pub fn write_collection<T>(&mut self, collection: &[T], writer: impl Fn(&mut Self, &T)) -> &mut Self {
        self.write_var_int(collection.len() as i32);
        for item in collection {
            writer(self, item);
        }
        self
    }

    /// `readList`
    pub fn read_list<T>(&mut self, reader: impl Fn(&mut Self) -> T) -> Vec<T> {
        self.read_collection(Vec::with_capacity, reader)
    }

    /// `readMap`
    pub fn read_map<K, V>(
        &mut self,
        map_factory: impl Fn(usize) -> HashMap<K, V>,
        key_reader: impl Fn(&mut Self) -> K,
        value_reader: impl Fn(&mut Self) -> V,
    ) -> HashMap<K, V>
    where
        K: Eq + std::hash::Hash,
    {
        let i = self.read_var_int();
        let mut map = map_factory(i.max(0) as usize);
        for _ in 0..i {
            let k = key_reader(self);
            let v = value_reader(self);
            map.insert(k, v);
        }
        map
    }

    /// `writeMap`
    pub fn write_map<K, V>(&mut self, map: &HashMap<K, V>, key_writer: impl Fn(&mut Self, &K), value_writer: impl Fn(&mut Self, &V)) -> &mut Self
    where
        K: Eq + std::hash::Hash,
    {
        self.write_var_int(map.len() as i32);
        for (k, v) in map {
            key_writer(self, k);
            value_writer(self, v);
        }
        self
    }

    /// `readWithCount`
    pub fn read_with_count(&mut self, reader: impl Fn(&mut Self)) -> &mut Self {
        let i = self.read_var_int();
        for _ in 0..i {
            reader(self);
        }
        self
    }

    /// `writeEnumSet` — fixed bitset по ordinal.
    pub fn write_enum_set<T>(&mut self, values: &[T], set: &HashSet<usize>) -> Result<&mut Self, CodecError> {
        let mut words = vec![0u64; (values.len() + 63) / 64];
        for &i in set {
            if i < values.len() {
                words[i / 64] |= 1u64 << (i % 64);
            }
        }
        self.write_fixed_bit_set(&words, values.len())
    }

    /// `readEnumSet`
    pub fn read_enum_set<T>(&mut self, values: &[T]) -> HashSet<usize> {
        let words = self.read_fixed_bit_set(values.len());
        let mut set = HashSet::new();
        for i in 0..values.len() {
            if (words[i / 64] >> (i % 64)) & 1 == 1 {
                set.insert(i);
            }
        }
        set
    }

    /// `writeOptional` / `readOptional`
    pub fn write_optional<T>(&mut self, optional: &Option<T>, writer: impl Fn(&mut Self, &T)) -> &mut Self {
        match optional {
            Some(v) => {
                self.write_bool(true);
                writer(self, v);
            }
            None => {
                self.write_bool(false);
            }
        }
        self
    }

    pub fn read_optional<T>(&mut self, reader: impl Fn(&mut Self) -> T) -> Option<T> {
        if self.source.read_bool() {
            Some(reader(self))
        } else {
            None
        }
    }

    /// `readNullable`/`writeNullable`
    pub fn read_nullable<T>(&mut self, reader: impl Fn(&mut Self) -> T) -> Option<T> {
        self.read_optional(reader)
    }

    pub fn write_nullable<T>(&mut self, value: &Option<T>, writer: impl Fn(&mut Self, &T)) -> &mut Self {
        self.write_optional(value, writer)
    }

    /// `readEither`/`writeEither`
    pub fn read_either<L, R>(&mut self, left_reader: impl Fn(&mut Self) -> L, right_reader: impl Fn(&mut Self) -> R) -> crate::util::either::Either<L, R> {
        if self.source.read_bool() {
            crate::util::either::Either::Left(left_reader(self))
        } else {
            crate::util::either::Either::Right(right_reader(self))
        }
    }

    pub fn write_either<L, R>(&mut self, value: &crate::util::either::Either<L, R>, left_writer: impl Fn(&mut Self, &L), right_writer: impl Fn(&mut Self, &R)) -> &mut Self {
        match value {
            crate::util::either::Either::Left(l) => {
                self.write_bool(true);
                left_writer(self, l);
            }
            crate::util::either::Either::Right(r) => {
                self.write_bool(false);
                right_writer(self, r);
            }
        }
        self
    }

    // ---- доступ -------------------------------------------------------

    pub fn copy_bytes(&self) -> Vec<u8> {
        self.source.copy_bytes()
    }

    pub fn to_string_utf8(&self) -> String {
        String::from_utf8_lossy(&self.source.data).into_owned()
    }

    /// Байты [readerIndex, writerIndex).
    pub fn readable_bytes_vec(&self) -> Vec<u8> {
        self.source.readable_bytes_vec()
    }
}

// reader_mark/writer_mark — Netty markReaderIndex хранит позицию;
// у нас храним последние mark'и.
// (поля добавлены через отдельную impl-блокировку ниже)

impl FriendlyByteBuf {
    // Эти поля нельзя добавить вне структуры — см. определение выше.
}
