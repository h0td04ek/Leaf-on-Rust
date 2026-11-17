//! Полный порт `net.minecraft.util.ARGB`.
//! `Vector3f`/`Vector4f`/`Vec3` — `crate::joml` / `crate::world::phys`.

use crate::joml::{Vector3f, Vector4f};
use crate::util::mth;
use crate::world::phys::Vec3;
use std::sync::OnceLock;

const LINEAR_CHANNEL_DEPTH: usize = 1024;

fn srgb_to_linear() -> &'static [i16; 256] {
    static T: OnceLock<Box<[i16; 256]>> = OnceLock::new();
    T.get_or_init(|| {
        let mut arr = Box::new([0i16; 256]);
        for (i, v) in arr.iter_mut().enumerate() {
            let f = i as f32 / 255.0;
            *v = java_round_f32(compute_srgb_to_linear(f) * 1023.0) as i16;
        }
        arr
    })
}

fn linear_to_srgb() -> &'static [u8; LINEAR_CHANNEL_DEPTH] {
    static T: OnceLock<Box<[u8; LINEAR_CHANNEL_DEPTH]>> = OnceLock::new();
    T.get_or_init(|| {
        let mut arr = Box::new([0u8; LINEAR_CHANNEL_DEPTH]);
        for (i, v) in arr.iter_mut().enumerate() {
            let f = i as f32 / 1023.0;
            *v = java_round_f32(compute_linear_to_srgb(f) * 255.0) as u8;
        }
        arr
    })
}

/// `Math.round(float)` — floor(a + 0.5).
fn java_round_f32(v: f32) -> i32 {
    (v + 0.5).floor() as i32
}

fn compute_srgb_to_linear(value: f32) -> f32 {
    if value >= 0.04045 {
        // Java: (float) Math.pow(...) — pow в double
        (((value + 0.055) / 1.055) as f64).powf(2.4) as f32
    } else {
        value / 12.92
    }
}

fn compute_linear_to_srgb(value: f32) -> f32 {
    if value >= 0.0031308 {
        (1.055 * (value as f64).powf(0.4166666666666667) - 0.055) as f32
    } else {
        12.92 * value
    }
}

pub fn srgb_to_linear_channel(srgb: i32) -> f32 {
    srgb_to_linear()[srgb as usize] as f32 / 1023.0
}

pub fn linear_to_srgb_channel(linear: f32) -> i32 {
    (linear_to_srgb()[mth::floor_f32(linear * 1023.0) as usize]) as i32 & 0xFF
}

pub fn mean_linear(srgb1: i32, srgb2: i32, srgb3: i32, srgb4: i32) -> i32 {
    color4(
        (alpha(srgb1) + alpha(srgb2) + alpha(srgb3) + alpha(srgb4)) / 4,
        linear_channel_mean(red(srgb1), red(srgb2), red(srgb3), red(srgb4)),
        linear_channel_mean(green(srgb1), green(srgb2), green(srgb3), green(srgb4)),
        linear_channel_mean(blue(srgb1), blue(srgb2), blue(srgb3), blue(srgb4)),
    )
}

fn linear_channel_mean(c1: i32, c2: i32, c3: i32, c4: i32) -> i32 {
    let i = (srgb_to_linear()[c1 as usize] as i32
        + srgb_to_linear()[c2 as usize] as i32
        + srgb_to_linear()[c3 as usize] as i32
        + srgb_to_linear()[c4 as usize] as i32)
        / 4;
    linear_to_srgb()[i as usize] as i32 & 0xFF
}

/// Java `color >>> 24` — логический сдвиг (беззнаковый).
pub fn alpha(color: i32) -> i32 {
    ((color as u32) >> 24) as i32
}

pub fn red(color: i32) -> i32 {
    (color >> 16) & 0xFF
}

pub fn green(color: i32) -> i32 {
    (color >> 8) & 0xFF
}

pub fn blue(color: i32) -> i32 {
    color & 0xFF
}

pub fn color4(alpha: i32, red: i32, green: i32, blue: i32) -> i32 {
    ((alpha & 0xFF) << 24) | ((red & 0xFF) << 16) | ((green & 0xFF) << 8) | (blue & 0xFF)
}

pub fn color3(red: i32, green: i32, blue: i32) -> i32 {
    color4(255, red, green, blue)
}

pub fn color_vec3(color: &Vec3) -> i32 {
    color3(
        as8_bit_channel(color.x as f32),
        as8_bit_channel(color.y as f32),
        as8_bit_channel(color.z as f32),
    )
}

pub fn multiply(color1: i32, color2: i32) -> i32 {
    if color1 == -1 {
        color2
    } else if color2 == -1 {
        color1
    } else {
        color4(
            alpha(color1) * alpha(color2) / 255,
            red(color1) * red(color2) / 255,
            green(color1) * green(color2) / 255,
            blue(color1) * blue(color2) / 255,
        )
    }
}

pub fn add_rgb(color1: i32, color2: i32) -> i32 {
    color4(
        alpha(color1),
        (red(color1) + red(color2)).min(255),
        (green(color1) + green(color2)).min(255),
        (blue(color1) + blue(color2)).min(255),
    )
}

pub fn subtract_rgb(color1: i32, color2: i32) -> i32 {
    color4(
        alpha(color1),
        (red(color1) - red(color2)).max(0),
        (green(color1) - green(color2)).max(0),
        (blue(color1) - blue(color2)).max(0),
    )
}

pub fn multiply_alpha(color: i32, alpha_multiplier: f32) -> i32 {
    if color == 0 || alpha_multiplier <= 0.0 {
        0
    } else if alpha_multiplier >= 1.0 {
        color
    } else {
        color_alpha_f(alpha_float(color) * alpha_multiplier, color)
    }
}

pub fn scale_rgb(color: i32, scale: f32) -> i32 {
    scale_rgb_channels(color, scale, scale, scale)
}

pub fn scale_rgb_channels(color: i32, red_scale: f32, green_scale: f32, blue_scale: f32) -> i32 {
    color4(
        alpha(color),
        mth::clamp_i32((red(color) as f32 * red_scale) as i32, 0, 255),
        mth::clamp_i32((green(color) as f32 * green_scale) as i32, 0, 255),
        mth::clamp_i32((blue(color) as f32 * blue_scale) as i32, 0, 255),
    )
}

pub fn scale_rgb_int(color: i32, scale: i32) -> i32 {
    color4(
        alpha(color),
        mth::clamp_i64(red(color) as i64 * scale as i64 / 255, 0, 255) as i32,
        mth::clamp_i64(green(color) as i64 * scale as i64 / 255, 0, 255) as i32,
        mth::clamp_i64(blue(color) as i64 * scale as i64 / 255, 0, 255) as i32,
    )
}

pub fn greyscale(color: i32) -> i32 {
    let i = (red(color) as f32 * 0.3 + green(color) as f32 * 0.59 + blue(color) as f32 * 0.11) as i32;
    color4(alpha(color), i, i, i)
}

pub fn alpha_blend(destination: i32, source: i32) -> i32 {
    let i = alpha(destination);
    let i1 = alpha(source);
    if i1 == 255 {
        source
    } else if i1 == 0 {
        destination
    } else {
        let i2 = i1 + i * (255 - i1) / 255;
        color4(
            i2,
            alpha_blend_channel(i2, i1, red(destination), red(source)),
            alpha_blend_channel(i2, i1, green(destination), green(source)),
            alpha_blend_channel(i2, i1, blue(destination), blue(source)),
        )
    }
}

fn alpha_blend_channel(result_alpha: i32, source_alpha: i32, destination: i32, source: i32) -> i32 {
    (source * source_alpha + destination * (result_alpha - source_alpha)) / result_alpha
}

pub fn srgb_lerp(delta: f32, color1: i32, color2: i32) -> i32 {
    color4(
        mth::lerp_int(delta, alpha(color1), alpha(color2)),
        mth::lerp_int(delta, red(color1), red(color2)),
        mth::lerp_int(delta, green(color1), green(color2)),
        mth::lerp_int(delta, blue(color1), blue(color2)),
    )
}

pub fn linear_lerp(delta: f32, color1: i32, color2: i32) -> i32 {
    let lut = srgb_to_linear();
    let lin = linear_to_srgb();
    color4(
        mth::lerp_int(delta, alpha(color1), alpha(color2)),
        lin[mth::lerp_int(delta, lut[red(color1) as usize] as i32, lut[red(color2) as usize] as i32) as usize]
            as i32
            & 0xFF,
        lin[mth::lerp_int(
            delta,
            lut[green(color1) as usize] as i32,
            lut[green(color2) as usize] as i32,
        ) as usize] as i32
            & 0xFF,
        lin[mth::lerp_int(delta, lut[blue(color1) as usize] as i32, lut[blue(color2) as usize] as i32)
            as usize] as i32
            & 0xFF,
    )
}

pub fn opaque(color: i32) -> i32 {
    color | -16777216 // 0xFF000000
}

pub fn transparent(color: i32) -> i32 {
    color & 16777215
}

/// `ARGB.color(int alpha, int color)`
pub fn color_alpha_int(alpha: i32, color: i32) -> i32 {
    (alpha << 24) | (color & 16777215)
}

/// `ARGB.color(float alpha, int color)`
pub fn color_alpha_f(alpha: f32, color: i32) -> i32 {
    (as8_bit_channel(alpha) << 24) | (color & 16777215)
}

pub fn white_f(alpha: f32) -> i32 {
    (as8_bit_channel(alpha) << 24) | 16777215
}

pub fn white_i(alpha: i32) -> i32 {
    (alpha << 24) | 16777215
}

pub fn black_f(alpha: f32) -> i32 {
    as8_bit_channel(alpha) << 24
}

pub fn black_i(alpha: i32) -> i32 {
    alpha << 24
}

pub fn color_from_float(alpha: f32, red: f32, green: f32, blue: f32) -> i32 {
    color4(
        as8_bit_channel(alpha),
        as8_bit_channel(red),
        as8_bit_channel(green),
        as8_bit_channel(blue),
    )
}

pub fn vector3f_from_rgb24(color: i32) -> Vector3f {
    Vector3f::new(red_float(color), green_float(color), blue_float(color))
}

pub fn vector4f_from_argb32(color: i32) -> Vector4f {
    Vector4f::new(red_float(color), green_float(color), blue_float(color), alpha_float(color))
}

pub fn average(color1: i32, color2: i32) -> i32 {
    color4(
        (alpha(color1) + alpha(color2)) / 2,
        (red(color1) + red(color2)) / 2,
        (green(color1) + green(color2)) / 2,
        (blue(color1) + blue(color2)) / 2,
    )
}

pub fn as8_bit_channel(value: f32) -> i32 {
    mth::floor_f32(value * 255.0)
}

pub fn alpha_float(color: i32) -> f32 {
    from8_bit_channel(alpha(color))
}

pub fn red_float(color: i32) -> f32 {
    from8_bit_channel(red(color))
}

pub fn green_float(color: i32) -> f32 {
    from8_bit_channel(green(color))
}

pub fn blue_float(color: i32) -> f32 {
    from8_bit_channel(blue(color))
}

fn from8_bit_channel(value: i32) -> f32 {
    value as f32 / 255.0
}

pub fn to_abgr(color: i32) -> i32 {
    (color & -16711936) | ((color & 0xFF0000) >> 16) | ((color & 0xFF) << 16)
}

pub fn from_abgr(color: i32) -> i32 {
    to_abgr(color)
}

pub fn set_brightness(color: i32, brightness: f32) -> i32 {
    let mut i = red(color);
    let mut i1 = green(color);
    let mut i2 = blue(color);
    let i3 = alpha(color);
    let max = i.max(i1).max(i2);
    let min = i.min(i1).min(i2);
    let f = (max - min) as f32;
    let f1 = if max != 0 { f / max as f32 } else { 0.0 };
    let mut f2;
    if f1 == 0.0 {
        f2 = 0.0;
    } else {
        let f3 = (max - i) as f32 / f;
        let f4 = (max - i1) as f32 / f;
        let f5 = (max - i2) as f32 / f;
        f2 = if i == max {
            f5 - f4
        } else if i1 == max {
            2.0 + f3 - f5
        } else {
            4.0 + f4 - f3
        };
        f2 /= 6.0;
        if f2 < 0.0 {
            f2 += 1.0;
        }
    }
    if f1 == 0.0 {
        i = java_round_f32(brightness * 255.0);
        i1 = i;
        i2 = i;
        return color4(i3, i, i1, i2);
    }
    let f3x = (f2 - f2.floor()) * 6.0;
    let f4x = f3x - f3x.floor();
    let f5x = brightness * (1.0 - f1);
    let f6 = brightness * (1.0 - f1 * f4x);
    let f7 = brightness * (1.0 - f1 * (1.0 - f4x));
    match f3x as i32 {
        0 => {
            i = java_round_f32(brightness * 255.0);
            i1 = java_round_f32(f7 * 255.0);
            i2 = java_round_f32(f5x * 255.0);
        }
        1 => {
            i = java_round_f32(f6 * 255.0);
            i1 = java_round_f32(brightness * 255.0);
            i2 = java_round_f32(f5x * 255.0);
        }
        2 => {
            i = java_round_f32(f5x * 255.0);
            i1 = java_round_f32(brightness * 255.0);
            i2 = java_round_f32(f7 * 255.0);
        }
        3 => {
            i = java_round_f32(f5x * 255.0);
            i1 = java_round_f32(f6 * 255.0);
            i2 = java_round_f32(brightness * 255.0);
        }
        4 => {
            i = java_round_f32(f7 * 255.0);
            i1 = java_round_f32(f5x * 255.0);
            i2 = java_round_f32(brightness * 255.0);
        }
        5 => {
            i = java_round_f32(brightness * 255.0);
            i1 = java_round_f32(f5x * 255.0);
            i2 = java_round_f32(f6 * 255.0);
        }
        _ => {}
    }
    color4(i3, i, i1, i2)
}
