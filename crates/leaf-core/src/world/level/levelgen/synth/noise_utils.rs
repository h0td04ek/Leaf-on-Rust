//! `net.minecraft.world.level.levelgen.synth.NoiseUtils`.

/// `biasTowardsExtreme`.
pub fn bias_towards_extreme(value: f64, bias: f64) -> f64 {
    value + (std::f64::consts::PI * value).sin() * bias / std::f64::consts::PI
}

/// `parityNoiseOctaveConfigString` (byte[] вариант).
pub fn parity_noise_octave_config_string_bytes(
    xo: f64,
    yo: f64,
    zo: f64,
    p: &[i8],
) -> String {
    format!(
        "xo={:.3}, yo={:.3}, zo={:.3}, p0={}, p255={}",
        xo as f32, yo as f32, zo as f32, p[0], p[255]
    )
}

/// `parityNoiseOctaveConfigString` (int[] вариант).
pub fn parity_noise_octave_config_string_ints(
    xo: f64,
    yo: f64,
    zo: f64,
    p: &[i32],
) -> String {
    format!(
        "xo={:.3}, yo={:.3}, zo={:.3}, p0={}, p255={}",
        xo as f32, yo as f32, zo as f32, p[0], p[255]
    )
}
