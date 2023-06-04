//! A super simple peak meter widget.

use crate::math_utils::Lerpable;
use nih_plug::prelude::util;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use static_assertions::const_assert;
use std::cell::Cell;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

/// Plot a 1D function or signal to the gui
pub struct Plot1DData<const N: usize> {
    pub ys: [f32; N],
    pub ylim: (f32, f32),
    pub xlim: (f32, f32),
    pub draw_ticks: bool,
    pub in_amp: f32,
    pub out_amp: f32,
}

impl<const N: usize> Plot1DData<N> {
    const _GUARD: () = assert!(N >= 2);
    const TICKS: [f32; 6] = [0.125893, 0.177828, 0.251189, 0.354813, 0.501187, 0.707946];
    const TICK_LABELS: [&str; 6] = ["-18", "", "-12", "-9", "-6", "-3"];
    pub fn new() -> Self {
        #[allow(clippy::let_unit_value)]
        let _ = Self::_GUARD; // HACK: if omitted, rustc will just remove the assertion
        Self {
            ys: [0.0; N],
            ylim: (-1.0, 1.0),
            xlim: (-1.0, 1.0),
            // TODO: remove
            draw_ticks: true,
            in_amp: 0.0,
            out_amp: 0.0,
        }
    }

    pub fn clear(&mut self) {
        for elem in self.ys.iter_mut() {
            *elem = 0.0;
        }
    }

    pub fn plot_function(&mut self, f: impl Fn(f32) -> f32) {
        for i in 0..N {
            let x = self.xlim.0.lerp(self.xlim.1, i as f32 / N as f32);
            self.ys[i] = f(x)
        }
    }
}

// TODO: this should be removed once custom CSS properties are supported by Vizia
struct Plot1DAdditionalStyles {
    fill_color: Color,
}

pub struct Plot1D<const N: usize> {
    data: Arc<Mutex<Plot1DData<N>>>,
    additional_styles: Plot1DAdditionalStyles,
}

impl<const N: usize> Plot1D<N> {
    pub fn new(cx: &mut Context, data: Arc<Mutex<Plot1DData<N>>>) -> Handle<Self> {
        Self {
            data,
            additional_styles: Plot1DAdditionalStyles {
                // TODO: this is hardcoded to be the same fill color as the param bars...
                //       not good!
                fill_color: Color::rgba(0xC4, 0xC4, 0xC4, 0x80),
            },
        }
        .build(cx, |_| {})
    }
}

impl<const N: usize> View for Plot1D<N> {
    fn element(&self) -> Option<&'static str> {
        Some("plot1d")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        // These basics are taken directly from the default implementation of this function
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }
        let border_width = match cx.border_width().unwrap_or_default() {
            Units::Pixels(val) => val,
            Units::Percentage(val) => bounds.w.min(bounds.h) * (val / 100.0),
            _ => 0.0,
        };
        let bx = bounds.x + border_width / 2.0;
        let by = bounds.y + border_width / 2.0;
        let bw = bounds.w - border_width;
        let bh = bounds.h - border_width;

        let outline_width = match cx.outline_width().unwrap_or_default() {
            Units::Pixels(val) => val,
            Units::Percentage(val) => bounds.w.min(bounds.h) * (val / 100.0),
            _ => 0.0,
        };

        let opacity = cx.opacity();
        let outline_color = {
            let mut outline_color: vg::Color =
                cx.outline_color().cloned().unwrap_or_default().into();
            outline_color.set_alphaf(outline_color.a * opacity);
            outline_color
        };
        let border_color = {
            let mut border_color: vg::Color = cx.border_color().cloned().unwrap_or_default().into();
            border_color.set_alphaf(outline_color.a * opacity);
            border_color
        };
        let fill_color = {
            let mut fill_color: vg::Color = self.additional_styles.fill_color.into();
            fill_color.set_alphaf(fill_color.a * opacity);
            fill_color
        };

        // draw plot
        let mut path = vg::Path::new();
        {
            // NOTE: lock acquired here
            let data = self.data.lock().unwrap();
            let mut points = data.ys.iter().enumerate().map(|(x, y)| {
                // scale x and y from xlim, ylim space to screen space
                let x = bx + (x as f32) / ((N - 1) as f32) * bw;
                let y = by + y.inverse_lerp(data.ylim.1, data.ylim.0) * bh;
                (x, y)
            });
            // this can't panic because we asserted that N >= 2 at compile time
            let (x, y) = points.next().unwrap();
            path.move_to(x, y);
            for (x, y) in points {
                path.line_to(x, y);
            }
            // end NOTE: lock released here
        };
        let paint = {
            let mut paint = vg::Paint::color(outline_color);
            paint.set_line_width(outline_width);
            paint
        };
        canvas.stroke_path(&mut path, &paint);

        // draw peak meters
        let mut path = vg::Path::new();
        {
            let data = self.data.lock().unwrap();
            path.rect(
                bx,
                by + bh - data.out_amp * bh,
                data.in_amp * bw,
                data.out_amp * bh,
            );
            //path.rect()
        }
        let paint = vg::Paint::color(fill_color);
        canvas.fill_path(&mut path, &paint);

        // draw ticks
        let mut path = vg::Path::new();
        {
            let y_base = by + bh;
            let y_top = y_base - 8.0;
            let x_base = bx;
            let x_top = x_base + 8.0;
            let data = self.data.lock().unwrap();
            if data.draw_ticks {
                // TODO: print labels
                for (t, _label) in Plot1DData::<0>::TICKS
                    .iter()
                    .zip(Plot1DData::<0>::TICK_LABELS.iter())
                {
                    let x = bx.lerp(bx + bw, *t);
                    let y = (by + bh).lerp(by, *t);
                    path.move_to(x, y_base);
                    path.line_to(x, y_top);
                    path.move_to(x_base, y);
                    path.line_to(x_top, y);
                }
            }
        }
        let mut paint = vg::Paint::color(border_color);
        paint.set_line_width(border_width);
        canvas.stroke_path(&mut path, &paint);

        // draw border
        let mut path = vg::Path::new();
        {
            path.move_to(bx, by);
            path.line_to(bx, by + bh);
            path.line_to(bx + bw, by + bh);
            path.line_to(bx + bw, by);
            path.line_to(bx, by);
            path.close();
        }
        let mut paint = vg::Paint::color(border_color);
        paint.set_line_width(border_width);
        canvas.stroke_path(&mut path, &paint);
    }
}
