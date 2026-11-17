//! `net.minecraft.world.level.biome.BiomeManager` — поиск биома
//! с "fiddle"-смещением и LCG-джиттером (порт с Leaf-оптимизациями:
//! QUART_OFFSETS, FIDDLE_TABLE, кэш биомов).

use super::biome::Biome;
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::core::quart_pos;
use crate::util::linear_congruential_generator::next as lcg_next;
use crate::util::mth;
use std::sync::Mutex;

/// `CHUNK_CENTER_QUART`.
pub const CHUNK_CENTER_QUART: i32 = quart_pos::from_block(8);
const ZOOM_BITS: i32 = 2;
const ZOOM: i32 = 4;
const ZOOM_MASK: i32 = 3;
/// `MAX_OFFSET` (Carpet-Fixes).
const MAX_OFFSET: f64 = 0.4500000001;
const BIOME_CACHE_SIZE: usize = 65536;

const QUART_OFFSETS: [f64; 4] = [0.0, 0.25, 0.5, 0.75];

fn fiddle_table() -> &'static [f64; 1024] {
    static TABLE: std::sync::OnceLock<[f64; 1024]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut t = [0.0f64; 1024];
        for (i, v) in t.iter_mut().enumerate() {
            *v = (i as f64 - 512.0) * (0.9 / 1024.0);
        }
        t
    })
}

/// `BiomeManager.obfuscateSeed` — Leaf: XXHash64 с ключом / SHA-256 fallback.
/// Здесь — FNV-1a-подобный миксер, эквивалентный по контракту
/// (детерминированный seed→seed); точный XXHash ключ — конфиг Leaf.
pub fn obfuscate_seed(seed: i64) -> i64 {
    // Leaf FastBiomeManagerSeedObfuscation использует XXHash64 с ключом
    // из конфига; портируем xxhash64 с фиксированным ключом 0.
    xxhash64(seed as u64, 0) as i64
}

/// XXHash64 для 8-байтового входа (seed обфускация Leaf).
fn xxhash64(input: u64, seed: u64) -> u64 {
    const P1: u64 = 11400714785074694791;
    const P2: u64 = 14029467366897019727;
    const P3: u64 = 1609587929392839161;
    const P4: u64 = 9650029242287828579;
    const P5: u64 = 2870177450012600261;
    let mut h = seed.wrapping_add(P5).wrapping_add(8);
    h ^= input.wrapping_mul(P2).rotate_left(31).wrapping_mul(P1);
    h = h.rotate_left(27).wrapping_mul(P1).wrapping_add(P4);
    h ^= h >> 33;
    h = h.wrapping_mul(P2).wrapping_add(P3);
    h ^= h >> 29;
    h = h.wrapping_mul(P3).wrapping_add(P4);
    h ^ (h >> 32)
}

/// `getFiddle(seed)` — табличный вариант.
fn get_fiddle(seed: i64) -> f64 {
    fiddle_table()[((seed >> 24) as usize) & 1023]
}

/// `BiomeManager.NoiseBiomeSource`.
pub trait NoiseBiomeSource: Send + Sync {
    /// `getNoiseBiome(x, y, z)` — quart-координаты.
    fn get_noise_biome(&self, x: i32, y: i32, z: i32) -> Holder<Biome>;
}

/// `BiomeManager`.
pub struct BiomeManager<S: NoiseBiomeSource> {
    noise_biome_source: S,
    biome_zoom_seed: i64,
    /// Leaf-кэш: 65536 записей по хешу packedPos.
    biome_cache: Mutex<Option<BiomeCache>>,
}

struct BiomeCache {
    biomes: Vec<Option<Holder<Biome>>>,
    positions: Vec<i64>,
}

impl<S: NoiseBiomeSource> BiomeManager<S> {
    /// `new BiomeManager(noiseBiomeSource, biomeZoomSeed)`.
    pub fn new(noise_biome_source: S, biome_zoom_seed: i64) -> Self {
        Self { noise_biome_source, biome_zoom_seed, biome_cache: Mutex::new(None) }
    }

    /// `withDifferentSource` — возвращает новый менеджер над другим источником.
    pub fn with_different_source<S2: NoiseBiomeSource>(&self, new_source: S2) -> BiomeManager<S2> {
        BiomeManager::new(new_source, self.biome_zoom_seed)
    }

    /// Включить/выключить кэш биомов (Leaf `OptimizeBiome`).
    pub fn set_cache_enabled(&self, enabled: bool) {
        let mut cache = self.biome_cache.lock().unwrap();
        *cache = if enabled {
            Some(BiomeCache {
                biomes: vec![None; BIOME_CACHE_SIZE],
                positions: vec![0; BIOME_CACHE_SIZE],
            })
        } else {
            None
        };
    }

    /// `getBiome(pos)` — публичный вход (block-координаты).
    pub fn get_biome(&self, pos: BlockPos) -> Holder<Biome> {
        self.get_biome_chunk(None, pos.x - 2, pos.y - 2, pos.z - 2)
    }

    /// `getBiomeCached(chunk, pos)` — с Leaf-кэшем.
    pub fn get_biome_cached(&self, pos: BlockPos) -> Holder<Biome> {
        {
            let mut guard = self.biome_cache.lock().unwrap();
            if let Some(cache) = guard.as_mut() {
                let x_minus_2 = pos.x - 2;
                let y_minus_2 = pos.y - 2;
                let z_minus_2 = pos.z - 2;
                let packed_pos = BlockPos::as_long_static(x_minus_2 >> 2, y_minus_2 >> 2, z_minus_2 >> 2);
                let hash = (hash_common_mix(packed_pos) & (BIOME_CACHE_SIZE as i64 - 1)) as usize;
                if cache.positions[hash] == packed_pos {
                    if let Some(biome) = &cache.biomes[hash] {
                        return biome.clone();
                    }
                }
                drop(guard);
                let biome = self.get_biome_chunk(None, x_minus_2, y_minus_2, z_minus_2);
                let mut guard = self.biome_cache.lock().unwrap();
                if let Some(cache) = guard.as_mut() {
                    cache.biomes[hash] = Some(biome.clone());
                    cache.positions[hash] = packed_pos;
                }
                return biome;
            }
        }
        self.get_biome(pos)
    }

    /// `getBiome(chunk, xMinus2, yMinus2, zMinus2)` — ядро алгоритма.
    fn get_biome_chunk(&self, _chunk: Option<&dyn ChunkNoiseBiome>, x_minus_2: i32, y_minus_2: i32, z_minus_2: i32) -> Holder<Biome> {
        let x = x_minus_2 >> ZOOM_BITS;
        let y = y_minus_2 >> ZOOM_BITS;
        let z = z_minus_2 >> ZOOM_BITS;
        let quart_x = QUART_OFFSETS[(x_minus_2 & ZOOM_MASK) as usize];
        let quart_y = QUART_OFFSETS[(y_minus_2 & ZOOM_MASK) as usize];
        let quart_z = QUART_OFFSETS[(z_minus_2 & ZOOM_MASK) as usize];

        let mut smallest_x = 0i32;
        let mut smallest_dist = f64::INFINITY;
        for biome_x in 0..8 {
            let every_other_quad = (biome_x & 4) == 0;
            let every_other_pair = (biome_x & 2) == 0;
            let every_other = (biome_x & 1) == 0;
            let quart_xx = if every_other_quad { quart_x } else { quart_x - 1.0 };
            let quart_yy = if every_other_pair { quart_y } else { quart_y - 1.0 };
            let quart_zz = if every_other { quart_z } else { quart_z - 1.0 };

            let mut min_quart_yy = 0.0f64;
            let mut min_quart_zz = 0.0f64;
            if biome_x != 0 {
                min_quart_yy = mth::square_f64((quart_yy.abs() - MAX_OFFSET).max(0.0));
                min_quart_zz = mth::square_f64((quart_zz.abs() - MAX_OFFSET).max(0.0));
                let min_quart_xx = mth::square_f64((quart_xx.abs() - MAX_OFFSET).max(0.0));
                if smallest_dist < min_quart_xx + min_quart_yy + min_quart_zz {
                    continue;
                }
            }
            let xx = if every_other_quad { x } else { x + 1 };
            let yy = if every_other_pair { y } else { y + 1 };
            let zz = if every_other { z } else { z + 1 };

            let mut seed = lcg_next(self.biome_zoom_seed, xx as i64);
            seed = lcg_next(seed, yy as i64);
            seed = lcg_next(seed, zz as i64);
            seed = lcg_next(seed, xx as i64);
            seed = lcg_next(seed, yy as i64);
            seed = lcg_next(seed, zz as i64);
            let offset_x = get_fiddle(seed);
            let sqr_x = mth::square_f64(quart_xx + offset_x);
            if biome_x != 0 && smallest_dist < sqr_x + min_quart_yy + min_quart_zz {
                continue;
            }
            seed = lcg_next(seed, self.biome_zoom_seed);
            let offset_y = get_fiddle(seed);
            let sqr_y = mth::square_f64(quart_yy + offset_y);
            if biome_x != 0 && smallest_dist < sqr_x + sqr_y + min_quart_zz {
                continue;
            }
            seed = lcg_next(seed, self.biome_zoom_seed);
            let offset_z = get_fiddle(seed);
            let biome_dist = sqr_x + sqr_y + mth::square_f64(quart_zz + offset_z);

            if smallest_dist > biome_dist {
                smallest_x = biome_x;
                smallest_dist = biome_dist;
            }
        }
        let x1 = if (smallest_x & 4) == 0 { x } else { x + 1 };
        let y1 = if (smallest_x & 2) == 0 { y } else { y + 1 };
        let z1 = if (smallest_x & 1) == 0 { z } else { z + 1 };
        if let Some(chunk) = _chunk {
            if chunk.loc_x() >> 2 == x1 >> 2 && chunk.loc_z() >> 2 == z1 >> 2 {
                return chunk.get_noise_biome(x1, y1, z1);
            }
        }
        self.noise_biome_source.get_noise_biome(x1, y1, z1)
    }

    /// `getNoiseBiomeAtPosition(x, y, z)` — double-координаты.
    pub fn get_noise_biome_at_position(&self, x: f64, y: f64, z: f64) -> Holder<Biome> {
        let qx = quart_pos::from_block(mth::floor(x));
        let qy = quart_pos::from_block(mth::floor(y));
        let qz = quart_pos::from_block(mth::floor(z));
        self.get_noise_biome_at_quart(qx, qy, qz)
    }

    /// `getNoiseBiomeAtPosition(pos)`.
    pub fn get_noise_biome_at_block_pos(&self, pos: BlockPos) -> Holder<Biome> {
        self.get_noise_biome_at_quart(
            quart_pos::from_block(pos.x),
            quart_pos::from_block(pos.y),
            quart_pos::from_block(pos.z),
        )
    }

    /// `getNoiseBiomeAtQuart(x, y, z)`.
    pub fn get_noise_biome_at_quart(&self, x: i32, y: i32, z: i32) -> Holder<Biome> {
        self.noise_biome_source.get_noise_biome(x, y, z)
    }

    /// `getNoiseBiomeSource`.
    pub fn noise_biome_source(&self) -> &S {
        &self.noise_biome_source
    }

    /// `getBiomeZoomSeed`.
    pub fn biome_zoom_seed(&self) -> i64 {
        self.biome_zoom_seed
    }
}

/// Тракт для chunk-биомов (LevelChunk.getNoiseBiome).
pub trait ChunkNoiseBiome {
    /// `locX`/`locZ` чанка.
    fn loc_x(&self) -> i32;
    /// `locZ`.
    fn loc_z(&self) -> i32;
    /// `getNoiseBiome(x, y, z)` — quart-координаты внутри чанка.
    fn get_noise_biome(&self, x: i32, y: i32, z: i32) -> Holder<Biome>;
}

/// `it.unimi.dsi.fastutil.HashCommon.mix(long)`.
fn hash_common_mix(x: i64) -> i64 {
    let h = (x as u64).wrapping_mul(0x9E3779B97F4A7C15);
    ((h ^ (h >> 32)) as i64)
}
