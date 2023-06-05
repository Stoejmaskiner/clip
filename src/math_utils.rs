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
