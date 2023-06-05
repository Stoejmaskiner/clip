use crate::widgets::{Plot1D, Plot1DData};
use crate::ClipParams;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::{ParamButton, ParamSlider, ResizeHandle};
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::{Arc, Mutex};

#[derive(Lens)]
struct Data {
    params: Arc<ClipParams>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 400))
}

macro_rules! vspace {
    ($cx: expr, $height: expr) => {{
        let handle = Element::new($cx).height(Pixels($height));

        #[cfg(feature = "draw_gizmos")]
        {
            let handle = handle
                .border_width(Pixels(1.0))
                .border_color(Color::rgb(255, 0, 0));
            handle
        }
        #[cfg(not(feature = "draw_gizmos"))]
        {
            handle
        }
    }};
}

macro_rules! with_gizmos {
    {$block: expr} => {
        #[cfg(feature = "draw_gizmos")]
        {
            $block
                .border_width(Pixels(1.0))
                .border_color(Color::rgb(0xff, 0, 0));
        }
        #[cfg(not(feature = "draw_gizmos"))]
        {
            $block
        }
    };
}

pub(crate) fn create<const N: usize>(
    params: Arc<ClipParams>,
    plot: Arc<Plot1DData<N>>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
        }
        .build(cx);

        ResizeHandle::new(cx);

        HStack::new(cx, |cx| {
            // left column
            with_gizmos! {
                VStack::new(cx, |cx| {
                    Label::new(cx, "Bypass");
                    vspace!(cx, 3.0);
                    ParamButton::new(cx, Data::params, |params| &params.bypass);
                    vspace!(cx, 6.0);
                    Label::new(cx, "Pre Gain");
                    vspace!(cx, 3.0);
                    ParamSlider::new(cx, Data::params, |params| &params.pre_gain);
                    vspace!(cx, 6.0);
                    Label::new(cx, "Post Gain");
                    vspace!(cx, 3.0);
                    ParamSlider::new(cx, Data::params, |params| &params.post_gain);
                    vspace!(cx, 6.0);
                    Label::new(cx, "Hardness");
                    vspace!(cx, 3.0);
                    ParamSlider::new(cx, Data::params, |params| &params.hardness);
                })
            };

            // center column
            with_gizmos! {
                VStack::new(cx, |cx| {
                    Label::new(cx, "Drive");
                    vspace!(cx, 3.0);
                    ParamSlider::new(cx, Data::params, |params| &params.drive);
                    vspace!(cx, 6.0);
                    Label::new(cx, "Threshold");
                    vspace!(cx, 3.0);
                    ParamSlider::new(cx, Data::params, |params| &params.threshold);
                    vspace!(cx, 6.0);
                    Label::new(cx, "Mix");
                    vspace!(cx, 3.0);
                    ParamSlider::new(cx, Data::params, |params| &params.mix);
                    vspace!(cx, 6.0);
                    Label::new(cx, "DC Block");
                    vspace!(cx, 3.0);
                    ParamButton::new(cx, Data::params, |params| &params.dc_block);
                })
            };
            // .border_width(Pixels(1.0))
            // .border_color(Color::rgb(255, 0, 0));

            Plot1D::new(cx, plot.clone())
                .outline_width(Pixels(2.0))
                .width(Pixels(222.0))
                .height(Stretch(1.0))
                .outline_color(Color::black())
                .border_width(Pixels(1.0))
                .border_color(Color::black());
            // Element::new(cx)
            //     .width(Pixels(100.0))
            //     .height(Stretch(1.0))
            //     .border_color(Color::rgb(255, 0, 0))
            //     .border_width(Pixels(1.0));
        })
        .width(Auto)
        .height(Auto)
        .col_between(Pixels(12.0))
        .left(Pixels(4.0))
        .top(Pixels(4.0));

        // VStack::new(cx, |cx| {
        //     Label::new(cx, "Clip")
        //         .font_family(vec![FamilyOwned::Name(String::from(
        //             assets::NOTO_SANS_THIN,
        //         ))])
        //         .font_size(30.0)
        //         .height(Pixels(50.0))
        //         .child_top(Stretch(1.0))
        //         .child_bottom(Pixels(0.0));

        //     Label::new(cx, "Drive");
        //     ParamSlider::new(cx, Data::params, |params| &params.drive);

        //     Label::new(cx, "Hardness");
        //     ParamSlider::new(cx, Data::params, |params| &params.hardness);

        //     PeakMeter::new(
        //         cx,
        //         Data::peak_meter
        //             .map(|peak_meter| util::gain_to_db(peak_meter.load(Ordering::Relaxed))),
        //         Some(Duration::from_millis(600)),
        //     )
        //     // This is how adding padding works in vizia
        //     .top(Pixels(10.0));
        // })
        // .row_between(Pixels(0.0))
        // .child_left(Stretch(1.0))
        // .child_right(Stretch(1.0));
    })
}
