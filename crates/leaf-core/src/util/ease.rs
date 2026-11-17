//! Перенос `net.minecraft.util.Ease` — easing-функции для анимаций.

use crate::util::mth;

pub fn in_back(progress: f32) -> f32 {
    let f1 = 2.70158f32;
    mth::square_f32(progress) * (f1 * progress - 1.70158)
}

pub fn in_bounce(progress: f32) -> f32 {
    1.0 - out_bounce(1.0 - progress)
}

pub fn in_cubic(progress: f32) -> f32 {
    mth::cube_f32(progress)
}

pub fn in_elastic(progress: f32) -> f32 {
    if progress == 0.0 {
        0.0
    } else if progress == 1.0 {
        1.0
    } else {
        let f = (std::f64::consts::PI * 2.0 / 3.0) as f32;
        (-2f32.powf(10.0 * progress - 10.0)
            * ((progress * 10.0 - 10.75) * (std::f64::consts::PI * 2.0 / 3.0) as f32).sin())
    }
}

pub fn in_expo(progress: f32) -> f32 {
    if progress == 0.0 { 0.0 } else { 2f32.powf(10.0 * progress - 10.0) }
}

pub fn in_quart(progress: f32) -> f32 {
    mth::square_f32(mth::square_f32(progress))
}

pub fn in_quint(progress: f32) -> f32 {
    mth::square_f32(mth::square_f32(progress)) * progress
}

pub fn in_sine(progress: f32) -> f32 {
    1.0 - mth::cos((progress * (std::f64::consts::PI as f32 / 2.0)) as f64)
}

pub fn in_out_bounce(progress: f32) -> f32 {
    if progress < 0.5 {
        (1.0 - out_bounce(1.0 - 2.0 * progress)) / 2.0
    } else {
        (1.0 + out_bounce(2.0 * progress - 1.0)) / 2.0
    }
}

pub fn in_out_circ(progress: f32) -> f32 {
    if progress < 0.5 {
        ((1.0 - (2.0 * progress as f64).powi(2)).sqrt()) as f32 / 2.0
    } else {
        (((1.0 - (-2.0 * progress as f64 + 2.0).powi(2)).sqrt()) as f32 + 1.0) / 2.0
    }
}

pub fn in_out_cubic(progress: f32) -> f32 {
    if progress < 0.5 {
        4.0 * mth::cube_f32(progress)
    } else {
        (1.0 - (-2.0 * progress as f64 + 2.0).powi(3) as f32 / 2.0)
    }
}

pub fn in_out_quad(progress: f32) -> f32 {
    if progress < 0.5 {
        2.0 * mth::square_f32(progress)
    } else {
        1.0 - (-2.0 * progress as f64 + 2.0).powi(2) as f32 / 2.0
    }
}

pub fn in_out_quart(progress: f32) -> f32 {
    if progress < 0.5 {
        8.0 * mth::square_f32(mth::square_f32(progress))
    } else {
        1.0 - (-2.0 * progress as f64 + 2.0).powi(4) as f32 / 2.0
    }
}

pub fn in_out_quint(progress: f32) -> f32 {
    if progress < 0.5 {
        16.0 * progress * progress * progress * progress * progress
    } else {
        (1.0 - (-2.0 * progress as f64 + 2.0).powi(5) as f32 / 2.0)
    }
}

pub fn out_bounce(progress: f32) -> f32 {
    if progress < 0.36363637 {
        7.5625 * mth::square_f32(progress)
    } else if progress < 0.72727275 {
        7.5625 * mth::square_f32(progress - 0.54545456) + 0.75
    } else if progress < 0.9090909090909091 {
        7.5625 * mth::square_f32(progress - 0.8181818) + 0.9375
    } else {
        7.5625 * mth::square_f32(progress - 0.95454544) + 0.984375
    }
}

pub fn out_elastic(progress: f32) -> f32 {
    if progress == 0.0 {
        0.0
    } else if progress == 1.0 {
        1.0
    } else {
        ((-10.0 * progress).exp2() * ((progress * 10.0 - 0.75) * (std::f64::consts::PI as f32 * 4.0 / 3.0)).sin()) as f32 + 1.0
    }
}

pub fn out_expo(progress: f32) -> f32 {
    if progress == 1.0 { 1.0 } else { 1.0 - (-10.0 * progress).exp2() as f32 }
}

pub fn out_quad(progress: f32) -> f32 {
    1.0 - mth::square_f32(1.0 - progress)
}

pub fn out_quint(progress: f32) -> f32 {
    1.0 - (1.0 - progress as f64).powi(5) as f32
}

pub fn out_sine(progress: f32) -> f32 {
    mth::sin((progress * (std::f64::consts::PI as f32 / 2.0)) as f64)
}

pub fn in_out_sine(progress: f32) -> f32 {
    -(mth::cos(std::f64::consts::PI * progress as f64) - 1.0) / 2.0
}

pub fn out_back(progress: f32) -> f32 {
    1.0 + 2.70158 * mth::cube_f32(progress - 1.0) + 1.70158 * mth::square_f32(progress - 1.0)
}

pub fn out_quart(progress: f32) -> f32 {
    1.0 - mth::square_f32(mth::square_f32(1.0 - progress))
}

pub fn out_cubic(progress: f32) -> f32 {
    1.0 - mth::cube_f32(1.0 - progress)
}

pub fn in_out_expo(progress: f32) -> f32 {
    if progress < 0.5 {
        if progress == 0.0 { 0.0 } else { (2f64.powf(20.0 * progress as f64 - 10.0) / 2.0) as f32 }
    } else if progress == 1.0 {
        1.0
    } else {
        ((2.0 - 2f64.powf(-20.0 * progress as f64 + 10.0)) / 2.0) as f32
    }
}

pub fn in_quad(progress: f32) -> f32 {
    progress * progress
}

pub fn out_circ(progress: f32) -> f32 {
    (1.0 - mth::square_f32(progress - 1.0)).sqrt()
}

pub fn in_out_elastic(progress: f32) -> f32 {
    let f = std::f32::consts::PI * 4.0 / 9.0;
    if progress == 0.0 {
        0.0
    } else if progress == 1.0 {
        1.0
    } else {
        let sin = ((20.0 * progress - 11.125) * f).sin();
        if progress < 0.5 {
            -(2f64.powf(20.0 * progress as f64 - 10.0) as f32 * sin) / 2.0
        } else {
            (2f64.powf(-20.0 * progress as f64 + 10.0) as f32 * sin / 2.0) + 1.0
        }
    }
}

pub fn in_circ(progress: f32) -> f32 {
    (-(1.0 - progress * progress).sqrt()) + 1.0
}

pub fn in_out_back(progress: f32) -> f32 {
    if progress < 0.5 {
        4.0 * progress * progress * (7.189819 * progress - 2.5949094) / 2.0
    } else {
        let f2 = 2.0 * progress - 2.0;
        (f2 * f2 * (3.5949094 * f2 + 2.5949094) + 2.0) / 2.0
    }
}
