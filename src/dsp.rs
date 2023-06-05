use num_traits::FromPrimitive;

/// variable hardness clipping. For hardness `h`, the range `[0, 0.935]` is normal.
///
/// Due to issues with stability when `h` approaches 1, crossfades internally to a
/// digital hard clip after 0.935.
pub fn var_hard_clip(x: f32, hardness: f32) -> f32 {
    let clamped_hardness = hardness.min(0.935);
    let fade = (hardness - clamped_hardness) / (1.0 - 0.935);
    let softness = 1.0 - clamped_hardness * 0.5 - 0.5;
    let analog = x / (1.0 + x.abs().powf(softness.recip())).powf(softness);
    let digital = x.clamp(-1.0, 1.0);
    analog * (1.0 - fade) + digital * fade
}

pub trait Processor {
    fn step(&mut self, x: f32) -> f32;

    /// implement to provide a vectorized version, otherwise it defaults to
    /// calling step repeatedly
    fn process(&mut self, buffer: &mut [f32]) {
        for x in buffer.iter_mut() {
            *x = self.step(*x);
        }
    }

    /// latency in fractions of samples. If you implement this, then `latency`
    /// is defined by default in terms of this.
    ///
    /// This latency can usually be calculated in terms of the exact latency of inner
    /// processors.
    fn exact_latency(&self) -> f32 {
        0.0
    }

    fn rounded_latency(&self) -> usize {
        usize::from_f32(self.exact_latency().max(0.0).ceil()).unwrap()
    }

    // TODO:
    // fn process_simd(&mut self, xs: <SIMDF32>) -> <SIMDF32>;
}

/// DC blocking filter. Very cheap, but not completely SR independent, oh well.
/// It is very transparent, so SR differences *should* be negligible, unless
/// you use absurd sampling rates, which you shouldn't btw.
#[derive(Default, Clone)]
pub struct DCBlock {
    x_z1: f32,
    y_z1: f32,
}

impl Processor for DCBlock {
    fn step(&mut self, x: f32) -> f32 {
        let y = x - self.x_z1 + 0.9975 * self.y_z1;
        self.x_z1 = x;
        self.y_z1 = y;
        y
    }
}

/// a simple processor that allows wrapping a function into a processor, for
/// use in processor chains and containers
pub struct ClosureProcessor<F>
where
    F: Fn(f32) -> f32,
{
    closure: F,
}

impl<F> Processor for ClosureProcessor<F>
where
    F: Fn(f32) -> f32,
{
    fn step(&mut self, x: f32) -> f32 {
        (self.closure)(x)
    }
}

/// fast X4 oversampling wrapper
pub struct OversampleX4<P: Processor> {
    inner_processor: P,
}

impl<P: Processor> Processor for OversampleX4<P> {
    fn step(&mut self, x: f32) -> f32 {
        x
    }
}
