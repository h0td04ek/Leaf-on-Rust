//! `net.minecraft.world.level.levelgen.synth.NormalNoise` —
//! нормализованный октавный шум (2× PerlinNoise).

use super::perlin_noise::PerlinNoise;
use crate::random::RandomSource;

const INPUT_FACTOR: f64 = 1.0181268882175227;
const TARGET_DEVIATION: f64 = 1.0 / 3.0;

/// `NormalNoise.NoiseParameters` — `(firstOctave, amplitudes)`.
#[derive(Clone, Debug, PartialEq)]
pub struct NoiseParameters {
    /// `firstOctave`.
    pub first_octave: i32,
    /// `amplitudes` — список амплитуд октав.
    pub amplitudes: Vec<f64>,
}

impl NoiseParameters {
    /// `new NoiseParameters(firstOctave, amplitudes)`.
    pub fn new(first_octave: i32, amplitudes: Vec<f64>) -> Self {
        Self { first_octave, amplitudes }
    }

    /// `defaultShift` — `NoiseParameters(-3, 1.0)`.
    pub fn default_shift() -> Self {
        Self { first_octave: -3, amplitudes: vec![1.0] }
    }

    /// `NoiseParameters(firstOctave, amplitude, otherAmplitudes...)`.
    pub fn with_first_amplitude(first_octave: i32, amplitude: f64, other: &[f64]) -> Self {
        let mut amplitudes = other.to_vec();
        amplitudes.insert(0, amplitude);
        Self { first_octave, amplitudes }
    }
}

/// `NormalNoise`.
#[derive(Clone)]
pub struct NormalNoise {
    value_factor: f64,
    first: PerlinNoise,
    second: PerlinNoise,
    max_value: f64,
    parameters: NoiseParameters,
}

impl NormalNoise {
    /// `createLegacyNetherBiome`.
    pub fn create_legacy_nether_biome(random: &mut dyn RandomSource, parameters: NoiseParameters) -> Self {
        Self::new(random, parameters, false)
    }

    /// `create(random, firstOctave, amplitudes...)`.
    pub fn create(random: &mut dyn RandomSource, first_octave: i32, amplitudes: &[f64]) -> Self {
        Self::create_with_params(random, NoiseParameters::new(first_octave, amplitudes.to_vec()))
    }

    /// `create(random, NoiseParameters)`.
    pub fn create_with_params(random: &mut dyn RandomSource, parameters: NoiseParameters) -> Self {
        Self::new(random, parameters, true)
    }

    fn new(random: &mut dyn RandomSource, parameters: NoiseParameters, use_new_factory: bool) -> Self {
        let i = parameters.first_octave;
        let list = parameters.amplitudes.clone();
        let (first, second) = if use_new_factory {
            (
                PerlinNoise::create_from_list(random, i, list.clone()),
                PerlinNoise::create_from_list(random, i, list.clone()),
            )
        } else {
            (
                PerlinNoise::create_legacy_for_legacy_nether_biome(random, i, list.clone()),
                PerlinNoise::create_legacy_for_legacy_nether_biome(random, i, list.clone()),
            )
        };
        let mut i1 = i32::MAX;
        let mut i2 = i32::MIN;
        for (idx, &d) in list.iter().enumerate() {
            if d != 0.0 {
                i1 = i1.min(idx as i32);
                i2 = i2.max(idx as i32);
            }
        }
        let value_factor = (1.0 / 6.0) / Self::expected_deviation(i2 - i1);
        let max_value = (first.max_value() + second.max_value()) * value_factor;
        Self { value_factor, first, second, max_value, parameters }
    }

    fn expected_deviation(octaves: i32) -> f64 {
        let _ = TARGET_DEVIATION;
        0.1 * (1.0 + 1.0 / (octaves as f64 + 1.0))
    }

    /// `maxValue`.
    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    /// `getValue(x, y, z)`.
    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        let d = x * INPUT_FACTOR;
        let d1 = y * INPUT_FACTOR;
        let d2 = z * INPUT_FACTOR;
        (self.first.get_value(x, y, z) + self.second.get_value(d, d1, d2)) * self.value_factor
    }

    /// `parameters`.
    pub fn parameters(&self) -> &NoiseParameters {
        &self.parameters
    }

    /// `parityConfigString`.
    pub fn parity_config_string(&self, builder: &mut String) {
        builder.push_str("NormalNoise {first: ");
        self.first.parity_config_string(builder);
        builder.push_str(", second: ");
        self.second.parity_config_string(builder);
        builder.push_str("}");
    }
}
