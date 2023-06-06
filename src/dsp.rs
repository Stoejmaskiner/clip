use num_traits::FromPrimitive;

// generated using scripts/filters.py
const MIN_PHASE_FIR_HALFBAND: usize = 30;
const MIN_PHASE_FIR_HALFBAND_FRAC_2: usize = MIN_PHASE_FIR_HALFBAND / 2;
const MIN_PHASE_HALFBAND: [f32; MIN_PHASE_FIR_HALFBAND] = [
    0.05349789653525128,
    0.20166530907017383,
    0.3665564610202743,
    0.38276975370675703,
    0.18390590463720796,
    -0.0751514681309898,
    -0.16576188035056055,
    -0.049620308414873714,
    0.08674984956321416,
    0.0788759087731493,
    -0.024001015790926138,
    -0.06699486240789757,
    -0.01164952637895525,
    0.042559957248806284,
    0.02487196863463194,
    -0.01999681539939316,
    -0.024144101815857494,
    0.0049342447197044375,
    0.01739727476557889,
    0.0023797023779942614,
    -0.010077325489379755,
    -0.004276124695963664,
    0.004771534702310419,
    0.0035256758204267155,
    -0.0018835102233401696,
    -0.002146347959864804,
    0.0007281761768500793,
    0.0010984367009729214,
    -0.00043280417766092364,
    0.0,
];

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

pub trait MonoProcessor {
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

impl MonoProcessor for DCBlock {
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

impl<F> MonoProcessor for ClosureProcessor<F>
where
    F: Fn(f32) -> f32,
{
    fn step(&mut self, x: f32) -> f32 {
        (self.closure)(x)
    }
}

struct RingBuffer<const N: usize> {
    buffer: [f32; N],
    write_head: usize,
}

impl<const N: usize> RingBuffer<N> {
    fn new() -> Self {
        Self {
            buffer: [0.0; N + 1],
            write_head: 0,
        }
    }

    fn push(&mut self, x: f32) -> &mut Self {
        self.write_head += 1;
        self.write_head %= N;
        self.buffer[self.write_head] = x;
        self
    }

    fn tap(&self, delay: usize) -> f32 {
        assert!(delay < N);
        let idx = (self.write_head - delay) % N;
        self.buffer[idx]
    }
}

struct FixedDelay<const N: usize> {
    buffer: RingBuffer<N>,
}

impl<const N: usize> FixedDelay<N> {
    fn new() -> Self {
        Self {
            buffer: RingBuffer<N>::new()
        }
    }
}

impl<const N: usize> MonoProcessor for FixedDelay<N> {
    fn step(&mut self, x: f32) -> f32 {
        self.buffer.push(x).tap(N)
    }

    fn exact_latency(&self) -> f32 {
        f32::from(N)
    }

    fn rounded_latency(&self) -> usize {
        N
    }
}

/// fast X2 oversampling wrapper
pub struct Oversample<P: MonoProcessor> {
    inner_processor: P,
    even_delay_line: FixedDelay,
    odd_delay_line: FixedDelay,
}

impl<P: MonoProcessor> Oversample<P> {
    fn new(inner_processor: P) -> Self {
        Self {
            inner_processor: inner_processor,
            even_delay_line: FixedDelay<MIN_PHASE_FIR_HALFBAND_FRAC_2>::new(),
            odd_delay_line: FixedDelay<MIN_PHASE_FIR_HALFBAND_FRAC_2>::new()
        }
    }

    fn step_even(&mut self) -> f32 {
        todo!();
    }

    fn step_odd(&mut self) -> f32 {
        todo!();
    }
}

impl<P: MonoProcessor> MonoProcessor for Oversample<P> {
    fn step(&mut self, x: f32) -> f32 {
        self.inner_processor.step(x)
    }
}
