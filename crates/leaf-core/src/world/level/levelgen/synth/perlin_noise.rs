//! `net.minecraft.world.level.levelgen.synth.PerlinNoise`.

use super::improved_noise::ImprovedNoise;
use crate::random::{PositionalRandomFactory, RandomSource};
use std::collections::BTreeSet;

const ROUND_OFF: f64 = 33554432.0;

/// `PerlinNoise` — октавный шум из `ImprovedNoise`.
#[derive(Clone)]
pub struct PerlinNoise {
    /// `noiseLevels` — по одному на октаву (None для нулевой амплитуды).
    noise_levels: Vec<Option<ImprovedNoise>>,
    /// `firstOctave`.
    first_octave: i32,
    /// `amplitudes`.
    amplitudes: Vec<f64>,
    lowest_freq_input_factor: f64,
    lowest_freq_value_factor: f64,
    max_value: f64,
}

impl PerlinNoise {
    /// `makeAmplitudes` — BTreeSet октет → (firstOctave, amplitudes).
    fn make_amplitudes(octaves: BTreeSet<i32>) -> (i32, Vec<f64>) {
        assert!(!octaves.is_empty(), "Need some octaves!");
        let i = -*octaves.iter().next().unwrap();
        let i1 = *octaves.iter().next_back().unwrap();
        let i2 = i + i1 + 1;
        assert!(i2 >= 1, "Total number of octaves needs to be >= 1");
        let mut list = vec![0.0f64; i2 as usize];
        for &o in &octaves {
            list[(o + i) as usize] = 1.0;
        }
        (-i, list)
    }

    /// `PerlinNoise(random, octaves, useNewFactory)` — protected-конструктор.
    fn new(
        random: &mut dyn RandomSource,
        first_octave: i32,
        amplitudes: Vec<f64>,
        use_new_factory: bool,
    ) -> Self {
        let size = amplitudes.len();
        let i = -first_octave;
        let mut noise_levels: Vec<Option<ImprovedNoise>> = vec![None; size];
        if use_new_factory {
            let mut positional = random.fork_positional();
            for i1 in 0..size {
                if amplitudes[i1] != 0.0 {
                    let i2 = first_octave + i1 as i32;
                    let mut rs = positional.from_hash_of(&format!("octave_{}", i2));
                    noise_levels[i1] = Some(ImprovedNoise::new(&mut *rs));
                }
            }
        } else {
            let mut improved_noise = ImprovedNoise::new(random);
            let ii = i;
            if ii >= 0 && ii < size as i32 {
                if amplitudes[ii as usize] != 0.0 {
                    noise_levels[ii as usize] = Some(improved_noise.clone());
                }
            }
            let mut i1x = ii - 1;
            while i1x >= 0 {
                if i1x < size as i32 {
                    if amplitudes[i1x as usize] != 0.0 {
                        noise_levels[i1x as usize] = Some(ImprovedNoise::new(random));
                    } else {
                        Self::skip_octave(random);
                    }
                } else {
                    Self::skip_octave(random);
                }
                i1x -= 1;
            }
            let non_null = noise_levels.iter().filter(|n| n.is_some()).count();
            let non_zero = amplitudes.iter().filter(|a| **a != 0.0).count();
            assert_eq!(non_null, non_zero, "Failed to create correct number of noise levels");
            assert!(ii < size as i32 - 1, "Positive octaves are temporarily disabled");
        }
        let lowest_freq_input_factor = 2.0f64.powi(-i);
        let lowest_freq_value_factor = 2.0f64.powi(size as i32 - 1) / (2.0f64.powi(size as i32) - 1.0);
        let mut noise = Self {
            noise_levels,
            first_octave,
            amplitudes,
            lowest_freq_input_factor,
            lowest_freq_value_factor,
            max_value: 0.0,
        };
        noise.max_value = noise.edge_value(2.0);
        noise
    }

    /// `createLegacyForBlendedNoise`.
    pub fn create_legacy_for_blended_noise(random: &mut dyn RandomSource, octaves: impl IntoIterator<Item = i32>) -> Self {
        let set: BTreeSet<i32> = octaves.into_iter().collect();
        let (first, amps) = Self::make_amplitudes(set);
        Self::new(random, first, amps, false)
    }

    /// `createLegacyForLegacyNetherBiome`.
    pub fn create_legacy_for_legacy_nether_biome(random: &mut dyn RandomSource, first_octave: i32, amplitudes: Vec<f64>) -> Self {
        Self::new(random, first_octave, amplitudes, false)
    }

    /// `create(random, octaves)` — новый фабричный путь.
    pub fn create(random: &mut dyn RandomSource, octaves: impl IntoIterator<Item = i32>) -> Self {
        let set: BTreeSet<i32> = octaves.into_iter().collect();
        let (first, amps) = Self::make_amplitudes(set);
        Self::new(random, first, amps, true)
    }

    /// `create(random, firstOctave, firstAmplitude, amplitudes...)`.
    pub fn create_with_amplitudes(random: &mut dyn RandomSource, first_octave: i32, first_amplitude: f64, amplitudes: &[f64]) -> Self {
        let mut list = amplitudes.to_vec();
        list.insert(0, first_amplitude);
        Self::new(random, first_octave, list, true)
    }

    /// `create(random, firstOctave, amplitudes)`.
    pub fn create_from_list(random: &mut dyn RandomSource, first_octave: i32, amplitudes: Vec<f64>) -> Self {
        Self::new(random, first_octave, amplitudes, true)
    }

    fn skip_octave(random: &mut dyn RandomSource) {
        random.consume_count(262);
    }

    /// `maxValue`.
    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    /// `getValue(x, y, z)`.
    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut d = 0.0;
        let mut e = self.lowest_freq_input_factor;
        let mut f = self.lowest_freq_value_factor;
        for (i, level) in self.noise_levels.iter().enumerate() {
            if let Some(perlin) = level {
                let g = perlin.noise(Self::wrap(x * e), Self::wrap(y * e), Self::wrap(z * e));
                d += self.amplitudes[i] * g * f;
            }
            e *= 2.0;
            f /= 2.0;
        }
        d
    }

    /// Deprecated `getValue` с yScale/yMax.
    pub fn get_value_with_yscale(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64, use_fixed_y: bool) -> f64 {
        let mut d = 0.0;
        let mut d1 = self.lowest_freq_input_factor;
        let mut d2 = self.lowest_freq_value_factor;
        for (i, level) in self.noise_levels.iter().enumerate() {
            if let Some(improved) = level {
                let d3 = improved.noise_with_yscale(
                    Self::wrap(x * d1),
                    if use_fixed_y { -improved.yo } else { Self::wrap(y * d1) },
                    Self::wrap(z * d1),
                    y_scale * d1,
                    y_max * d1,
                );
                d += self.amplitudes[i] * d3 * d2;
            }
            d1 *= 2.0;
            d2 /= 2.0;
        }
        d
    }

    /// `maxBrokenValue`.
    pub fn max_broken_value(&self, y_multiplier: f64) -> f64 {
        self.edge_value(y_multiplier + 2.0)
    }

    fn edge_value(&self, multiplier: f64) -> f64 {
        let mut d = 0.0;
        let mut d1 = self.lowest_freq_value_factor;
        for (i, level) in self.noise_levels.iter().enumerate() {
            if level.is_some() {
                d += self.amplitudes[i] * multiplier * d1;
            }
            d1 /= 2.0;
        }
        d
    }

    /// `getOctaveNoise`.
    pub fn get_octave_noise(&self, octave: usize) -> Option<&ImprovedNoise> {
        self.noise_levels.get(self.noise_levels.len() - 1 - octave).and_then(|o| o.as_ref())
    }

    /// `wrap`.
    pub fn wrap(value: f64) -> f64 {
        value - (value / ROUND_OFF + 0.5).floor() * ROUND_OFF
    }

    /// `firstOctave`.
    pub fn first_octave(&self) -> i32 {
        self.first_octave
    }

    /// `amplitudes`.
    pub fn amplitudes(&self) -> &[f64] {
        &self.amplitudes
    }

    /// `parityConfigString`.
    pub fn parity_config_string(&self, builder: &mut String) {
        builder.push_str("PerlinNoise{first octave: ");
        builder.push_str(&self.first_octave.to_string());
        builder.push_str(", amplitudes: [");
        for (i, a) in self.amplitudes.iter().enumerate() {
            if i > 0 {
                builder.push_str(", ");
            }
            builder.push_str(&format!("{:.2}", a));
        }
        builder.push_str("], noise levels: [");
        for (i, level) in self.noise_levels.iter().enumerate() {
            builder.push_str(&format!("{}: ", i));
            match level {
                None => builder.push_str("null"),
                Some(n) => n.parity_config_string(builder),
            }
            builder.push_str(", ");
        }
        builder.push_str("]}");
    }
}
