//! `net.minecraft.world.level.levelgen.synth.PerlinSimplexNoise`.

use super::simplex_noise::SimplexNoise;
use crate::random::{LegacyRandomSource, RandomSource};
use crate::world::level::levelgen::worldgen_random::WorldgenRandom;
use std::collections::BTreeSet;

/// `PerlinSimplexNoise` — композиция октав `SimplexNoise`.
#[derive(Clone)]
pub struct PerlinSimplexNoise {
    /// `noiseLevels` — Some на каждой октаве из набора.
    noise_levels: Vec<Option<SimplexNoise>>,
    highest_freq_value_factor: f64,
    highest_freq_input_factor: f64,
}

impl PerlinSimplexNoise {
    /// `new PerlinSimplexNoise(random, octaves)`.
    pub fn new(random: &mut dyn RandomSource, octaves: impl IntoIterator<Item = i32>) -> Self {
        let set: BTreeSet<i32> = octaves.into_iter().collect();
        assert!(!set.is_empty(), "Need some octaves!");
        let i = -*set.iter().next().unwrap();
        let i1 = *set.iter().next_back().unwrap();
        let i2 = i + i1 + 1;
        assert!(i2 >= 1, "Total number of octaves needs to be >= 1");

        let simplex_noise = SimplexNoise::new(random);
        let i3 = i1;
        let mut noise_levels: Vec<Option<SimplexNoise>> = vec![None; i2 as usize];
        if i1 >= 0 && i1 < i2 && set.contains(&0) {
            noise_levels[i1 as usize] = Some(simplex_noise.clone());
        }
        let mut i4 = i1 + 1;
        while i4 < i2 {
            if i4 >= 0 && set.contains(&(i3 - i4)) {
                noise_levels[i4 as usize] = Some(SimplexNoise::new(random));
            } else {
                random.consume_count(262);
            }
            i4 += 1;
        }
        if i1 > 0 {
            let l = (simplex_noise.get_value_3d(simplex_noise.xo, simplex_noise.yo, simplex_noise.zo) * 9.223372e18) as i64;
            let mut random_source = WorldgenRandom::new(Box::new(LegacyRandomSource::new(l)));
            let mut i5 = i3 - 1;
            while i5 >= 0 {
                if i5 < i2 && set.contains(&(i3 - i5)) {
                    noise_levels[i5 as usize] = Some(SimplexNoise::new(&mut random_source));
                } else {
                    random_source.consume_count(262);
                }
                i5 -= 1;
            }
        }
        let highest_freq_input_factor = 2.0f64.powi(i1);
        let highest_freq_value_factor = 1.0 / (2.0f64.powi(i2) - 1.0);
        Self { noise_levels, highest_freq_value_factor, highest_freq_input_factor }
    }

    /// `getValue(x, y, useNoiseOffsets)`.
    pub fn get_value(&self, x: f64, y: f64, use_noise_offsets: bool) -> f64 {
        let mut d = 0.0;
        let mut d1 = self.highest_freq_input_factor;
        let mut d2 = self.highest_freq_value_factor;
        for level in &self.noise_levels {
            if let Some(simplex) = level {
                d += simplex.get_value_2d(
                    x * d1 + if use_noise_offsets { simplex.xo } else { 0.0 },
                    y * d1 + if use_noise_offsets { simplex.yo } else { 0.0 },
                ) * d2;
            }
            d1 /= 2.0;
            d2 *= 2.0;
        }
        d
    }
}
