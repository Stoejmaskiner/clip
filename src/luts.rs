//! TODO: this does not work as of now!

use crate::math_utils::Lerpable;
use num_traits::AsPrimitive;
use std::fmt::Debug;

trait BasicValue: Sized + Debug + Clone + std::ops::Sub {}

use nih_plug::wrapper::clap::lazy_static;

#[derive(Debug, Clone)]
pub enum Interpolation {
    Truncate,
    Nearest,
    Linear,
    Quadratic,
    Cubic,
}

#[derive(Debug, Clone)]
pub struct LinSpace(f32, f32);

#[derive(Debug, Clone)]
pub struct LUT1D<const N: usize>([f32; N], LinSpace);

impl<const N: usize> LUT1D<N> {
    pub fn from_function(f: impl Fn(f32) -> f32, range: LinSpace) -> Self {
        let vals: [f32; N] = (0..N)
            .map(|i| (i as f32).remap(0.0, N as f32, range.0, range.1))
            //.map(|i| (i as f32 / N as f32) * (max - min) + min)
            .map(|x| f(x))
            .collect::<Vec<f32>>()
            .try_into()
            .unwrap();

        Self(vals, range)
    }

    pub fn eval(&self, x: f32, interp: Interpolation) -> f32 {
        match interp {
            Interpolation::Truncate => todo!(),
            Interpolation::Nearest => todo!(),
            Interpolation::Linear => todo!(),
            Interpolation::Quadratic => todo!(),
            Interpolation::Cubic => self.eval_cubic(x),
        }
    }

    pub fn eval_cubic(x: f32) -> f32 {}
}

#[derive(Debug, Clone)]
pub struct LUT2D<const N: usize, const M: usize>([[f32; M]; N], f32, f32, f32, f32);

impl<const N: usize, const M: usize> LUT2D<N, M> {
    pub fn from_function(
        f: impl Fn(f32, f32) -> f32,
        xmin: f32,
        ymin: f32,
        xmax: f32,
        ymax: f32,
    ) -> Self {
        let idx_xmax = N as f32;
        let idx_ymax = M as f32;
        Self(
            (0..N)
                .map(|xi| {
                    let x = (xi as f32 / idx_xmax) * (xmax - xmin) + xmin;
                    (0..M)
                        .map(|yi| (yi as f32 / idx_ymax) * (ymax - ymin) + ymin)
                        .map(|y| f(x, y))
                        .collect::<Vec<f32>>()
                        .try_into()
                        .unwrap()
                })
                .collect::<Vec<[f32; M]>>()
                .try_into()
                .unwrap(),
            xmin,
            ymin,
            xmax,
            ymax,
        )
    }
}
