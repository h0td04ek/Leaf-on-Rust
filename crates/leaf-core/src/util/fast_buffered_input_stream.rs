//! Перенос `net.minecraft.util.FastBufferedInputStream`.

use std::io::{self, Read};

const DEFAULT_BUFFER_SIZE: usize = 8192;

/// `FastBufferedInputStream` — InputStream с буфером 8192 и fast-path
/// для больших чтений напрямую из источника.
pub struct FastBufferedInputStream<R: Read> {
    inner: R,
    buffer: Vec<u8>,
    limit: usize,
    position: usize,
}

impl<R: Read> FastBufferedInputStream<R> {
    pub fn new(inner: R) -> Self {
        Self::with_buffer_size(inner, DEFAULT_BUFFER_SIZE)
    }

    pub fn with_buffer_size(inner: R, buffer_size: usize) -> Self {
        Self { inner, buffer: vec![0u8; buffer_size], limit: 0, position: 0 }
    }

    /// `bytesInBuffer`.
    fn bytes_in_buffer(&self) -> usize {
        self.limit - self.position
    }

    fn fill(&mut self) -> io::Result<()> {
        self.limit = 0;
        self.position = 0;
        let n = self.inner.read(&mut self.buffer)?;
        if n > 0 {
            self.limit = n;
        }
        Ok(())
    }
}

impl<R: Read> Read for FastBufferedInputStream<R> {
    /// `read()`.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if self.bytes_in_buffer() <= 0 {
            if buf.len() >= self.buffer.len() {
                return self.inner.read(buf);
            }
            self.fill()?;
            if self.bytes_in_buffer() <= 0 {
                return Ok(0);
            }
        }
        let length = buf.len().min(self.bytes_in_buffer());
        buf[..length].copy_from_slice(&self.buffer[self.position..self.position + length]);
        self.position += length;
        Ok(length)
    }
}

impl<R: Read> FastBufferedInputStream<R> {
    /// `read()` без аргументов — один байт или -1.
    pub fn read_byte(&mut self) -> io::Result<i32> {
        if self.position >= self.limit {
            self.fill()?;
            if self.position >= self.limit {
                return Ok(-1);
            }
        }
        let b = self.buffer[self.position];
        self.position += 1;
        Ok(b as i32)
    }

    /// `skip(amount)`.
    pub fn skip_bytes(&mut self, amount: u64) -> io::Result<u64> {
        if amount == 0 {
            return Ok(0);
        }
        let l = self.bytes_in_buffer() as u64;
        if l <= 0 {
            // Пропуск через чтение в буфер (seek может быть недоступен).
            let mut remaining = amount;
            let mut chunk = vec![0u8; self.buffer.len().min(remaining as usize).max(1)];
            while remaining > 0 {
                let want = remaining.min(chunk.len() as u64) as usize;
                match self.inner.read(&mut chunk[..want])? {
                    0 => break,
                    n => remaining -= n as u64,
                }
            }
            return Ok(amount - remaining);
        }
        let amount = amount.min(l);
        self.position += amount as usize;
        Ok(amount)
    }

    /// `available()`.
    pub fn available(&self) -> usize {
        self.bytes_in_buffer()
    }
}
