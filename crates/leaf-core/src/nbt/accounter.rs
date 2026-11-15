//! Перенос `net.minecraft.nbt.NbtAccounter` — учёт памяти и глубины при чтении NBT.

use std::fmt;

pub const DEFAULT_NBT_QUOTA: i64 = 2_097_152;
pub const UNCOMPRESSED_NBT_QUOTA: i64 = 104_857_600;
const MAX_STACK_DEPTH: usize = 512;

#[derive(Debug)]
pub struct NbtAccounter {
    quota: i64,
    usage: i64,
    max_depth: usize,
    depth: usize,
}

#[derive(Debug)]
pub struct NbtAccounterException(pub String);

impl fmt::Display for NbtAccounterException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for NbtAccounterException {}

impl NbtAccounter {
    pub fn new(quota: i64, max_depth: usize) -> Self {
        Self { quota, usage: 0, max_depth, depth: 0 }
    }

    pub fn create(quota: i64) -> Self {
        Self::new(quota, MAX_STACK_DEPTH)
    }

    pub fn default_quota() -> Self {
        Self::new(DEFAULT_NBT_QUOTA, MAX_STACK_DEPTH)
    }

    pub fn uncompressed_quota() -> Self {
        Self::new(UNCOMPRESSED_NBT_QUOTA, MAX_STACK_DEPTH)
    }

    pub fn unlimited_heap() -> Self {
        Self::new(i64::MAX, MAX_STACK_DEPTH)
    }

    pub fn account_bytes_per_item(&mut self, bytes_per_item: i64, items: i64) -> Result<(), NbtAccounterException> {
        self.account_bytes(bytes_per_item.saturating_mul(items))
    }

    pub fn account_bytes(&mut self, bytes: i64) -> Result<(), NbtAccounterException> {
        if bytes < 0 {
            return Err(NbtAccounterException(format!(
                "Tried to account NBT tag with negative size: {bytes}"
            )));
        }
        if self.usage.saturating_add(bytes) > self.quota {
            return Err(NbtAccounterException(format!(
                "Tried to read NBT tag that was too big; tried to allocate: {} + {bytes} bytes where max allowed: {}",
                self.usage, self.quota
            )));
        }
        self.usage += bytes;
        Ok(())
    }

    pub fn push_depth(&mut self, depth: usize) -> Result<(), NbtAccounterException> {
        if self.depth + depth >= self.max_depth {
            return Err(NbtAccounterException(format!(
                "Tried to read NBT tag with too high complexity, depth > {}",
                self.max_depth
            )));
        }
        self.depth += depth;
        Ok(())
    }

    pub fn push_depth_one(&mut self) -> Result<(), NbtAccounterException> {
        self.push_depth(1)
    }

    pub fn pop_depth(&mut self) -> Result<(), NbtAccounterException> {
        if self.depth == 0 {
            return Err(NbtAccounterException(
                "NBT-Accounter tried to pop stack-depth at top-level".into(),
            ));
        }
        self.depth -= 1;
        Ok(())
    }

    pub fn usage(&self) -> i64 {
        self.usage
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quota() {
        let mut acc = NbtAccounter::create(100);
        assert!(acc.account_bytes(60).is_ok());
        assert!(acc.account_bytes(50).is_err());
        assert_eq!(acc.usage(), 60);
    }

    #[test]
    fn depth() {
        let mut acc = NbtAccounter::create(i64::MAX);
        for _ in 0..511 {
            acc.push_depth_one().unwrap();
        }
        assert!(acc.push_depth_one().is_err());
        acc.pop_depth().unwrap();
        assert!(acc.push_depth_one().is_ok());
    }
}
