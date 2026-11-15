//! Перенос `net.minecraft.world.level.levelgen.RandomSupport` —
//! 128-битные сиды и stafford13-миксер.

use std::sync::atomic::{AtomicU64, Ordering};

pub const GOLDEN_RATIO_64: i64 = -7046029254386353131;
pub const SILVER_RATIO_64: i64 = 7640891576956012809;

static SEED_UNIQUIFIER: AtomicU64 = AtomicU64::new(8682522807148012);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seed128bit {
    pub seed_lo: i64,
    pub seed_hi: i64,
}

impl Seed128bit {
    pub fn xor(self, other: Seed128bit) -> Seed128bit {
        Seed128bit {
            seed_lo: self.seed_lo ^ other.seed_lo,
            seed_hi: self.seed_hi ^ other.seed_hi,
        }
    }

    pub fn xor_lo_hi(self, seed_lo: i64, seed_hi: i64) -> Seed128bit {
        Seed128bit {
            seed_lo: self.seed_lo ^ seed_lo,
            seed_hi: self.seed_hi ^ seed_hi,
        }
    }

    pub fn mixed(self) -> Seed128bit {
        Seed128bit {
            seed_lo: mix_stafford13(self.seed_lo),
            seed_hi: mix_stafford13(self.seed_hi),
        }
    }
}

/// `mixStafford13` — миксер.stafford13 (finalizer).
pub fn mix_stafford13(mut seed: i64) -> i64 {
    seed = (seed ^ (seed >> 30)).wrapping_mul(-4658895280553007687);
    seed = (seed ^ (seed >> 27)).wrapping_mul(-7723592293110705685);
    seed ^ (seed >> 31)
}

pub fn upgrade_seed_to_128bit_unmixed(seed: i64) -> Seed128bit {
    let l = seed ^ SILVER_RATIO_64;
    let l1 = l.wrapping_add(GOLDEN_RATIO_64);
    Seed128bit { seed_lo: l, seed_hi: l1 }
}

pub fn upgrade_seed_to_128bit(seed: i64) -> Seed128bit {
    upgrade_seed_to_128bit_unmixed(seed).mixed()
}

/// `seedFromHashOf` — MD5 от UTF-8 строки → два i64 (big-endian).
pub fn seed_from_hash_of(string: &str) -> Seed128bit {
    use md5::{Digest, Md5};
    let hash = Md5::digest(string.as_bytes());
    let mut lo = [0u8; 8];
    let mut hi = [0u8; 8];
    lo.copy_from_slice(&hash[0..8]);
    hi.copy_from_slice(&hash[8..16]);
    Seed128bit {
        seed_lo: i64::from_be_bytes(lo),
        seed_hi: i64::from_be_bytes(hi),
    }
}

/// `generateUniqueSeed` — AtomicLong(unqiufier) * phi + nanoTime.
pub fn generate_unique_seed() -> i64 {
    // AtomicLong.updateAndGet(l -> l * C)
    let old = loop {
        let prev = SEED_UNIQUIFIER.load(Ordering::Relaxed);
        let next = prev.wrapping_mul(1181783497276652981);
        if SEED_UNIQUIFIER
            .compare_exchange(prev, next, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            break next;
        }
    };
    // System.nanoTime() — наносекунды монотонных часов
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    (old as i64) ^ (nanos as i64)
}
