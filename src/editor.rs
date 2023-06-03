use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;

use crate::ClipParams;

#[derive(Lens)]
struct Data {
    params: Arc<ClipParams>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (572, 174))
}

pub(crate) fn create(
    params: Arc<ClipParams>,
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
            VStack::new(cx, |cx| {
                Label::new(cx, "Bypass");
                Element::new(cx)
                    .height(Pixels(3.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                ParamButton::new(cx, Data::params, |params| &params.bypass);
                Element::new(cx)
                    .height(Pixels(6.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                Label::new(cx, "Pre Gain");
                Element::new(cx)
                    .height(Pixels(3.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                ParamSlider::new(cx, Data::params, |params| &params.pre_gain);
                Element::new(cx)
                    .height(Pixels(6.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                Label::new(cx, "Post Gain");
                Element::new(cx)
                    .height(Pixels(3.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                ParamSlider::new(cx, Data::params, |params| &params.post_gain);
            })
            .border_width(Pixels(1.0))
            .border_color(Color::rgb(255, 0, 0));

            // center column
            VStack::new(cx, |cx| {
                Label::new(cx, "Hardness");
                Element::new(cx)
                    .height(Pixels(3.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                ParamSlider::new(cx, Data::params, |params| &params.hardness);
                Element::new(cx)
                    .height(Pixels(6.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                Label::new(cx, "Drive");
                Element::new(cx)
                    .height(Pixels(3.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                ParamSlider::new(cx, Data::params, |params| &params.drive);
                Element::new(cx)
                    .height(Pixels(6.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                Label::new(cx, "Threshold");
                Element::new(cx)
                    .height(Pixels(3.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                ParamSlider::new(cx, Data::params, |params| &params.threshold);
            })
            .border_width(Pixels(1.0))
            .border_color(Color::rgb(255, 0, 0));

            // right column
            VStack::new(cx, |cx| {
                Label::new(cx, "Mix");
                Element::new(cx)
                    .height(Pixels(3.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                ParamSlider::new(cx, Data::params, |params| &params.mix);
                Element::new(cx)
                    .height(Pixels(6.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                Label::new(cx, "DC Block");
                Element::new(cx)
                    .height(Pixels(3.0))
                    .border_width(Pixels(1.0))
                    .border_color(Color::rgb(255, 0, 0));
                ParamButton::new(cx, Data::params, |params| &params.dc_block);
            })
            .border_width(Pixels(1.0))
            .border_color(Color::rgb(255, 0, 0));
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
