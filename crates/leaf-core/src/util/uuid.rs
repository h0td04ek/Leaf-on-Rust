//! Минимальный `java.util.UUID` — нужен для сетевого протокола.

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

impl Uuid {
    pub const fn new(most_significant_bits: i64, least_significant_bits: i64) -> Self {
        Self { most_significant_bits, least_significant_bits }
    }

    pub const NIL: Uuid = Uuid { most_significant_bits: 0, least_significant_bits: 0 };

    pub fn version(&self) -> i32 {
        ((self.most_significant_bits >> 12) & 0xF) as i32
    }

    pub fn variant(&self) -> i32 {
        ((self.least_significant_bits >> 62) & 0x3) as i32
    }

    /// `UUID.randomUUID` — через RandomSource? Java использует SecureRandom;
    /// здесь — xorshift на системном времени (достаточно для dev).
    pub fn random() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        let mut s = seed ^ 0x9E3779B97F4A7C15;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let mut msb = next() as i64;
        let mut lsb = next() as i64;
        // version 4, variant 10xx
        msb = (msb & !(0xF << 12)) | (4 << 12);
        lsb = (lsb & !(0x3 << 62)) | (0x2 << 62);
        Self::new(msb, lsb)
    }

    /// `UUID.nameUUIDFromBytes` — MD5 → version 3, IETF variant.
    pub fn name_uuid_from_bytes(bytes: &[u8]) -> Self {
        use md5::{Digest, Md5};
        let hash = Md5::digest(bytes);
        let mut msb = 0i64;
        let mut lsb = 0i64;
        for i in 0..8 {
            msb = (msb << 8) | hash[i] as i64;
            lsb = (lsb << 8) | hash[i + 8] as i64;
        }
        // version 3
        let msb = (msb & !(0xF << 12)) | (3 << 12);
        // variant IETF (10xx)
        let lsb = (lsb & !(0x3 << 62)) | (0x2 << 62);
        Self::new(msb, lsb)
    }

    /// `UUIDUtil.createOfflinePlayerUUID` — nameUUIDFromBytes("OfflinePlayer:"+name).
    pub fn create_offline_player_uuid(username: &str) -> Self {
        Self::name_uuid_from_bytes(format!("OfflinePlayer:{username}").as_bytes())
    }

    /// `Util.NIL_UUID`-эквивалент.
    pub fn nil() -> Self {
        Self::NIL
    }

    /// `UUID.fromString` — базовый парсинг.
    pub fn from_string(s: &str) -> Option<Self> {
        let hex: String = s.chars().filter(|&c| c != '-').collect();
        if hex.len() != 32 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let msb = u64::from_str_radix(&hex[..16], 16).ok()? as i64;
        let lsb = u64::from_str_radix(&hex[16..], 16).ok()? as i64;
        Some(Self::new(msb, lsb))
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            (self.most_significant_bits >> 32) as u32,
            (self.most_significant_bits >> 16) as u16,
            (self.most_significant_bits) as u16,
            (self.least_significant_bits >> 48) as u16,
            self.least_significant_bits as u64 & 0xFFFFFFFFFFFF
        )
    }
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uuid({})", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_roundtrip() {
        let u = Uuid::new(0x1234567890ABCDEF, 0x0FEDCBA987654321);
        let s = u.to_string();
        assert_eq!(Uuid::from_string(&s), Some(u));
    }

    #[test]
    fn random_is_v4() {
        let u = Uuid::random();
        assert_eq!(u.version(), 4);
        assert_eq!(u.variant(), 2);
    }
}
