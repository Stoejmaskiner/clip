use num_traits::FromPrimitive;

// mod filter_coefficients;
mod ring_buffer;
use crate::filter_coefficients::LP_FIR_2X_TO_1X_MINIMUM;
use crate::filter_coefficients::LP_FIR_2X_TO_1X_MINIMUM_LEN;
use crate::filter_coefficients::LP_FIR_4X_TO_2X_MINIMUM;
use crate::filter_coefficients::LP_FIR_4X_TO_2X_MINIMUM_LEN;

use self::ring_buffer::RingBuffer;

// TODO: this is necessary because rust does not have const generic expressions
const LP_FIR_2X_TO_1X_MINIMUM_LEN_FRAC_2: usize = LP_FIR_2X_TO_1X_MINIMUM_LEN / 2;
const LP_FIR_4X_TO_2X_MINIMUM_LEN_FRAC_2: usize = LP_FIR_4X_TO_2X_MINIMUM_LEN / 2;
const LP_FIR_2X_TO_1X_MINIMUM_LEN_PLUS_1: usize = LP_FIR_2X_TO_1X_MINIMUM_LEN + 1;
const LP_FIR_4X_TO_2X_MINIMUM_LEN_PLUS_1: usize = LP_FIR_4X_TO_2X_MINIMUM_LEN + 1;
const LP_FIR_2X_TO_1X_MINIMUM_LEN_FRAC_2_PLUS_1: usize = LP_FIR_2X_TO_1X_MINIMUM_LEN_FRAC_2 + 1;
const LP_FIR_4X_TO_2X_MINIMUM_LEN_FRAC_2_PLUS_1: usize = LP_FIR_4X_TO_2X_MINIMUM_LEN_FRAC_2 + 1;
const EFFECTIVE_TAIL: usize = 128;

/// variable hardness clipping. For hardness `h`, the range `[0, 0.935]` is normal.
///
/// Due to issues with stability when `h` approaches 1, crossfades internally to a
/// digital hard clip after 0.935.
// TODO: very hot function, optimize!
pub(crate) fn var_hard_clip(x: f32, hardness: f32) -> f32 {
    let clamped_hardness = hardness.min(0.935);
    let fade = (hardness - clamped_hardness) / (1.0 - 0.935);
    let softness = 1.0 - clamped_hardness * 0.5 - 0.5;
    let analog = x / (1.0 + x.abs().powf(softness.recip())).powf(softness);
    let digital = x.clamp(-1.0, 1.0);
    analog * (1.0 - fade) + digital * fade
}

/// processor version of `var_hard_clip`
#[derive(Default, Clone, Debug)]
pub struct VarHardClip {
    pub hardness: f32,
}

impl MonoProcessor for VarHardClip {
    fn step(&mut self, x: f32) -> f32 {
        var_hard_clip(x, self.hardness)
    }

    fn reset(&mut self) {}

    fn initialize(&mut self) {}
}

pub trait MonoProcessor {
    /// process a single sample of audio
    fn step(&mut self, x: f32) -> f32;

    /// reset any buffers or envelopes
    fn reset(&mut self);

    /// initialize expensive calculations that are only run on program changes
    fn initialize(&mut self);

    /// implement to provide a vectorized version, otherwise it defaults to
    /// calling step repeatedly
    fn process(&mut self, buffer: &mut [f32]) {
        for x in buffer.iter_mut() {
            *x = self.step(*x);
        }
    }

    /// latency in fractions of samples. If you implement this, then `rounded_latency`
    /// is defined by default in terms of this.
    ///
    /// This latency can usually be calculated in terms of the exact latency of inner
    /// processors.
    fn exact_latency(&self) -> f32 {
        0.0
    }

    fn rounded_latency(&self) -> usize {
        usize::from_f32(self.exact_latency().max(0.0).round()).unwrap()
    }

    /// tail length in fractions of samples. If you implement this, then `rounded_tail`
    /// is defined by default in terms of this.
    ///
    /// This tail length can usually be calculated in terms of the exact latency of
    /// inner processors.
    fn exact_tail(&self) -> f32 {
        0.0
    }

    fn rounded_tail(&self) -> usize {
        usize::from_f32(self.exact_tail().max(0.0).ceil()).unwrap()
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

    fn reset(&mut self) {
        self.y_z1 = 0.0;
        self.x_z1 = 0.0;
    }

    fn initialize(&mut self) {}
}

// /// a simple processor that allows wrapping a function into a processor, for
// /// use in processor chains and containers
// pub struct ClosureProcessor<F>
// where
//     F: Fn(f32) -> f32,
// {
//     closure: F,
// }

// impl<F> MonoProcessor for ClosureProcessor<F>
// where
//     F: Fn(f32) -> f32,
// {
//     fn step(&mut self, x: f32) -> f32 {
//         (self.closure)(x)
//     }
// }

/// fast X4 oversampling wrapper
#[derive(Debug, Clone)]
pub struct OversampleX4<P: MonoProcessor> {
    pub inner_processor: P,
    up_delay_line_x2: RingBuffer<LP_FIR_2X_TO_1X_MINIMUM_LEN_FRAC_2_PLUS_1>,
    up_delay_line_x4: RingBuffer<LP_FIR_4X_TO_2X_MINIMUM_LEN_FRAC_2_PLUS_1>,
    down_delay_line_x2: RingBuffer<LP_FIR_2X_TO_1X_MINIMUM_LEN_PLUS_1>,
    down_delay_line_x4: RingBuffer<LP_FIR_4X_TO_2X_MINIMUM_LEN_PLUS_1>,
}

impl<P: MonoProcessor> OversampleX4<P> {
    pub fn new(inner_processor: P) -> Self {
        Self {
            inner_processor: inner_processor,
            up_delay_line_x2: RingBuffer::new(),
            up_delay_line_x4: RingBuffer::new(),
            down_delay_line_x2: RingBuffer::new(),
            down_delay_line_x4: RingBuffer::new(),
        }
    }

    fn step_up_2x(&mut self, x: f32) -> (f32, f32) {
        self.up_delay_line_x2.push(x);

        // 2x even step
        let even = {
            let mut a = 0.0f32;
            for i in 0..(self.up_delay_line_x2.len() - 1) {
                let coeff = LP_FIR_2X_TO_1X_MINIMUM[i * 2];
                a += self.up_delay_line_x2.tap(i) * coeff;
            }
            a * 2.0
        };

        // 2x odd step
        let odd = {
            let mut a = 0.0f32;
            for i in 0..(self.up_delay_line_x2.len() - 1) {
                let coeff = LP_FIR_2X_TO_1X_MINIMUM[1 + i * 2];
                a += self.up_delay_line_x2.tap(i) * coeff;
            }
            a * 2.0
        };

        (even, odd)
    }

    fn step_up_4x(&mut self, x: f32) -> (f32, f32) {
        self.up_delay_line_x4.push(x);

        // 4x even step
        let even = {
            let mut a = 0.0f32;
            for i in 0..(self.up_delay_line_x4.len() - 1) {
                let coeff = LP_FIR_4X_TO_2X_MINIMUM[i * 2];
                a += self.up_delay_line_x4.tap(i) * coeff;
            }
            a * 2.0
        };

        // 4x odd step
        let odd = {
            let mut a = 0.0f32;
            for i in 0..(self.up_delay_line_x4.len() - 1) {
                let coeff = LP_FIR_4X_TO_2X_MINIMUM[1 + i * 2];
                a += self.up_delay_line_x4.tap(i) * coeff;
            }
            a * 2.0
        };

        (even, odd)
    }

    fn step_down_4x(&mut self, x0: f32, x1: f32) -> f32 {
        self.down_delay_line_x4.push(x0);
        self.down_delay_line_x4.push(x1);

        let mut a = 0.0f32;
        for i in 0..(self.down_delay_line_x4.len() - 1) {
            let coeff = LP_FIR_4X_TO_2X_MINIMUM[i];
            a += self.down_delay_line_x4.tap(i) * coeff;
        }
        a
    }

    fn step_down_2x(&mut self, x0: f32, x1: f32) -> f32 {
        self.down_delay_line_x2.push(x0);
        self.down_delay_line_x2.push(x1);

        let mut a = 0.0f32;
        for i in 0..(self.down_delay_line_x2.len() - 1) {
            let coeff = LP_FIR_2X_TO_1X_MINIMUM[i];
            a += self.down_delay_line_x2.tap(i) * coeff;
        }
        a
    }
}

impl<P: MonoProcessor> MonoProcessor for OversampleX4<P> {
    fn step(&mut self, x: f32) -> f32 {
        // get 4 consecutive upsampled samples from 1 input sample
        // let (x0, x1) = self.step_up_2x(x);
        // let (x00, x01) = self.step_up_4x(x0);
        // let (x10, x11) = self.step_up_4x(x1);

        // let y00 = self.inner_processor.step(x00);
        // let y01 = self.inner_processor.step(x01);
        // let y10 = self.inner_processor.step(x10);
        // let y11 = self.inner_processor.step(x11);

        // let y0 = self.step_down_4x(y00, y01);
        // let y1 = self.step_down_4x(y10, y11);
        // self.step_down_2x(y0, y1)
        self.inner_processor.step(x)
    }

    fn reset(&mut self) {
        self.up_delay_line_x2.reset();
        self.up_delay_line_x4.reset();
        self.down_delay_line_x2.reset();
        self.down_delay_line_x4.reset();
    }

    fn initialize(&mut self) {}
}
