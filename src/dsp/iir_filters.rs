use super::MonoProcessor;

/// biquad filter structure with constant coefficients. Uses the transposed
/// direct form 2
pub struct FixedBiquad {
    // numerator coefficients
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,

    // denominator coefficients
    pub a1: f32,
    pub a2: f32,

    // states
    s1: f32,
    s2: f32,
}

impl FixedBiquad {
    pub fn new(b0: f32, b1: f32, b2: f32, a1: f32, a2: f32) -> Self {
        Self {
            b0: b0,
            b1: b1,
            b2: b2,
            a1: a1,
            a2: a2,
            s1: 0.0,
            s2: 0.0,
        }
    }
}

impl MonoProcessor for FixedBiquad {
    /// transposed direct form 2
    fn step(&mut self, x: f32) -> f32 {
        let y = x * self.b0 + self.s1;
        self.s1 = x * self.b1 - self.a1 * y + self.s2;
        self.s2 = x * self.b2 - self.a2 * y;
        y
    }

    fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }

    fn initialize(&mut self) {}
}

pub struct FixedChebyshev2<const N: usize> {
    biquads: [FixedBiquad; N],
}

impl<const N: usize> FixedChebyshev2<N> {
    const ORDER: usize = N * 2;

    /// given normalized cutoff frequency in `[0, 1]` and passband attenuation
    /// in positive dB, generates a cascaded biquad implementation of the
    /// chebyshev2 filter
    pub fn new(cutoff: f32, passband_atten: f32) -> Self {}
}
