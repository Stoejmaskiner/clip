use fast_math;
use fast_math::{exp2_raw, log2_raw};
use fastapprox::fast;
use wide::{f32x4, f32x8};

/// Linear interpolation, implementing `lerp()` gives you a lot of other utility
/// functions for free.
pub trait Lerpable: Sized {
    /// Interpolate from `self` to `other` as `t` goes from `0` to `1`.
    ///
    /// Pseudocode:
    /// ```ignore
    /// (1 - t) * self + t * other
    /// ```
    fn lerp(&self, other: Self, t: f32) -> Self;

    /// Given a value `self` in the range `min..max`, return the proportional
    /// position of `self` within the range, as a number between `0` and `1`. If
    /// `self` is close to `min`, the value will be close to `0`, if `self` is
    /// close to `max`, the value will be close to `max`.
    ///
    /// Pseudocode:
    /// ```ignore
    /// (self - min) / (max - self)
    /// ```
    fn inverse_lerp(&self, min: Self, max: Self) -> f32;

    /// Given a value `self` in the range `imin..imax`, return a proportional
    /// value in the range `omin..omax`. If `self` is close to `imin`, the value
    /// will be close to `omin`. If `self` is close to `imax`, the value will be
    /// close to `omax`.
    fn remap(&self, imin: Self, imax: Self, omin: Self, omax: Self) -> Self {
        omin.lerp(omax, self.inverse_lerp(imin, imax))
    }

    /// same as `lerp`, but `t` is clamped to `0..1`
    fn clamped_lerp(&self, other: Self, t: f32) -> Self {
        self.lerp(other, t.clamp(0.0, 1.0))
    }

    /// same as `inverse_lerp`, but output is is clamped to `0..1`
    fn clamped_inverse_lerp(&self, min: Self, max: Self) -> f32 {
        self.inverse_lerp(min, max).clamp(0.0, 1.0)
    }

    /// same as `remap`, but output is clamped to `omin..omax`
    fn clamped_remap(&self, imin: Self, imax: Self, omin: Self, omax: Self) -> Self {
        omin.lerp(omax, self.inverse_lerp(imin, imax).clamp(0.0, 1.0))
    }
}

impl Lerpable for f32 {
    fn lerp(&self, other: f32, t: f32) -> f32 {
        (1.0 - t) * self + t * other
    }

    fn inverse_lerp(&self, min: f32, max: f32) -> f32 {
        (self - min) / (max - min)
    }
}

impl Lerpable for f32x4 {
    fn lerp(&self, other: f32x4, t: f32) -> f32x4 {
        (1.0 - t) * *self + t * other
    }

    fn inverse_lerp(&self, min: f32x4, max: f32x4) -> f32 {
        unimplemented!()
    }
}

impl Lerpable for f32x8 {
    fn lerp(&self, other: f32x8, t: f32) -> f32x8 {
        (1.0 - t) * *self + t * other
    }

    fn inverse_lerp(&self, min: f32x8, max: f32x8) -> f32 {
        unimplemented!()
    }
}

/// perform Catmull-Rom interpolation between two points `x1` and `x2`, with
/// two control points `x0` and `x2` and interpolation parameter `t` in `0..1`
pub fn catmull_rom_interp(x0: f32, x1: f32, x2: f32, x3: f32, t: f32) -> f32 {
    let q0 = (-1.0 * t * t * t) + (2.0 * t * t) + (-1.0 * t);
    let q1 = (3.0 * t * t * t) + (-5.0 * t * t) + 2.0;
    let q2 = (-3.0 * t * t * t) + (4.0 * t * t) + t;
    let q3 = t * t * t - t * t;
    0.5 * (x0 * q0 + x1 * q1 + x2 * q2 + x3 * q3)
}

/// compute fast power assuming x and y are positive or zero
pub fn fast_powf(x: f32, y: f32) -> f32 {
    // a very small number, but larger than machine epsilon, to avoid NaN
    // TODO: removing this SMALL gives ~3 ns performance
    const SMALL: f32 = 1E-30;
    (y * (x + SMALL).log2()).exp2()
}

pub fn fast_powf_simd(x: f32x4, y: f32) -> f32x4 {
    const SMALL: f32 = 1E-30;
    (y * (x + SMALL).ln()).exp()
}

pub fn faster_powf(x: f32, y: f32) -> f32 {
    const SMALL: f32 = 1E-30;
    fast_math::exp2(y * fast_math::log2(x + SMALL))
}

pub fn faster_powf_simd(x: f32x4, y: f32) -> f32x4 {
    const SMALL: f32 = 1E-30;
    let x = x + SMALL;
    let x = y * f32x4::new([
        fast_math::log2(x.as_array_ref()[0]),
        fast_math::log2(x.as_array_ref()[1]),
        fast_math::log2(x.as_array_ref()[2]),
        fast_math::log2(x.as_array_ref()[3]),
    ]);
    f32x4::new([
        fast_math::exp2(x.as_array_ref()[0]),
        fast_math::exp2(x.as_array_ref()[1]),
        fast_math::exp2(x.as_array_ref()[2]),
        fast_math::exp2(x.as_array_ref()[3]),
    ])
}

pub fn fastest_powf(x: f32, y: f32) -> f32 {
    const SMALL: f32 = 1E-30;
    exp2_raw(y * log2_raw(x + SMALL))
}

pub fn fastest_powf_simd(x: f32x4, y: f32) -> f32x4 {
    const SMALL: f32 = 1E-30;
    let x = x + SMALL;
    let x = y * f32x4::new([
        log2_raw(x.as_array_ref()[0]),
        log2_raw(x.as_array_ref()[1]),
        log2_raw(x.as_array_ref()[2]),
        log2_raw(x.as_array_ref()[3]),
    ]);
    f32x4::new([
        exp2_raw(x.as_array_ref()[0]),
        exp2_raw(x.as_array_ref()[1]),
        exp2_raw(x.as_array_ref()[2]),
        exp2_raw(x.as_array_ref()[3]),
    ])
}

/// compute fast power assuming x and y are positive or zero
#[inline(always)]
pub fn inline_fast_powf(x: f32, y: f32) -> f32 {
    // a very small number, but larger than machine epsilon, to avoid NaN
    // TODO: removing this SMALL gives ~3 ns performance
    const SMALL: f32 = 1E-30;
    (y * (x + SMALL).log2()).exp2()
}

#[inline(always)]
pub fn inline_fast_powf_simd(x: f32x4, y: f32) -> f32x4 {
    const SMALL: f32 = 1E-30;
    (y * (x + SMALL).ln()).exp()
}

#[inline(always)]
pub fn inline_faster_powf(x: f32, y: f32) -> f32 {
    const SMALL: f32 = 1E-30;
    fast_math::exp2(y * fast_math::log2(x + SMALL))
}

#[inline(always)]
pub fn inline_faster_powf_simd(x: f32x4, y: f32) -> f32x4 {
    const SMALL: f32 = 1E-30;
    let x = x + SMALL;
    let x = y * f32x4::new([
        fast_math::log2(x.as_array_ref()[0]),
        fast_math::log2(x.as_array_ref()[1]),
        fast_math::log2(x.as_array_ref()[2]),
        fast_math::log2(x.as_array_ref()[3]),
    ]);
    f32x4::new([
        fast_math::exp2(x.as_array_ref()[0]),
        fast_math::exp2(x.as_array_ref()[1]),
        fast_math::exp2(x.as_array_ref()[2]),
        fast_math::exp2(x.as_array_ref()[3]),
    ])
}

#[inline(always)]
pub fn inline_fastest_powf(x: f32, y: f32) -> f32 {
    const SMALL: f32 = 1E-30;
    exp2_raw(y * log2_raw(x + SMALL))
}

#[inline(always)]
pub fn inline_fastest_powf_simd(x: f32x4, y: f32) -> f32x4 {
    const SMALL: f32 = 1E-30;
    let x = x + SMALL;
    let x = y * f32x4::new([
        log2_raw(x.as_array_ref()[0]),
        log2_raw(x.as_array_ref()[1]),
        log2_raw(x.as_array_ref()[2]),
        log2_raw(x.as_array_ref()[3]),
    ]);
    f32x4::new([
        exp2_raw(x.as_array_ref()[0]),
        exp2_raw(x.as_array_ref()[1]),
        exp2_raw(x.as_array_ref()[2]),
        exp2_raw(x.as_array_ref()[3]),
    ])
}
